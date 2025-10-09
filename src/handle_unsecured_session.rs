use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::handle_starttls::handle_starttls;

pub async fn handle_unsecured_session(stream: TcpStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream);
    
    let mut is_tls = false;
    let mut line = String::new();
    while reader.read_line(&mut line).await? != 0 {
        let space_pos = line.find(" ").unwrap_or(line.len());
        let (command, _) = line.split_at(space_pos);

        match command.trim().to_uppercase().as_ref() {
            "EHLO" | "HELO" => {
                let stream = reader.get_mut();
                stream.write_all(b"250-windmill Hello\r\n").await?;
                stream.write_all(b"250-STARTTLS\r\n").await?;
                stream.write_all(b"250 What you've got?\r\n").await?;
                stream.flush().await?;
            }
            "STARTTLS" => {
                let stream = reader.get_mut();
                stream.write_all(b"220 GO ON\r\n").await?;
                stream.flush().await?;
                is_tls = true;
                break;
            }
            "QUIT" => {
                let stream = reader.get_mut();
                stream.write_all(b"221 Have a nice day!\r\n").await?;
                stream.flush().await?;
                break;
            }
            "NOOP" => {
                let stream = reader.get_mut();
                stream.write_all(b"250 OK\r\n").await?;
                stream.flush().await?;
            }
            "MAIL" | "RCPT" | "DATA" | "RSET" => {
                let stream = reader.get_mut();
                stream
                    .write_all(b"530 Must issue a STARTTLS command first\r\n")
                    .await?;
                stream.flush().await?;
            }
            _ => {
                let stream = reader.get_mut();
                stream.write_all(b"500 Unknown command\r\n").await?;
                stream.flush().await?;
            }
        }

        line.clear();
    }

    if is_tls {
        // Extract the stream from the BufReader for TLS handshake
        let stream = reader.into_inner();
        handle_starttls(stream).await?;
    }
    
    Ok(())
}
