use crate::account::{Accounts, ACCOUNTS_FILE};
use crate::message::Message;
use crate::queue::Queue;
use crate::server::{Config, Server};
use tokio::sync::mpsc;

struct Daemon {
    server: Server,
    queue: Queue,
    accounts: Accounts,
    done: bool,
}

impl Daemon {
    fn new(server: Server) -> Self {
        Daemon {
            server: server,
            queue: Queue::new(),
            accounts: Accounts::new(),
            done: false,
        }
    }

    async fn handle_message(
        &mut self,
        msg: Message,
    ) -> Result<(), mpsc::error::SendError<Message>> {
        print!("{msg}");
        match msg {
            Message::Help => {
                let msg = Message::info_help();
                self.queue.send(msg).await; // TODO: improve
                Ok(())
            }
            Message::Bye => {
                self.queue.set_client(None).await;
                Ok(())
            }
            Message::Quit => {
                self.done = true;
                Ok(())
            }
            Message::Version => {
                let msg = Message::info_version();
                self.queue.send(msg).await; // TODO: improve
                Ok(())
            }
            Message::AccountList => {
                for account in self.accounts.list() {
                    let msg = Message::Account {
                        id: account.id.to_string(),
                        name: "()".into(),
                        protocol: account.protocol.clone(),
                        user: account.user.clone(),
                        status: account.get_status(),
                    };
                    self.queue.send(msg).await; // TODO: improve
                }
                Ok(())
            }
            Message::AccountAdd {
                protocol,
                user,
                password,
            } => {
                self.accounts.add(protocol, user, password);
                self.accounts.save(ACCOUNTS_FILE).await.unwrap(); // TODO: improve
                Ok(())
            }
            Message::AccountDelete { id } => {
                if let Ok(id) = id.parse::<u32>() {
                    self.accounts.remove(&id);
                    self.accounts.save(ACCOUNTS_FILE).await.unwrap(); // TODO: improve
                }
                Ok(())
            }
            _ => {
                self.queue.send(msg).await; // TODO: improve
                Ok(())
            }
        }
    }

    async fn run(&mut self) -> std::io::Result<()> {
        if let Err(err) = self.accounts.load(ACCOUNTS_FILE).await {
            // TODO: improve
            println!("could not load accounts: {err}");
        }

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
                        if self.queue.has_client() {
                            // client already connected, decline connection
                            _ = c.send_message(Message::info_already_connected()).await;
                            continue;
                        }
                        if let Err(err) = c.send_message(Message::info_welcome()).await {
                            println!("Error sending welcome message to client: {err}");
                            continue;
                        }
                        self.queue.set_client(Some(c)).await;
                    }
                    Err(err) => {
                        // server broken?
                        println!("Error getting client: {err}");
                        return Err(err);
                    }
                },

                // handle message from client
                Some(msg) = self.queue.get_message() => match msg {
                    Some(msg) => {
                        if let Err(err) = self.handle_message(msg).await {
                            // client broken?
                            println!("Error handling message: {err}");
                            self.queue.set_client(None).await;
                            continue;
                        }
                    }
                    None => {
                        // client broken?
                        println!("Error getting message from client");
                        self.queue.set_client(None).await;
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
