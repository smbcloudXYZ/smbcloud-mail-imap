use super::commands::{ImapCommand, ImapCommandHandler, ImapResponse};
use super::mailbox::MailboxStore;
use super::state::ImapState;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LinesCodec};
use futures::StreamExt;
use futures::SinkExt;

pub struct ImapSession {
    state: ImapState,
    handler: ImapCommandHandler,
}

impl ImapSession {
    pub fn new(store: MailboxStore) -> Self {
        ImapSession {
            state: ImapState::NotAuthenticated,
            handler: ImapCommandHandler::new(store),
        }
    }

    pub async fn handle<S>(&mut self, stream: S) -> anyhow::Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let mut framed = Framed::new(stream, LinesCodec::new());

        // Send greeting
        framed.send("* OK IMAP4rev1 Server Ready".to_string()).await?;

        while let Some(line_result) = framed.next().await {
            let line = line_result?;
            
            tracing::debug!("IMAP received: {}", line);

            // Parse command
            let cmd = match ImapCommand::parse(&line) {
                Ok(cmd) => cmd,
                Err(e) => {
                    let response = ImapResponse::bad("*".to_string(), format!("Parse error: {}", e));
                    framed.send(response.format().trim_end().to_string()).await?;
                    continue;
                }
            };

            // Handle command
            let responses = self.handle_command(&cmd);

            // Send responses
            for response in responses {
                let formatted = response.format();
                tracing::debug!("IMAP sending: {}", formatted.trim_end());
                framed.send(formatted.trim_end().to_string()).await?;
            }

            // Check if we should logout
            if self.state == ImapState::Logout {
                break;
            }
        }

        Ok(())
    }

    fn handle_command(&mut self, cmd: &ImapCommand) -> Vec<ImapResponse> {
        match cmd.command.as_str() {
            "CAPABILITY" => self.handler.handle_capability(cmd),
            "LOGIN" => self.handler.handle_login(cmd, &mut self.state),
            "LIST" | "LSUB" => self.handler.handle_list(cmd, &self.state),
            "SELECT" => self.handler.handle_select(cmd, &mut self.state, false),
            "EXAMINE" => self.handler.handle_examine(cmd, &mut self.state),
            "FETCH" => self.handler.handle_fetch(cmd, &self.state),
            "SEARCH" => self.handler.handle_search(cmd, &self.state),
            "STATUS" => self.handler.handle_status(cmd, &self.state),
            "LOGOUT" => self.handler.handle_logout(cmd, &mut self.state),
            "NOOP" => self.handler.handle_noop(cmd),
            _ => vec![ImapResponse::bad(
                cmd.tag.clone(),
                format!("Unknown command: {}", cmd.command),
            )],
        }
    }
}
