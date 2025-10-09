use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use crate::handle_session::handle_session;

pub async fn listen() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 2525));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("SMTP server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                tokio::spawn(async move {
                    if let Err(err) = handle_connection(&mut stream).await {
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

async fn handle_connection(stream: &mut TcpStream) -> anyhow::Result<()> {
    // Send greeting
    stream.write_all(b"220 My SMTP server\r\n").await?;
    
    // Handle the session
    handle_session(stream).await
}
