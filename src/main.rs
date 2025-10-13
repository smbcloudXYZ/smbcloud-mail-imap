use anyhow::Result;
use smbcloud_mail::{listen::listen, storage::MessageStorage};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Create shared message storage
    let storage = MessageStorage::new();
    
    // Clone storage for IMAP server
    let imap_storage = storage.clone();
    
    // Start IMAP server in a separate task
    tokio::spawn(async move {
        if let Err(err) = smbcloud_mail::imap::listen::listen(imap_storage.mailbox_store).await {
            tracing::error!("IMAP server error: {:?}", err);
        }
    });

    // Start SMTP server (this will block)
    listen(storage).await
}
