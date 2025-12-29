use crate::message::Message;
use crate::server::Client;
use std::collections::VecDeque;
use tracing::error;

pub struct Queue {
    q: VecDeque<Message>,
    client: Option<Client>,
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            q: VecDeque::new(),
            client: None,
        }
    }

    pub async fn set_client(&mut self, client: Option<Client>) {
        self.client = client;

        if self.client.is_none() {
            return;
        }
        while let Some(msg) = self.q.pop_front() {
            let client = self.client.as_mut().unwrap();
            if let Err(ref err) = client.send_message(msg.clone()).await {
                // TODO: get send error
                error!(error = %err, "Error sending from queue to client, dropping client");
                self.q.push_front(msg);
                self.client = None;
                return;
            }
        }
    }

    pub fn has_client(&self) -> bool {
        self.client.is_some()
    }

    pub async fn send(&mut self, message: Message) {
        self.q.push_back(message);

        if self.client.is_none() {
            return;
        }
        while let Some(msg) = self.q.pop_front() {
            let client = self.client.as_mut().unwrap();
            if let Err(err) = client.send_message(msg.clone()).await {
                // TODO: get send error
                error!(error = %err, "Error sending from queue to client, dropping client");
                self.q.push_front(msg);
                self.client = None;
                return;
            }
        }
    }

    // TODO: should this really be here?
    pub async fn get_message(&mut self) -> Option<Option<Message>> {
        match self.client.as_mut() {
            Some(client) => Some(client.get_message().await),
            None => None,
        }
    }
}
