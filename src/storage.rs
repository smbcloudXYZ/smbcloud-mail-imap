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
