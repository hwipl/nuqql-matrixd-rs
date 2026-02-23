use crate::account::{Accounts, ACCOUNTS_FILE};
use crate::matrix::Event;
use crate::message::Message;
use crate::queue::Queue;
use crate::server::{Config, Server};
use anyhow::Context;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

struct Daemon {
    server: Server,
    queue: Queue,
    accounts: Accounts,
    matrix_clients: HashMap<u32, mpsc::Sender<Event>>,
    done: bool,
}

impl Daemon {
    fn new(server: Server) -> Self {
        Daemon {
            server: server,
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
                for account in self.accounts.list() {
                    let msg = Message::Account {
                        id: account.id.to_string(),
                        name: account.get_name(),
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
                if let Err(err) = self.accounts.save(ACCOUNTS_FILE).await {
                    error!(file = ACCOUNTS_FILE, error = %err, "Could not save accounts to file");
                }
                Ok(())
            }
            Message::AccountDelete { id } => {
                if let Ok(id) = id.parse::<u32>() {
                    self.accounts.remove(&id);
                    if let Err(err) = self.accounts.save(ACCOUNTS_FILE).await {
                        error!(file = ACCOUNTS_FILE, error = %err, "Could not save accounts to file");
                    }
                }
                Ok(())
            }

            Message::BuddyList { account_id, status } => {
                if let Ok(id) = account_id.parse::<u32>() {
                    if let Some(client) = self.matrix_clients.get(&id) {
                        if let Err(error) = client
                            .send(Event::Message(Message::BuddyList { account_id, status }))
                            .await
                        {
                            error!(%error, "Could not send buddy list message");
                        }
                    };
                };
                Ok(())
            }

            Message::ChatMessageSend {
                account_id,
                chat,
                message,
            } => {
                info!("Received chat message send message");
                if let Ok(id) = account_id.parse::<u32>() {
                    if let Some(client) = self.matrix_clients.get(&id) {
                        if let Err(error) = client
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
                    };
                };
                Ok(())
            }
            _ => {
                self.queue.send(msg).await; // TODO: improve
                Ok(())
            }
        }
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        if let Err(err) = self.accounts.load(ACCOUNTS_FILE).await {
            warn!(file = ACCOUNTS_FILE, error = %err, "Could not load accounts from file");
        }

        // create channel for matrix events
        let (from_matrix_tx, mut from_matrix_rx) = mpsc::channel(1);

        // start accounts
        // TODO: move/improve?
        for account in self.accounts.list() {
            if account.protocol != "matrix" {
                continue;
            }
            let tx = account.start(from_matrix_tx.clone());
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
                        _ = c.send_message(Message::info_already_connected()).await;
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

pub async fn run_daemon() -> anyhow::Result<()> {
    let server = Server::listen(Config::default()).await?;
    info!(address = %server.listen_address()?, "Starting daemon...");
    Daemon::new(server).run().await
}
