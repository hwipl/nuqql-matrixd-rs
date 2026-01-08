use matrix_sdk::{
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    ruma::api::client::filter::FilterDefinition,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
    LoopCtrl, Room, RoomState,
};
use tracing::{debug, info};

pub struct Client {
    server: String,
    user: String,
    password: String,

    session_file: std::path::PathBuf,
    db_path: std::path::PathBuf,
}

impl Client {
    pub fn new(server: &str, user: &str, password: &str) -> Self {
        Client {
            server: server.into(),
            user: user.into(),
            password: password.into(),

            session_file: std::path::PathBuf::from(format!("session-{}-{}", server, user)),
            db_path: std::path::PathBuf::from(format!("db-{}-{}", server, user)),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let client = if self.session_file.exists() {
            self.restore_session().await?
        } else {
            self.login().await?
        };

        debug!(self.server, self.user, "Matrix client logged in");
        self.sync(client).await
    }

    async fn restore_session(&self) -> anyhow::Result<matrix_sdk::Client> {
        println!(
            "Previous session found in '{}'",
            self.session_file.to_string_lossy()
        );

        // The session was serialized as JSON in a file.
        let serialized_session = tokio::fs::read_to_string(&self.session_file).await?;
        let user_session: MatrixSession = serde_json::from_str(&serialized_session)?;

        // Build the client with the previous settings from the session.
        let client = matrix_sdk::Client::builder()
            .server_name_or_homeserver_url(&self.server)
            //.sqlite_store(client_session.db_path, Some(&passphrase))
            .sqlite_store(&self.db_path, None)
            .build()
            .await?;

        println!("Restoring session for {}…", user_session.meta.user_id);

        // Restore the Matrix user session.
        client.restore_session(user_session).await?;

        Ok(client)
    }

    /// Login with a new device.
    async fn login(&self) -> anyhow::Result<matrix_sdk::Client> {
        println!("No previous session found, logging in…");

        let client = matrix_sdk::Client::builder()
            .server_name_or_homeserver_url(&self.server)
            //.sqlite_store(&db_path, Some(&passphrase))
            .sqlite_store(&self.db_path, None)
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
        let serialized_session = serde_json::to_string(&user_session)?;
        tokio::fs::write(&self.session_file, serialized_session).await?;

        println!(
            "Session persisted in {}",
            &self.session_file.to_string_lossy()
        );

        // After logging in, you might want to verify this session with another one (see
        // the `emoji_verification` example), or bootstrap cross-signing if this is your
        // first session with encryption, or if you need to reset cross-signing because
        // you don't have access to your old sessions (see the
        // `cross_signing_bootstrap` example).

        Ok(client)
    }

    /// Setup the client to listen to new messages.
    async fn sync(&self, client: matrix_sdk::Client) -> anyhow::Result<()> {
        // Enable room members lazy-loading, it will speed up the initial sync a lot
        // with accounts in lots of rooms.
        // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
        let filter = FilterDefinition::with_lazy_loading();
        let sync_settings = SyncSettings::default().filter(filter.into());

        client.add_event_handler(Self::handle_room_message);
        client
            .sync_with_result_callback(sync_settings, |sync_result| async move {
                sync_result?;
                Ok(LoopCtrl::Continue)
            })
            .await?;

        Ok(())
    }

    async fn handle_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
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
                println!("Error getting room display name: {error}");
                // Let's fallback to the room ID.
                room.room_id().to_string()
            }
        };

        println!("[{room_name}] {}: {}", event.sender, text_content.body)
    }
}
