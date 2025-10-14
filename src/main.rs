use anyhow::Result;
use smbcloud_mail::{listen::listen, storage::MessageStorage, mail_mode::MailMode};

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

    // Clone storage for submission SMTP server (port 2526)
    let submit_storage = storage.clone();
    
    // Start submission SMTP server (port 2526) in a separate task
    // Port 2526: Mail submission port for outgoing mail
    // - Used by mail clients to submit outgoing messages
    // - Messages are logged and would be relayed to external mail servers
    // - In Submit mode, messages are not stored in INBOX
    tokio::spawn(async move {
        if let Err(err) = listen(submit_storage, 2526, MailMode::Submit).await {
            tracing::error!("SMTP submission server error: {:?}", err);
        }
    });

    // Start receiving SMTP server (port 2525) - this will block
    // Port 2525: Mail receiving port for incoming mail
    // - Used to receive incoming messages from other mail servers
    // - Messages are stored in the INBOX mailbox
    // - In Receive mode, messages are persisted for IMAP access
    listen(storage, 2525, MailMode::Receive).await
}
