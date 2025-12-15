use crate::account::Accounts;
use crate::message::Message;
use crate::server::{Client, Config, Server};
use tokio::sync::mpsc;

struct Daemon {
    server: Server,
    client: Option<Client>,
    accounts: Accounts,
    done: bool,
}

impl Daemon {
    fn new(server: Server) -> Self {
        Daemon {
            server: server,
            client: None,
            accounts: Accounts::new(),
            done: false,
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
                let client = self.client.as_mut().unwrap();
                return client.send_message(Message::info_help()).await;
            }
            Message::Bye => {
                self.client = None;
                Ok(())
            }
            Message::Quit => {
                self.done = true;
                Ok(())
            }
            Message::Version => {
                let client = self.client.as_mut().unwrap();
                return client.send_message(Message::info_version()).await;
            }
            Message::AccountList => {
                let client = self.client.as_mut().unwrap();
                for account in self.accounts.list() {
                    if let Err(err) = client
                        .send_message(Message::Account {
                            id: account.id.to_string(),
                            name: "NAME_TODO".into(),
                            protocol: account.protocol,
                            user: account.user,
                            status: "STATUS_TODO".into(),
                        })
                        .await
                    {
                        return Err(err);
                    }
                }
                Ok(())
            }
            Message::AccountAdd {
                protocol,
                user,
                password,
            } => {
                self.accounts.add(protocol, user, password);
                Ok(())
            }
            Message::AccountDelete { id } => {
                if let Ok(id) = id.parse::<u32>() {
                    self.accounts.remove(&id);
                }
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
            if self.done {
                println!("Stopping daemon...");
                return Ok(());
            }
            tokio::select! {
                // handle new client connection
                c = self.server.next() => match c {
                    // only one client connection is handled at the same time
                    Ok(mut c) => {
                        if self.client.is_some() {
                            // client already connected, decline connection
                            _ = c.send_message(Message::info_already_connected()).await;
                            continue;
                        }
                        if let Err(err) = c.send_message(Message::info_welcome()).await {
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
    let server = Server::listen(Config::default()).await?;
    println!("Server listening on: {}", server.listen_address()?);
    Daemon::new(server).run().await
}
