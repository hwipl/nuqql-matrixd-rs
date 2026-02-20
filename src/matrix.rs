use crate::message::Message;
use matrix_sdk::{
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    event_handler::Ctx,
    ruma::api::client::filter::FilterDefinition,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
    ruma::RoomId,
    LoopCtrl, Room, RoomState,
};
use std::os::unix::fs::PermissionsExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

const SESSION_FILE_PERMISSIONS: u32 = 0o600;
const DB_FILE_PERMISSIONS: u32 = 0o600;
const DIR_PERMISSIONS: u32 = 0o700;

#[derive(Debug)]
pub enum Event {
    Message(Message),
}

pub struct Client {
    server: String,
    user: String,
    password: String,

    session_file: std::path::PathBuf,
    db_path: std::path::PathBuf,
    db_passphrase: String,
    secret_store_key: String,
}

impl Client {
    pub fn new(
        server: &str,
        user: &str,
        password: &str,
        db_passphrase: &str,
        secret_store_key: &str,
    ) -> Self {
        Client {
            server: server.into(),
            user: user.into(),
            password: password.into(),

            session_file: ["data", server, user, "session"].iter().collect(),
            db_path: ["data", server, user, "db"].iter().collect(),
            db_passphrase: db_passphrase.into(),
            secret_store_key: secret_store_key.into(),
        }
    }

    pub async fn start(
        &self,
        account_id: u32,
        from_matrix: mpsc::Sender<Event>,
        mut to_matrix: mpsc::Receiver<Event>,
    ) -> anyhow::Result<()> {
        // client
        let client = if self.session_file.exists() {
            self.restore_session().await?
        } else {
            self.login().await?
        };
        self.set_session_permissions().await?;
        self.set_db_permissions().await?;

        // secret store
        if self.secret_store_key != "" {
            match client
                .encryption()
                .secret_storage()
                .open_secret_store(&self.secret_store_key)
                .await
            {
                Ok(store) => {
                    if let Err(error) = store.import_secrets().await {
                        error!(%error, "Could not import secrets from secret store");
                    }
                }
                Err(error) => error!(%error, "Could not open secret store"),
            }
        }

        // client sync (incoming events from matrix)
        debug!(self.server, self.user, "Matrix client logged in");
        let c = client.clone();
        tokio::spawn(async move { Self::sync(c, account_id, from_matrix).await });

        // handle events (outgoing events to matrix)
        while let Some(msg) = to_matrix.recv().await {
            info!("Received event message to be handled by matrix");
            match msg {
                Event::Message(Message::ChatMessageSend { chat, message, .. }) => {
                    info!("Received chat message send message to be sent");
                    let Ok(room_id) = RoomId::parse(chat) else {
                        continue;
                    };
                    let Some(room) = client.get_room(&room_id) else {
                        continue;
                    };
                    let content = RoomMessageEventContent::text_plain(message);
                    if let Err(error) = room.send(content).await {
                        error!(%error, "Could not send message to room");
                    };
                }
                _ => (),
            };
        }
        Ok(())
    }

    async fn restore_session(&self) -> anyhow::Result<matrix_sdk::Client> {
        info!(
            session_file = %self.session_file.to_string_lossy(),
            "Previous session found'"
        );

        // The session was serialized as JSON in a file.
        let serialized_session = tokio::fs::read_to_string(&self.session_file).await?;
        let user_session: MatrixSession = serde_json::from_str(&serialized_session)?;

        // Build the client with the previous settings from the session.
        let client = matrix_sdk::Client::builder()
            .server_name_or_homeserver_url(&self.server)
            .sqlite_store(&self.db_path, Some(&self.db_passphrase))
            .build()
            .await?;

        info!(%user_session.meta.user_id, "Restoring session for user");

        // Restore the Matrix user session.
        client.restore_session(user_session).await?;

        Ok(client)
    }

