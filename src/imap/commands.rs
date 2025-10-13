use super::mailbox::MailboxStore;
use super::state::ImapState;

pub struct ImapCommand {
    pub tag: String,
    pub command: String,
    pub args: Vec<String>,
}

impl ImapCommand {
    pub fn parse(line: &str) -> Result<Self, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 2 {
            return Err("Invalid command format".to_string());
        }

        let tag = parts[0].to_string();
        let command = parts[1].to_uppercase();
        let args = parts[2..].iter().map(|s| s.to_string()).collect();

        Ok(ImapCommand { tag, command, args })
    }
}

pub struct ImapResponse {
    pub tag: Option<String>,
    pub status: String,
    pub text: String,
}

impl ImapResponse {
    pub fn ok(tag: String, text: String) -> Self {
        ImapResponse {
            tag: Some(tag),
            status: "OK".to_string(),
            text,
        }
    }

    pub fn no(tag: String, text: String) -> Self {
        ImapResponse {
            tag: Some(tag),
            status: "NO".to_string(),
            text,
        }
    }

    pub fn bad(tag: String, text: String) -> Self {
        ImapResponse {
            tag: Some(tag),
            status: "BAD".to_string(),
            text,
        }
    }

    pub fn untagged(text: String) -> Self {
        ImapResponse {
            tag: None,
            status: "*".to_string(),
            text,
        }
    }

    pub fn format(&self) -> String {
        if let Some(ref tag) = self.tag {
            format!("{} {} {}\r\n", tag, self.status, self.text)
        } else {
            format!("{} {}\r\n", self.status, self.text)
        }
    }
}

pub struct ImapCommandHandler {
    pub store: MailboxStore,
}

impl ImapCommandHandler {
    pub fn new(store: MailboxStore) -> Self {
        ImapCommandHandler { store }
    }

    pub fn handle_capability(&self, cmd: &ImapCommand) -> Vec<ImapResponse> {
        vec![
            ImapResponse::untagged("CAPABILITY IMAP4rev1 AUTH=PLAIN".to_string()),
            ImapResponse::ok(cmd.tag.clone(), "CAPABILITY completed".to_string()),
        ]
    }

    pub fn handle_login(&self, cmd: &ImapCommand, state: &mut ImapState) -> Vec<ImapResponse> {
        if cmd.args.len() < 2 {
            return vec![ImapResponse::bad(cmd.tag.clone(), "Invalid LOGIN command".to_string())];
        }

        // Simple authentication - accept any username/password for demo
        *state = ImapState::Authenticated;
        
        vec![ImapResponse::ok(cmd.tag.clone(), "LOGIN completed".to_string())]
    }

    pub fn handle_list(&self, cmd: &ImapCommand, state: &ImapState) -> Vec<ImapResponse> {
        if !state.is_authenticated() {
            return vec![ImapResponse::no(cmd.tag.clone(), "Not authenticated".to_string())];
        }

        let mut responses = Vec::new();
        let mailboxes = self.store.list_mailboxes();
        
        for mailbox in mailboxes {
            responses.push(ImapResponse::untagged(
                format!("LIST () \"/\" \"{}\"", mailbox)
            ));
        }
        
        responses.push(ImapResponse::ok(cmd.tag.clone(), "LIST completed".to_string()));
        responses
    }

