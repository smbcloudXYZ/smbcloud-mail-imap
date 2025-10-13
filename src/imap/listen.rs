use super::mailbox::MailboxStore;
use super::session::ImapSession;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

pub async fn listen(store: Arc<MailboxStore>) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 143));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("IMAP server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let store_clone = Arc::clone(&store);
                tokio::spawn(async move {
                    if let Err(err) = handle_connection(stream, store_clone).await {
                        tracing::error!("Error handling IMAP connection: {:?}", err);
                    }
                });
            }
            Err(err) => {
                tracing::error!("Error establishing IMAP connection: {:?}", err);
            }
        }
    }
}

async fn handle_connection(stream: TcpStream, store: Arc<MailboxStore>) -> anyhow::Result<()> {
    let mut session = ImapSession::new((*store).clone());
    session.handle(stream).await
}
