use crate::{handle_starttls::handle_starttls, storage::MessageStorage, mail_mode::MailMode};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
};

pub async fn handle_unsecured_session(stream: &mut TcpStream, storage: Option<MessageStorage>, mode: MailMode) -> anyhow::Result<()> {
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    let mut is_tls = false;
    let mut line = String::new();
    while reader.read_line(&mut line).await? != 0 {
        let space_pos = line.find(" ").unwrap_or(line.len());
        let (command, _) = line.split_at(space_pos);

        match command.trim().to_uppercase().as_ref() {
            "EHLO" | "HELO" => {
                writer.write_all(b"250-windmill Hello\r\n").await?;
                writer.write_all(b"250-STARTTLS\r\n").await?;
                writer.write_all(b"250 What you've got?\r\n").await?;
                writer.flush().await?;
            }
            "STARTTLS" => {
                writer.write_all(b"220 GO ON\r\n").await?;
                writer.flush().await?;
                is_tls = true;
                break;
            }
            "QUIT" => {
                writer.write_all(b"221 Have a nice day!\r\n").await?;
                writer.flush().await?;
                break;
            }
            "NOOP" => {
                writer.write_all(b"250 OK\r\n").await?;
                writer.flush().await?;
            }
            "MAIL" | "RCPT" | "DATA" | "RSET" => {
                writer
                    .write_all(b"530 Must issue a STARTTLS command first\r\n")
                    .await?;
                writer.flush().await?;
            }
            _ => {
                writer.write_all(b"500 Unknown command\r\n").await?;
                writer.flush().await?;
            }
        }

        line.clear();
    }

    if is_tls {
        handle_starttls(stream, storage, mode).await?;
    }

    Ok(())
}
