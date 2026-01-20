use crate::message::DataChannelMessage;
use std::collections::VecDeque;

pub struct TransientStorage {
    messages: VecDeque<DataChannelMessage>,
    capacity: usize,
}

impl TransientStorage {
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add_message(&mut self, message: DataChannelMessage) {
        if self.messages.len() >= self.capacity {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    pub fn get_messages(&self) -> Vec<DataChannelMessage> {
        self.messages.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
