use crate::message::Message;
use std::collections::VecDeque;

pub struct Queue {
    q: VecDeque<Message>,
}

impl Queue {
    pub fn new() -> Self {
        Queue { q: VecDeque::new() }
    }

    pub fn send(&mut self, message: Message) {
        self.q.push_back(message);
    }
}
