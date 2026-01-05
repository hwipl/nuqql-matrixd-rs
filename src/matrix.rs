use matrix_sdk::{
    authentication::matrix::MatrixSession,
    config::SyncSettings,
    ruma::api::client::filter::FilterDefinition,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
    Error, LoopCtrl, Room, RoomState,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// The full session to persist.
#[derive(Debug, Serialize, Deserialize)]
struct FullSession {
    /// The Matrix user session.
    user_session: MatrixSession,

    /// The latest sync token.
    ///
    /// It is only needed to persist it when using `Client::sync_once()` and we
    /// want to make our syncs faster by not receiving all the initial sync
    /// again.
    #[serde(skip_serializing_if = "Option::is_none")]
    sync_token: Option<String>,
}

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
        let (client, sync_token) = if self.session_file.exists() {
            self.restore_session().await?
        } else {
            (self.login().await?, None)
        };

        debug!(self.server, self.user, "Matrix client logged in");
        self.sync(client, sync_token).await
    }

    async fn restore_session(&self) -> anyhow::Result<(matrix_sdk::Client, Option<String>)> {
        println!(
            "Previous session found in '{}'",
            self.session_file.to_string_lossy()
        );

        // The session was serialized as JSON in a file.
        let serialized_session = tokio::fs::read_to_string(&self.session_file).await?;
        let FullSession {
            user_session,
            sync_token,
        } = serde_json::from_str(&serialized_session)?;

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

        Ok((client, sync_token))
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
        let serialized_session = serde_json::to_string(&FullSession {
            user_session,
            sync_token: None,
        })?;
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
    async fn sync(
        &self,
        client: matrix_sdk::Client,
        initial_sync_token: Option<String>,
    ) -> anyhow::Result<()> {
        println!("Launching a first sync to ignore past messages…");

        // Enable room members lazy-loading, it will speed up the initial sync a lot
        // with accounts in lots of rooms.
        // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
        let filter = FilterDefinition::with_lazy_loading();

        let mut sync_settings = SyncSettings::default().filter(filter.into());

        // We restore the sync where we left.
        // This is not necessary when not using `sync_once`. The other sync methods get
        // the sync token from the store.
        if let Some(sync_token) = initial_sync_token {
            sync_settings = sync_settings.token(sync_token);
        }

        // TODO: do proper syncing
        // Let's ignore messages before the program was launched.
        // This is a loop in case the initial sync is longer than our timeout. The
        // server should cache the response and it will ultimately take less time to
        // receive.
        loop {
            match client.sync_once(sync_settings.clone()).await {
                Ok(response) => {
                    // This is the last time we need to provide this token, the sync method after
                    // will handle it on its own.
                    sync_settings = sync_settings.token(response.next_batch.clone());
                    self.persist_sync_token(response.next_batch).await?;
                    break;
                }
                Err(error) => {
                    println!("An error occurred during initial sync: {error}");
                    println!("Trying again…");
                }
            }
        }

        println!("The client is ready! Listening to new messages…");

        // Now that we've synced, let's attach a handler for incoming room messages.
        client.add_event_handler(Self::handle_room_message);

        // This loops until we kill the program or an error happens.
        client
            .sync_with_result_callback(sync_settings, |sync_result| async move {
                let response = sync_result?;

                // We persist the token each time to be able to restore our session
                self.persist_sync_token(response.next_batch)
                    .await
                    .map_err(|err| Error::UnknownError(err.into()))?;

                Ok(LoopCtrl::Continue)
            })
            .await?;

        Ok(())
    }

    /// Persist the sync token for a future session.
    /// Note that this is needed only when using `sync_once`. Other sync methods get
    /// the sync token from the store.
    async fn persist_sync_token(&self, sync_token: String) -> anyhow::Result<()> {
        let serialized_session = tokio::fs::read_to_string(&self.session_file).await?;
        let mut full_session: FullSession = serde_json::from_str(&serialized_session)?;

        full_session.sync_token = Some(sync_token);
        let serialized_session = serde_json::to_string(&full_session)?;
        tokio::fs::write(&self.session_file, serialized_session).await?;

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