    /// Login with a new device.
    async fn login(&self) -> anyhow::Result<matrix_sdk::Client> {
        info!("No previous session found, logging in...");

        // create dir with permissions
        tokio::fs::DirBuilder::new()
            .recursive(true)
            .mode(DIR_PERMISSIONS)
            .create(&self.db_path)
            .await?;

        let client = matrix_sdk::Client::builder()
            .server_name_or_homeserver_url(&self.server)
            .sqlite_store(&self.db_path, Some(&self.db_passphrase))
            .build()
            .await?;
        let matrix_auth = client.matrix_auth();

        matrix_auth
            .login_username(&self.user, &self.password)
            .initial_device_display_name("nuqql-matrixd-rs")
            .await?;

        debug!(self.server, self.user, "Matrix client logged");

        // Persist the session to reuse it later.
        // This is not very secure, for simplicity. If the system provides a way of
        // storing secrets securely, it should be used instead.
        // Note that we could also build the user session from the login response.
        let user_session = matrix_auth
            .session()
            .expect("A logged-in client should have a session");
        let serialized_session = serde_json::to_vec(&user_session)?;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(SESSION_FILE_PERMISSIONS)
            .open(&self.session_file)
            .await?;
        file.write_all(&serialized_session).await?;

        info!(
            session_file = %self.session_file.to_string_lossy(),
            "Session persisted",
        );

        // After logging in, you might want to verify this session with another one (see
        // the `emoji_verification` example), or bootstrap cross-signing if this is your
        // first session with encryption, or if you need to reset cross-signing because
        // you don't have access to your old sessions (see the
        // `cross_signing_bootstrap` example).

        Ok(client)
    }

    /// Sets permissions of the session file.
    async fn set_session_permissions(&self) -> anyhow::Result<()> {
        tokio::fs::set_permissions(
            &self.session_file,
            std::fs::Permissions::from_mode(SESSION_FILE_PERMISSIONS),
        )
        .await?;
        Ok(())
    }

    /// Sets permissions of files in db path.
    async fn set_db_permissions(&self) -> anyhow::Result<()> {
        let mut dir = tokio::fs::read_dir(&self.db_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if let Ok(metadata) = entry.metadata().await {
                if !metadata.is_file() {
                    continue;
                }
                tokio::fs::set_permissions(
                    path,
                    std::fs::Permissions::from_mode(DB_FILE_PERMISSIONS),
                )
                .await?;
            } else {
                error!(file = %path.to_string_lossy(), "Could not get metadata of file");
            }
        }
        Ok(())
    }

    /// Setup the client to listen to new messages.
    async fn sync(
        client: matrix_sdk::Client,
        account_id: u32,
        from_matrix: mpsc::Sender<Event>,
    ) -> anyhow::Result<()> {
        // Enable room members lazy-loading, it will speed up the initial sync a lot
        // with accounts in lots of rooms.
        // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
        let filter = FilterDefinition::with_lazy_loading();
        let sync_settings = SyncSettings::default().filter(filter.into());

        client.add_event_handler_context(account_id);
        client.add_event_handler_context(from_matrix);
        client.add_event_handler(Self::handle_room_message);
        client
            .sync_with_result_callback(sync_settings, |sync_result| async move {
                sync_result?;
                Ok(LoopCtrl::Continue)
            })
            .await?;

        Ok(())
    }

    async fn handle_room_message(
        event: OriginalSyncRoomMessageEvent,
        room: Room,
        account_id: Ctx<u32>,
        from_matrix: Ctx<mpsc::Sender<Event>>,
    ) {
        info!(room = %room.room_id(), "Handling room message");

        if room.state() != RoomState::Joined {
            return;
        }
        let MessageType::Text(text_content) = event.content.msgtype else {
            return;
        };

        // TODO: handle messages properly
        info!(text_content.body);

        let room_name = match room.display_name().await {
            Ok(room_name) => room_name.to_string(),
            Err(error) => {
                error!(%error, "Error getting room display name");
                // Let's fallback to the room ID.
                room.room_id().to_string()
            }
        };

        info!("[{room_name}] {}: {}", event.sender, text_content.body);
        if let Err(error) = from_matrix
            .send(Event::Message(Message::ChatMessage {
                account_id: account_id.to_string(),
                chat: room.room_id().to_string(),
                timestamp: event.origin_server_ts.as_secs().to_string(),
                sender: event.sender.to_string(),
                message: text_content.body,
            }))
            .await
        {
            error!(%error, "Could not send message event");
        };
    }
}
