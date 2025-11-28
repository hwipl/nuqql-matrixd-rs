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
        match msg {
            Message::Help => {
                let reply = Message::Info {
                    message: "help text".into(),
                };
                let client = self.client.as_mut().unwrap();
                return client.send_message(reply).await;
            }
            Message::Bye => {
                // TODO: improve client disconnect?
                self.client = None;
                Ok(())
            }
            _ => {
                let client = self.client.as_mut().unwrap();
                client.send_message(msg).await
            }
        }
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
                            // TODO: improve client disconnect?
                            _ = c.send_message(Message::Info{message: "client already connected".into()}).await;
                            continue;
                        }
                        if let Err(err) = c.send_message(Message::Info{message: "Welcome to nuqql-matrixd-rs!".into()}).await {
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
