use futures::{SinkExt, StreamExt, stream::iter};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};

use crate::send_commands::send_commands;

enum SmtpState {
    Command,
    Data,
    Quit,
}

pub async fn handle_session(mut stream: TcpStream) -> anyhow::Result<()> {
    let RE_SMTP_MAIL = regex::Regex::new(r"(?i)from: ?<(.+)>").unwrap();
    let RE_SMTP_RCPT = regex::Regex::new(r"(?i)to: ?<(.+)>").unwrap();
    let mut message = String::new();
    let mut state = SmtpState::Command;
    let mut mailfrom: Option<String> = None;
    let mut rcpts: Vec<String> = Vec::new();
    let mut framed = Framed::new(stream, LinesCodec::new());
    while let Some(line_str) = framed.next().await {
        let line = line_str?;
        match state {
            SmtpState::Command => {
                let space_pos = line.find(" ").unwrap_or(line.len());
                let (command, arg) = line.split_at(space_pos);
                let arg = arg.trim();
                match &*command.trim().to_uppercase() {
                    "HELO" | "EHLO" => {
                        send_commands(&mut framed, vec!["250 Hello".to_string()]).await?;
                    }
                    "MAIL" => {
                        // Handle MAIL FROM command
                        if let Some(address) = RE_SMTP_MAIL.captures(arg).and_then(|cap| cap.get(1))
                        {
                            mailfrom = Some(address.as_str().to_string());
                            send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                        } else {
                            send_commands(
                                &mut framed,
                                vec!["501 Syntax: MAIL From: <address>".to_string()],
                            )
                            .await?;
                        }
                    }
                    "RCPT" => {
                        // Handle RCPT TO command
                        if mailfrom.is_none() {
                            send_commands(
                                &mut framed,
                                vec!["503 Error: Send MAIL first".to_string()],
                            )
                            .await?;
                        } else {
                            if let Some(address) =
                                RE_SMTP_RCPT.captures(arg).and_then(|cap| cap.get(1))
                            {
                                rcpts.push(address.as_str().to_string());
                                send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                            } else {
                                send_commands(
                                    &mut framed,
                                    vec!["501 Syntax: RCPT TO: <address>".to_string()],
                                )
                                .await?;
                            }
                        }
                    }
                    "DATA" => {
                        if rcpts.is_empty() {
                            send_commands(&mut framed, vec!["503 Error: MAIL FROM and RCPT TO must be set before sending DATA".to_string()]).await?;
                        } else {
                            state = SmtpState::Data;
                            send_commands(
                                &mut framed,
                                vec!["354 End data with <CR><LF>.<CR><LF>".to_string()],
                            )
                            .await?;
                        }
                    }
                    "NOOP" => {
                        send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                    }
                    "RSET" => {
                        mailfrom = None;
                        rcpts = Vec::new();
                        message = String::new();
                        send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                    }
                    "QUIT" => {
                        send_commands(&mut framed, vec!["221 Bye".to_string()]).await?;
                        state = SmtpState::Quit;
                    }
                    _ => {
                        send_commands(&mut framed, vec!["500 Unknown command".to_string()]).await?;
                    }
                }
            }
            SmtpState::Data => {
                if line.trim() == "." {
                    // The end of the email content has been received
                    send_commands(&mut framed, vec!["250 OK".to_string()]).await?;
                    // reset the state and variables for the next email
                    mailfrom = None;
                    rcpts = Vec::new();
                    message = String::new();
                    state = SmtpState::Command;
                    // we can now handle the email:
                    //handle_email(mailfrom, rcpts, message);
                    panic!()
                } else {
                    // Add the received line to the email content
                    message.push_str(&line);
                    message.push_str("\n");
                }
            }
            SmtpState::Quit => {
                break;
            }
        }
    }
    Ok(())
}
