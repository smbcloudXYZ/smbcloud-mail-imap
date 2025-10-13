use crate::imap::mailbox::MailboxStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct MessageStorage {
    pub mailbox_store: Arc<MailboxStore>,
}

impl MessageStorage {
    pub fn new() -> Self {
        MessageStorage {
            mailbox_store: Arc::new(MailboxStore::new()),
        }
    }

    pub fn store_message(&self, from: String, to: Vec<String>, content: String) {
        // Store message in INBOX by default
        if let Some(inbox) = self.mailbox_store.get_mailbox("INBOX") {
            inbox.add_message(from, to, content);
            tracing::info!("Message stored in INBOX");
        }
    }
}

impl Default for MessageStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_storage() {
        let storage = MessageStorage::new();
        
        // Store a message
        let from = "sender@example.com".to_string();
        let to = vec!["recipient@example.com".to_string()];
        let content = "Subject: Test\r\n\r\nTest body".to_string();
        
        storage.store_message(from.clone(), to.clone(), content.clone());
        
        // Verify message was stored
        if let Some(inbox) = storage.mailbox_store.get_mailbox("INBOX") {
            let messages = inbox.get_messages();
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].from, from);
            assert_eq!(messages[0].to, to);
            assert_eq!(messages[0].subject, "Test");
        } else {
            panic!("INBOX mailbox not found");
        }
    }

    #[test]
    fn test_multiple_messages() {
        let storage = MessageStorage::new();
        
        // Store multiple messages
        storage.store_message(
            "sender1@example.com".to_string(),
            vec!["recipient@example.com".to_string()],
            "Subject: First\r\n\r\nFirst message".to_string()
        );
        
        storage.store_message(
            "sender2@example.com".to_string(),
            vec!["recipient@example.com".to_string()],
            "Subject: Second\r\n\r\nSecond message".to_string()
        );
        
        // Verify both messages were stored
        if let Some(inbox) = storage.mailbox_store.get_mailbox("INBOX") {
            let messages = inbox.get_messages();
            assert_eq!(messages.len(), 2);
            assert_eq!(messages[0].subject, "First");
            assert_eq!(messages[1].subject, "Second");
        }
    }

    #[test]
    fn test_mailbox_list() {
        let storage = MessageStorage::new();
        let mailboxes = storage.mailbox_store.list_mailboxes();
        
        // Verify default mailboxes exist
        assert!(mailboxes.contains(&"INBOX".to_string()));
        assert!(mailboxes.contains(&"Sent".to_string()));
        assert!(mailboxes.contains(&"Drafts".to_string()));
        assert!(mailboxes.contains(&"Trash".to_string()));
    }
}