    pub fn handle_select(&self, cmd: &ImapCommand, state: &mut ImapState, readonly: bool) -> Vec<ImapResponse> {
        if !state.is_authenticated() {
            return vec![ImapResponse::no(cmd.tag.clone(), "Not authenticated".to_string())];
        }

        if cmd.args.is_empty() {
            return vec![ImapResponse::bad(cmd.tag.clone(), "No mailbox specified".to_string())];
        }

        let mailbox_name = &cmd.args[0];
        
        if let Some(mailbox) = self.store.get_mailbox(mailbox_name) {
            let mut responses = Vec::new();
            
            responses.push(ImapResponse::untagged(format!("{} EXISTS", mailbox.exists())));
            responses.push(ImapResponse::untagged(format!("{} RECENT", mailbox.recent())));
            responses.push(ImapResponse::untagged(
                format!("OK [UNSEEN {}] Message {} is first unseen", mailbox.unseen(), mailbox.unseen())
            ));
            responses.push(ImapResponse::untagged(
                format!("OK [UIDVALIDITY {}] UIDs valid", mailbox.uidvalidity())
            ));
            responses.push(ImapResponse::untagged(
                format!("OK [UIDNEXT {}] Predicted next UID", mailbox.uidnext())
            ));
            responses.push(ImapResponse::untagged(
                "FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)".to_string()
            ));
            
            *state = ImapState::Selected(mailbox.clone());
            
            let mode = if readonly { "[READ-ONLY]" } else { "[READ-WRITE]" };
            responses.push(ImapResponse::ok(
                cmd.tag.clone(),
                format!("{} SELECT completed", mode)
            ));
            
            responses
        } else {
            vec![ImapResponse::no(cmd.tag.clone(), "Mailbox does not exist".to_string())]
        }
    }

    pub fn handle_examine(&self, cmd: &ImapCommand, state: &mut ImapState) -> Vec<ImapResponse> {
        self.handle_select(cmd, state, true)
    }

    pub fn handle_fetch(&self, cmd: &ImapCommand, state: &ImapState) -> Vec<ImapResponse> {
        if !state.is_selected() {
            return vec![ImapResponse::no(cmd.tag.clone(), "No mailbox selected".to_string())];
        }

        if cmd.args.len() < 2 {
            return vec![ImapResponse::bad(cmd.tag.clone(), "Invalid FETCH command".to_string())];
        }

        let mailbox = state.get_selected_mailbox().unwrap();
        let sequence_set = &cmd.args[0];
        let data_items = cmd.args[1..].join(" ");

        let mut responses = Vec::new();
        let messages = mailbox.get_messages();

        // Parse sequence set (simplified - only handles single numbers or ranges)
        let sequences = parse_sequence_set(sequence_set, messages.len() as u32);

        for seq in sequences {
            if let Some(message) = mailbox.get_message_by_sequence(seq) {
                let fetch_data = build_fetch_response(&message, &data_items);
                responses.push(ImapResponse::untagged(
                    format!("{} FETCH {}", seq, fetch_data)
                ));
            }
        }

        responses.push(ImapResponse::ok(cmd.tag.clone(), "FETCH completed".to_string()));
        responses
    }

    pub fn handle_search(&self, cmd: &ImapCommand, state: &ImapState) -> Vec<ImapResponse> {
        if !state.is_selected() {
            return vec![ImapResponse::no(cmd.tag.clone(), "No mailbox selected".to_string())];
        }

        let mailbox = state.get_selected_mailbox().unwrap();
        let messages = mailbox.get_messages();

        // Simplified SEARCH - for demo, return all messages
        let sequence_numbers: Vec<String> = messages.iter()
            .map(|m| m.sequence_number.to_string())
            .collect();

        let mut responses = Vec::new();
        responses.push(ImapResponse::untagged(
            format!("SEARCH {}", sequence_numbers.join(" "))
        ));
        responses.push(ImapResponse::ok(cmd.tag.clone(), "SEARCH completed".to_string()));
        
        responses
    }

    pub fn handle_status(&self, cmd: &ImapCommand, state: &ImapState) -> Vec<ImapResponse> {
        if !state.is_authenticated() {
            return vec![ImapResponse::no(cmd.tag.clone(), "Not authenticated".to_string())];
        }

        if cmd.args.is_empty() {
            return vec![ImapResponse::bad(cmd.tag.clone(), "No mailbox specified".to_string())];
        }

        let mailbox_name = &cmd.args[0];
        
        if let Some(mailbox) = self.store.get_mailbox(mailbox_name) {
            let status_items = format!(
                "MESSAGES {} RECENT {} UNSEEN {} UIDNEXT {} UIDVALIDITY {}",
                mailbox.exists(),
                mailbox.recent(),
                mailbox.unseen(),
                mailbox.uidnext(),
                mailbox.uidvalidity()
            );

            vec![
                ImapResponse::untagged(format!("STATUS \"{}\" ({})", mailbox_name, status_items)),
                ImapResponse::ok(cmd.tag.clone(), "STATUS completed".to_string()),
            ]
        } else {
            vec![ImapResponse::no(cmd.tag.clone(), "Mailbox does not exist".to_string())]
        }
    }

