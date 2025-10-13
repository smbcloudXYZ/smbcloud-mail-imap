use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Pop3Message {
    pub id: usize,
    pub content: String,
    pub size: usize,
    pub deleted: bool,
}

impl Pop3Message {
    pub fn new(id: usize, content: String) -> Self {
        let size = content.len();
        Self {
            id,
            content,
            size,
            deleted: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pop3Mailbox {
    messages: Arc<Mutex<HashMap<String, Vec<Pop3Message>>>>,
    next_id: Arc<Mutex<HashMap<String, usize>>>,
}

impl Pop3Mailbox {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_message(&self, username: &str, content: String) {
        let mut messages = self.messages.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        
        let user_messages = messages.entry(username.to_string()).or_insert_with(Vec::new);
        let id_counter = next_id.entry(username.to_string()).or_insert(1);
        let id = *id_counter;
        *id_counter += 1;
        
        user_messages.push(Pop3Message::new(id, content));
    }

    pub fn get_messages(&self, username: &str) -> Vec<Pop3Message> {
        let messages = self.messages.lock().unwrap();
        messages
            .get(username)
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    pub fn mark_deleted(&self, username: &str, msg_id: usize) -> bool {
        let mut messages = self.messages.lock().unwrap();
        if let Some(user_messages) = messages.get_mut(username) {
            if let Some(msg) = user_messages.iter_mut().find(|m| m.id == msg_id) {
                msg.deleted = true;
                return true;
            }
        }
        false
    }

    pub fn reset_deleted(&self, username: &str) {
        let mut messages = self.messages.lock().unwrap();
        if let Some(user_messages) = messages.get_mut(username) {
            for msg in user_messages.iter_mut() {
                msg.deleted = false;
            }
        }
    }

    pub fn delete_marked(&self, username: &str) {
        let mut messages = self.messages.lock().unwrap();
        if let Some(user_messages) = messages.get_mut(username) {
            // Delete marked messages without re-indexing
            // This preserves message IDs for any remaining messages
            user_messages.retain(|m| !m.deleted);
        }
    }

    pub fn get_stats(&self, username: &str) -> (usize, usize) {
        let messages = self.messages.lock().unwrap();
        if let Some(user_messages) = messages.get(username) {
            let non_deleted: Vec<_> = user_messages.iter().filter(|m| !m.deleted).collect();
            let count = non_deleted.len();
            let total_size = non_deleted.iter().map(|m| m.size).sum();
            (count, total_size)
        } else {
            (0, 0)
        }
    }
}

impl Default for Pop3Mailbox {
    fn default() -> Self {
        Self::new()
    }
}
