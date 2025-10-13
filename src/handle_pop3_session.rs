use crate::pop3_storage::Pop3Mailbox;
use crate::send_commands::send_commands;
use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Debug, Clone, PartialEq)]
enum Pop3State {
    Authorization,
    Transaction,
    Update,
}

pub async fn handle_pop3_session<S>(stream: S, mailbox: Pop3Mailbox) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut state = Pop3State::Authorization;
    let mut username: Option<String> = None;
    let mut authenticated_user: Option<String> = None;
    
    let mut framed = Framed::new(stream, LinesCodec::new());
    
    // Send greeting
    send_commands(&mut framed, vec!["+OK POP3 server ready".to_string()]).await?;
    
    while let Some(line_str) = framed.next().await {
        let line = line_str?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        
        if parts.is_empty() {
            send_commands(&mut framed, vec!["-ERR Invalid command".to_string()]).await?;
            continue;
        }
        
        let command = parts[0].to_uppercase();
        
        match state {
            Pop3State::Authorization => {
                match command.as_str() {
                    "USER" => {
                        if parts.len() < 2 {
                            send_commands(&mut framed, vec!["-ERR Username required".to_string()]).await?;
                        } else {
                            username = Some(parts[1].to_string());
                            send_commands(&mut framed, vec![format!("+OK User {} accepted", parts[1])]).await?;
                        }
                    }
                    "PASS" => {
                        if username.is_none() {
                            send_commands(&mut framed, vec!["-ERR No username given".to_string()]).await?;
                        } else if parts.len() < 2 {
                            send_commands(&mut framed, vec!["-ERR Password required".to_string()]).await?;
                        } else {
                            // Simple authentication - accept any password for now
                            let user = username.clone().unwrap();
                            authenticated_user = Some(user.clone());
                            
                            let (count, size) = mailbox.get_stats(&user);
                            send_commands(&mut framed, vec![format!("+OK {} messages ({} octets)", count, size)]).await?;
                            state = Pop3State::Transaction;
                        }
                    }
                    "CAPA" => {
                        send_commands(&mut framed, vec![
                            "+OK Capability list follows".to_string(),
                            "USER".to_string(),
                            "UIDL".to_string(),
                            "TOP".to_string(),
                            ".".to_string(),
                        ]).await?;
                    }
                    "QUIT" => {
                        send_commands(&mut framed, vec!["+OK Goodbye".to_string()]).await?;
                        break;
                    }
                    _ => {
                        send_commands(&mut framed, vec!["-ERR Unknown command".to_string()]).await?;
                    }
                }
            }
            Pop3State::Transaction => {
                match command.as_str() {
                    "STAT" => {
                        if let Some(ref user) = authenticated_user {
                            let (count, size) = mailbox.get_stats(user);
                            send_commands(&mut framed, vec![format!("+OK {} {}", count, size)]).await?;
                        } else {
                            send_commands(&mut framed, vec!["-ERR Not authenticated".to_string()]).await?;
                        }
                    }
                    "LIST" => {
                        if let Some(ref user) = authenticated_user {
                            let messages = mailbox.get_messages(user);
                            let non_deleted: Vec<_> = messages.iter().filter(|m| !m.deleted).collect();
                            
                            if parts.len() == 1 {
                                // List all messages
                                let (count, size) = mailbox.get_stats(user);
                                let mut response = vec![format!("+OK {} messages ({} octets)", count, size)];
                                for msg in non_deleted {
                                    response.push(format!("{} {}", msg.id, msg.size));
                                }
                                response.push(".".to_string());
                                send_commands(&mut framed, response).await?;
                            } else if let Ok(msg_id) = parts[1].parse::<usize>() {
                                // List specific message
                                if let Some(msg) = non_deleted.iter().find(|m| m.id == msg_id) {
                                    send_commands(&mut framed, vec![format!("+OK {} {}", msg.id, msg.size)]).await?;
                                } else {
                                    send_commands(&mut framed, vec!["-ERR No such message".to_string()]).await?;
                                }
                            } else {
                                send_commands(&mut framed, vec!["-ERR Invalid message number".to_string()]).await?;
                            }
                        } else {
                            send_commands(&mut framed, vec!["-ERR Not authenticated".to_string()]).await?;
                        }
                    }
                    "RETR" => {
                        if let Some(ref user) = authenticated_user {
                            if parts.len() < 2 {
                                send_commands(&mut framed, vec!["-ERR Message number required".to_string()]).await?;
                            } else if let Ok(msg_id) = parts[1].parse::<usize>() {
                                let messages = mailbox.get_messages(user);
                                if let Some(msg) = messages.iter().find(|m| m.id == msg_id && !m.deleted) {
                                    let mut response = vec![format!("+OK {} octets", msg.size)];
                                    response.extend(msg.content.lines().map(|l| l.to_string()));
                                    response.push(".".to_string());
                                    send_commands(&mut framed, response).await?;
                                } else {
                                    send_commands(&mut framed, vec!["-ERR No such message".to_string()]).await?;
                                }
                            } else {
                                send_commands(&mut framed, vec!["-ERR Invalid message number".to_string()]).await?;
                            }
                        } else {
                            send_commands(&mut framed, vec!["-ERR Not authenticated".to_string()]).await?;
                        }
                    }
                    "DELE" => {
                        if let Some(ref user) = authenticated_user {
                            if parts.len() < 2 {
                                send_commands(&mut framed, vec!["-ERR Message number required".to_string()]).await?;
                            } else if let Ok(msg_id) = parts[1].parse::<usize>() {
                                if mailbox.mark_deleted(user, msg_id) {
                                    send_commands(&mut framed, vec![format!("+OK Message {} deleted", msg_id)]).await?;
                                } else {
                                    send_commands(&mut framed, vec!["-ERR No such message".to_string()]).await?;
                                }
                            } else {
                                send_commands(&mut framed, vec!["-ERR Invalid message number".to_string()]).await?;
                            }
                        } else {
                            send_commands(&mut framed, vec!["-ERR Not authenticated".to_string()]).await?;
                        }
                    }
                    "NOOP" => {
                        send_commands(&mut framed, vec!["+OK".to_string()]).await?;
                    }
                    "RSET" => {
                        if let Some(ref user) = authenticated_user {
                            mailbox.reset_deleted(user);
                            let (count, size) = mailbox.get_stats(user);
                            send_commands(&mut framed, vec![format!("+OK {} messages ({} octets)", count, size)]).await?;
                        } else {
                            send_commands(&mut framed, vec!["-ERR Not authenticated".to_string()]).await?;
                        }
                    }
                    "QUIT" => {
                        state = Pop3State::Update;
                        break;
                    }
                    "CAPA" => {
                        send_commands(&mut framed, vec![
                            "+OK Capability list follows".to_string(),
                            "USER".to_string(),
                            "UIDL".to_string(),
                            "TOP".to_string(),
                            ".".to_string(),
                        ]).await?;
                    }
                    _ => {
                        send_commands(&mut framed, vec!["-ERR Unknown command".to_string()]).await?;
                    }
                }
            }
            Pop3State::Update => {
                break;
            }
        }
    }
    
    // Update state - delete marked messages
    if state == Pop3State::Update {
        if let Some(ref user) = authenticated_user {
            mailbox.delete_marked(user);
            tracing::info!("POP3 session ended for user: {}", user);
        }
        send_commands(&mut framed, vec!["+OK Goodbye".to_string()]).await?;
    }
    
    Ok(())
}
