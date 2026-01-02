use matrix_sdk::{
    config::SyncSettings,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
    Room, RoomState,
};
use tracing::{debug, info};

pub struct Client {
    server: String,
    user: String,
    password: String,
}

impl Client {
    pub fn new(server: &str, user: &str, password: &str) -> Self {
        Client {
            server: server.into(),
            user: user.into(),
            password: password.into(),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let client = matrix_sdk::Client::builder()
            .homeserver_url(&self.server)
            .build()
            .await?;
        client
            .matrix_auth()
            .login_username(&self.user, &self.password)
            .initial_device_display_name("nuqql-matrixd-rs")
            .await?;

        debug!(self.server, self.user, "Matrix client logged");

        // TODO: do proper syncing
        let response = client.sync_once(SyncSettings::default()).await.unwrap();
        client.add_event_handler(Self::handle_room_message);
        let settings = SyncSettings::default().token(response.next_batch);
        client.sync(settings).await?;

        Ok(())
    }

    async fn handle_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
        if room.state() != RoomState::Joined {
            return;
        }
        let MessageType::Text(text_content) = event.content.msgtype else {
            return;
        };

        // TODO: handle messages properly
        info!(text_content.body);
    }
}
