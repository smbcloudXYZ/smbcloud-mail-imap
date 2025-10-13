use crate::handle_pop3_session::handle_pop3_session;
use crate::pop3_storage::Pop3Mailbox;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn listen_pop3(mailbox: Pop3Mailbox) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 2110));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("POP3 server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mailbox_clone = mailbox.clone();
                tokio::spawn(async move {
                    if let Err(err) = handle_pop3_session(stream, mailbox_clone).await {
                        tracing::error!("Error handling POP3 connection: {:?}", err);
                    }
                });
            }
            Err(err) => {
                tracing::error!("Error establishing POP3 connection: {:?}", err);
            }
        }
    }
}
