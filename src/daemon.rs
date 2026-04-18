use crate::account::Accounts;
use crate::config::Config;
use crate::matrix::Event;
use crate::message::Message;
use crate::queue::Queue;
use crate::server::{self, Server};
use anyhow::Context;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

struct Daemon {
    config: Config,
    server: Server,
    queue: Queue,
    accounts: Accounts,
    matrix_clients: HashMap<u32, mpsc::Sender<Event>>,
    done: bool,
}

impl Daemon {
    fn new(config: Config, server: Server) -> Self {
        Daemon {
            config,
            server,
            queue: Queue::new(),
            accounts: Accounts::new(),
            matrix_clients: HashMap::new(),
            done: false,
        }
    }

    async fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        debug!(%msg, "Handling message");
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
                let accounts = self.accounts.list();
                for account in &accounts {
                    let msg = Message::Account {
                        id: account.id.to_string(),
                        name: account.get_name(),
                        protocol: account.protocol.clone(),
                        user: account.user.clone(),
                        status: account.get_status(),
                    };
                    self.queue.send(msg).await; // TODO: improve
                }
                self.queue.send(Message::info("listed accounts.")).await; // TODO: improve
                if accounts.is_empty() {
                    for txt in [
                        "You do not have any accounts configured.",
                        "You can add a new matrix account with the following command: \
                            account add matrix <username>@<server> <password>",
                        "Example: account add matrix dummy@yourserver.org YourPassword",
                    ] {
                        self.queue.send(Message::info(txt)).await; // TODO: improve
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
                if let Err(err) = self.accounts.save(&self.config.accounts_file, self.config.accounts_file_permissions).await {
                    error!(file = %self.config.accounts_file.to_string_lossy(), permissions=self.config.accounts_file_permissions, error = %err, "Could not save accounts to file");
                }
                Ok(())
            }
            Message::AccountDelete { id } => {
                if let Ok(id) = id.parse::<u32>() {
                    self.accounts.remove(&id);
                    if let Err(err) = self.accounts.save(&self.config.accounts_file, self.config.accounts_file_permissions).await {
                        error!(file = %self.config.accounts_file.to_string_lossy(), permissions=self.config.accounts_file_permissions, error = %err, "Could not save accounts to file");
                    }
                }
                Ok(())
            }

            Message::MessageCollect { account_id } => {
                if account_id.parse::<u32>().is_ok() {
                    let msg = Message::error("history is not supported");
                    self.queue.send(msg).await; // TODO: improve
                };
                Ok(())
            }

            Message::BuddyList { account_id, status } => {
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::BuddyList { account_id, status }))
                        .await
                {
                    error!(%error, "Could not send buddy list message");
                }
                Ok(())
            }

            Message::MessageSend {
                account_id,
                destination,
                message,
            } => {
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::MessageSend {
                            account_id,
                            destination,
                            message,
                        }))
                        .await
                {
                    error!(%error, "Could not send message send message");
                }
                Ok(())
            }

            Message::StatusGet { account_id } => {
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::StatusGet { account_id }))
                        .await
                {
                    error!(%error, "Could not send status get message");
                }
                Ok(())
            }

            Message::StatusSet { account_id, status } => {
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::StatusSet { account_id, status }))
                        .await
                {
                    error!(%error, "Could not send status set message");
                }
                Ok(())
            }

            Message::ChatList { account_id } => {
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::ChatList { account_id }))
                        .await
                {
                    error!(%error, "Could not send chat list message");
                }
                Ok(())
            }

            Message::ChatJoin { account_id, chat } => {
                info!("Received chat join message");
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::ChatJoin { account_id, chat }))
                        .await
                {
                    error!(%error, "Could not send chat join message");
                }
                info!("Forwarded chat join message to be sent");
                Ok(())
            }

            Message::ChatLeave { account_id, chat } => {
                info!("Received chat leave message");
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::ChatLeave { account_id, chat }))
                        .await
                {
                    error!(%error, "Could not send chat leave message");
                }
                info!("Forwarded chat leave message to be sent");
                Ok(())
            }

            Message::ChatMessageSend {
                account_id,
                chat,
                message,
            } => {
                info!("Received chat message send message");
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::ChatMessageSend {
                            account_id,
                            chat,
                            message,
                        }))
                        .await
                {
                    error!(%error, "Could not send chat send message");
                }
                info!("Forwarded chat message send message to be sent");
                Ok(())
            }

            Message::ChatUserList { account_id, chat } => {
                info!("Received chat user list message");
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::ChatUserList { account_id, chat }))
                        .await
                {
                    error!(%error, "Could not send chat user list message");
                }
                Ok(())
            }

            Message::ChatUserInvite {
                account_id,
                chat,
                user,
            } => {
                info!("Received chat user invite message");
                if let Ok(id) = account_id.parse::<u32>()
                    && let Some(client) = self.matrix_clients.get(&id)
                    && let Err(error) = client
                        .send(Event::Message(Message::ChatUserInvite {
                            account_id,
                            chat,
                            user,
                        }))
                        .await
                {
                    error!(%error, "Could not send chat user invite message");
                }
                Ok(())
            }

            _ => {
                self.queue.send(msg).await; // TODO: improve
                Ok(())
            }
        }
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        if let Err(err) = self.accounts.load(&self.config.accounts_file).await {
            warn!(file = %self.config.accounts_file.to_string_lossy(), error = %err, "Could not load accounts from file");
        }

        // create channel for matrix events
        let (from_matrix_tx, mut from_matrix_rx) = mpsc::channel(1);

        // start accounts
        // TODO: move/improve?
        for account in self.accounts.list() {
            if account.protocol != "matrix" {
                continue;
            }
            let tx = account.start(self.config.clone(), from_matrix_tx.clone());
            self.matrix_clients.insert(account.id, tx);
        }

        loop {
            if self.done {
                info!("Stopping daemon...");
                return Ok(());
            }
            tokio::select! {
                // handle new client connection
                c = self.server.next() => {
                    // server broken?
                    let mut c = c.context("Could not get client from server")?;

                    // only one client connection is handled at the same time
                    if self.queue.has_client() {
                        // client already connected, decline connection
                        _ = c.send_message(Message::error_already_connected()).await;
                        continue;
                    }
                    if let Err(err) = c.send_message(Message::info_welcome()).await {
                        error!(error = %err, "Error sending welcome message to client");
                        continue;
                    }
                    self.queue.set_client(Some(c)).await;
                },

                // handle message from client
                Some(msg) = self.queue.get_message() => match msg {
                    Some(msg) => {
                        if let Err(err) = self.handle_message(msg).await {
                            // client broken?
                            error!(error = %err, "Error handling message");
                            self.queue.set_client(None).await;
                            continue;
                        }
                    }
                    None => {
                        // client broken?
                        error!("Error getting message from client");
                        self.queue.set_client(None).await;
                    }
                },

                // handle matrix event
                Some(event) = from_matrix_rx.recv() => {
                    info!(?event, "Received matrix event");
                    match event {
                        Event::Message(msg) => self.queue.send(msg).await,
                    }
                }
            }
        }
    }
}

pub async fn run_daemon(config: Config) -> anyhow::Result<()> {
    // create dir with permissions
    tokio::fs::DirBuilder::new()
        .recursive(true)
        .mode(config.dir_permissions)
        .create(&config.dir)
        .await?;
    let server = Server::listen(server::Config::default()).await?;
    info!(address = %server.listen_address()?, "Starting daemon...");
    Daemon::new(config, server).run().await
}
