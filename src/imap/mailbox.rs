use super::message::{Message, MessageFlag};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Mailbox {
    pub name: String,
    pub messages: Arc<Mutex<Vec<Message>>>,
    next_uid: Arc<Mutex<u32>>,
}

impl Mailbox {
    pub fn new(name: String) -> Self {
        Mailbox {
            name,
            messages: Arc::new(Mutex::new(Vec::new())),
            next_uid: Arc::new(Mutex::new(1)),
        }
    }

    pub fn add_message(&self, from: String, to: Vec<String>, raw_content: String) -> u32 {
        let mut messages = self.messages.lock().unwrap();
        let mut next_uid = self.next_uid.lock().unwrap();
        
        let uid = *next_uid;
        *next_uid += 1;
        
        let sequence_number = messages.len() as u32 + 1;
        let message = Message::new(uid, sequence_number, from, to, raw_content);
        messages.push(message);
        
        uid
    }

    pub fn get_message_by_sequence(&self, seq: u32) -> Option<Message> {
        let messages = self.messages.lock().unwrap();
        messages.iter()
            .find(|m| m.sequence_number == seq)
            .cloned()
    }

    pub fn get_message_by_uid(&self, uid: u32) -> Option<Message> {
        let messages = self.messages.lock().unwrap();
        messages.iter()
            .find(|m| m.uid == uid)
            .cloned()
    }

    pub fn get_messages(&self) -> Vec<Message> {
        let messages = self.messages.lock().unwrap();
        messages.clone()
    }

    pub fn exists(&self) -> u32 {
        let messages = self.messages.lock().unwrap();
        messages.len() as u32
    }

    pub fn recent(&self) -> u32 {
        let messages = self.messages.lock().unwrap();
        messages.iter()
            .filter(|m| m.has_flag(&MessageFlag::Recent))
            .count() as u32
    }

    pub fn unseen(&self) -> u32 {
        let messages = self.messages.lock().unwrap();
        messages.iter()
            .filter(|m| !m.has_flag(&MessageFlag::Seen))
            .count() as u32
    }

    pub fn uidnext(&self) -> u32 {
        let next_uid = self.next_uid.lock().unwrap();
        *next_uid
    }

    pub fn uidvalidity(&self) -> u32 {
        // For simplicity, using a fixed value
        // In a real implementation, this should be persistent and unique per mailbox
        1
    }
}

#[derive(Debug, Clone)]
pub struct MailboxStore {
    mailboxes: Arc<Mutex<Vec<Mailbox>>>,
}

impl MailboxStore {
    pub fn new() -> Self {
        let store = MailboxStore {
            mailboxes: Arc::new(Mutex::new(Vec::new())),
        };
        
        // Create default mailboxes
        store.create_mailbox("INBOX".to_string());
        store.create_mailbox("Sent".to_string());
        store.create_mailbox("Drafts".to_string());
        store.create_mailbox("Trash".to_string());
        
        store
    }

    pub fn create_mailbox(&self, name: String) -> bool {
        let mut mailboxes = self.mailboxes.lock().unwrap();
        
        // Check if mailbox already exists
        if mailboxes.iter().any(|m| m.name == name) {
            return false;
        }
        
        mailboxes.push(Mailbox::new(name));
        true
    }

    pub fn get_mailbox(&self, name: &str) -> Option<Mailbox> {
        let mailboxes = self.mailboxes.lock().unwrap();
        mailboxes.iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn list_mailboxes(&self) -> Vec<String> {
        let mailboxes = self.mailboxes.lock().unwrap();
        mailboxes.iter()
            .map(|m| m.name.clone())
            .collect()
    }

    pub fn delete_mailbox(&self, name: &str) -> bool {
        let mut mailboxes = self.mailboxes.lock().unwrap();
        if let Some(pos) = mailboxes.iter().position(|m| m.name == name) {
            mailboxes.remove(pos);
            return true;
        }
        false
    }
}

impl Default for MailboxStore {
    fn default() -> Self {
        Self::new()
    }
}
