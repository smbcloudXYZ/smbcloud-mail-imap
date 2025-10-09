use std::net::SocketAddr;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};

use crate::handle_session::handle_session;

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
    let mut writer = BufWriter::new(&mut stream);
    writer.write_all(b"220 My SMTP server\r\n").await?;
    writer.flush().await?;
    drop(writer);

    // handle session
    handle_session(stream).await
}
