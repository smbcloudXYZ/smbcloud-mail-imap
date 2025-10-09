use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use crate::handle_unsecured_session::handle_unsecured_session;

pub async fn listen() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 2525));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("SMTP server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(err) = handle_connection(stream).await {
                        tracing::error!("Error handling SMTP connection: {:?}", err);
                    };
                });
            }
            Err(err) => {
                tracing::error!("Error establishing SMTP connection: {:?}", err);
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    // Send initial greeting
    stream.write_all(b"220 My SMTP server\r\n").await?;
    stream.flush().await?;

    // Handle unsecured session which will handle STARTTLS
    handle_unsecured_session(stream).await?;
    Ok(())
}