    pub fn handle_logout(&self, cmd: &ImapCommand, state: &mut ImapState) -> Vec<ImapResponse> {
        *state = ImapState::Logout;
        
        vec![
            ImapResponse::untagged("BYE IMAP4rev1 Server logging out".to_string()),
            ImapResponse::ok(cmd.tag.clone(), "LOGOUT completed".to_string()),
        ]
    }

    pub fn handle_noop(&self, cmd: &ImapCommand) -> Vec<ImapResponse> {
        vec![ImapResponse::ok(cmd.tag.clone(), "NOOP completed".to_string())]
    }
}

fn parse_sequence_set(sequence_set: &str, max: u32) -> Vec<u32> {
    let mut sequences = Vec::new();
    
    // Handle "*" (all messages)
    if sequence_set == "*" {
        sequences.push(max);
        return sequences;
    }

    // Handle ranges like "1:5" or single numbers like "3"
    if sequence_set.contains(':') {
        let parts: Vec<&str> = sequence_set.split(':').collect();
        if parts.len() == 2 {
            let start = if parts[0] == "*" {
                max
            } else {
                parts[0].parse::<u32>().unwrap_or(1)
            };
            
            let end = if parts[1] == "*" {
                max
            } else {
                parts[1].parse::<u32>().unwrap_or(max)
            };

            for i in start..=end.min(max) {
                sequences.push(i);
            }
        }
    } else if let Ok(seq) = sequence_set.parse::<u32>() {
        sequences.push(seq);
    }

    sequences
}

fn build_fetch_response(message: &super::message::Message, data_items: &str) -> String {
    let data_items_upper = data_items.to_uppercase();
    let mut parts = Vec::new();

    if data_items_upper.contains("FLAGS") {
        parts.push(format!("FLAGS {}", message.flags_string()));
    }

    if data_items_upper.contains("UID") {
        parts.push(format!("UID {}", message.uid));
    }

    if data_items_upper.contains("RFC822.SIZE") || data_items_upper.contains("BODY") {
        parts.push(format!("RFC822.SIZE {}", message.raw_content.len()));
    }

    if data_items_upper.contains("BODY[]") || data_items_upper.contains("RFC822") {
        parts.push(format!("BODY[] {{{}}}\r\n{}", message.raw_content.len(), message.raw_content));
    } else if data_items_upper.contains("BODY[HEADER]") {
        let header = extract_header(&message.raw_content);
        parts.push(format!("BODY[HEADER] {{{}}}\r\n{}", header.len(), header));
    } else if data_items_upper.contains("BODY[TEXT]") {
        parts.push(format!("BODY[TEXT] {{{}}}\r\n{}", message.body.len(), message.body));
    }

    if data_items_upper.contains("ENVELOPE") {
        let envelope = build_envelope(message);
        parts.push(format!("ENVELOPE {}", envelope));
    }

    format!("({})", parts.join(" "))
}

fn extract_header(content: &str) -> String {
    let mut header = String::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            break;
        }
        header.push_str(line);
        header.push_str("\r\n");
    }
    header
}

fn build_envelope(message: &super::message::Message) -> String {
    // Simplified envelope structure
    format!(
        "(\"{}\" \"{}\" ((NIL NIL \"{}\" NIL)) ((NIL NIL \"{}\" NIL)) ((NIL NIL \"{}\" NIL)) ((NIL NIL \"{}\" NIL)) NIL NIL NIL NIL)",
        "",  // date
        message.subject,
        message.from,
        message.from,
        message.from,
        message.to.first().unwrap_or(&"".to_string())
    )
}
