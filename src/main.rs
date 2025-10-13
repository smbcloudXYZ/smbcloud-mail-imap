use anyhow::Result;
use smbcloud_mail::listen::listen;
use smbcloud_mail::listen_pop3::listen_pop3;
use smbcloud_mail::pop3_storage::Pop3Mailbox;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Create shared POP3 mailbox
    let mailbox = Pop3Mailbox::new();
    
    // Add some sample messages for testing
    mailbox.add_message("test", "From: sender@example.com\r\nTo: test@example.com\r\nSubject: Test Message 1\r\n\r\nThis is the first test message.".to_string());
    mailbox.add_message("test", "From: sender@example.com\r\nTo: test@example.com\r\nSubject: Test Message 2\r\n\r\nThis is the second test message.".to_string());

    // Run both SMTP and POP3 servers concurrently
    let smtp_server = tokio::spawn(async move {
        if let Err(e) = listen().await {
            tracing::error!("SMTP server error: {:?}", e);
        }
    });

    let pop3_server = tokio::spawn(async move {
        if let Err(e) = listen_pop3(mailbox).await {
            tracing::error!("POP3 server error: {:?}", e);
        }
    });

    // Wait for both servers (they run indefinitely)
    tokio::select! {
        _ = smtp_server => {},
        _ = pop3_server => {},
    }

    Ok(())
}
