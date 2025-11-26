use crate::message::Message;
use crate::server::{Client, Server};
use tokio::sync::mpsc;

struct Daemon {
    server: Server,
    client: Option<Client>,
}

impl Daemon {
    fn new(server: Server) -> Self {
        Daemon {
            server: server,
            client: None,
        }
    }

    async fn get_message(client: &mut Option<Client>) -> Option<Option<Message>> {
        match client.as_mut() {
            Some(client) => Some(client.get_message().await),
            None => None,
        }
    }

    async fn handle_message(
        &mut self,
        msg: Message,
    ) -> Result<(), mpsc::error::SendError<Message>> {
        print!("{msg}");
        let client = self.client.as_mut().unwrap();
        match msg {
            Message::Help => {
                let reply = Message::Info {
                    message: "help text".into(),
                };
                return client.send_message(reply).await;
            }
            _ => (),
        }
        client.send_message(msg).await
    }

    async fn run(&mut self) -> std::io::Result<()> {
        loop {
            tokio::select! {
                // handle new client connection
                c = self.server.next() => match c {
                    // only one client connection is handled at the same time
                    Ok(mut c) => {
                        if self.client.is_some() {
                            // client already connected, decline connection
                            // FIXME
                            _ = c.send_message(Message::Info{message: "info: client already connected\r\n".into()}).await;
                            continue;
                        }
                        // FIXME
                        if let Err(err) = c.send_message(Message::Info{message: "info: Welcome to nuqql-matrixd-rs!\r\n".into()}).await {
                            println!("Error sending welcome message to client: {err}");
                            continue;
                        }
                        self.client = Some(c);
                    }
                    Err(err) => {
                        // server broken?
                        println!("Error getting client: {err}");
                        return Err(err);
                    }
                },

                // handle message from client
                Some(msg) = Self::get_message(&mut self.client) => match msg {
                    Some(msg) => {
                        if let Err(err) = self.handle_message(msg).await {
                            // client broken?
                            println!("Error handling message: {err}");
                            self.client = None;
                            continue;
                        }
                    }
                    None => {
                        // client broken?
                        println!("Error getting message from client");
                        self.client = None;
                    }
                }
            }
        }
    }
}

pub async fn run_daemon() -> std::io::Result<()> {
    let server = Server::listen().await?;
    Daemon::new(server).run().await
}
