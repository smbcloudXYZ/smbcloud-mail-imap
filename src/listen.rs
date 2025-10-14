use crate::{handle_unsecured_session::handle_unsecured_session, storage::MessageStorage, mail_mode::MailMode};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

pub async fn listen(storage: MessageStorage, port: u16, mode: MailMode) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;

    let mode_str = match mode {
        MailMode::Receive => "RECEIVE",
        MailMode::Submit => "SUBMIT",
    };
    tracing::info!("SMTP server listening on {} (mode: {})", addr, mode_str);

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                let storage_clone = storage.clone();
                tokio::spawn(async move {
                    if let Err(err) = handle_connection(&mut stream, storage_clone, mode).await {
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

async fn handle_connection(stream: &mut TcpStream, storage: MessageStorage, mode: MailMode) -> anyhow::Result<()> {
    // Send greeting
    stream.write_all(b"220 My SMTP server\r\n").await?;

    // Handle unsecured session which will handle STARTTLS
    handle_unsecured_session(stream, Some(storage), mode).await
}
