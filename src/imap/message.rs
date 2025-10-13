use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageFlag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
}

impl MessageFlag {
    pub fn to_string(&self) -> &str {
        match self {
            MessageFlag::Seen => "\\Seen",
            MessageFlag::Answered => "\\Answered",
            MessageFlag::Flagged => "\\Flagged",
            MessageFlag::Deleted => "\\Deleted",
            MessageFlag::Draft => "\\Draft",
            MessageFlag::Recent => "\\Recent",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "\\SEEN" => Some(MessageFlag::Seen),
            "\\ANSWERED" => Some(MessageFlag::Answered),
            "\\FLAGGED" => Some(MessageFlag::Flagged),
            "\\DELETED" => Some(MessageFlag::Deleted),
            "\\DRAFT" => Some(MessageFlag::Draft),
            "\\RECENT" => Some(MessageFlag::Recent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub uid: u32,
    pub sequence_number: u32,
    pub flags: HashSet<MessageFlag>,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub raw_content: String,
}

impl Message {
    pub fn new(uid: u32, sequence_number: u32, from: String, to: Vec<String>, raw_content: String) -> Self {
        // Parse subject from raw content
        let subject = Self::extract_subject(&raw_content);
        let body = Self::extract_body(&raw_content);
        
        let mut flags = HashSet::new();
        flags.insert(MessageFlag::Recent);
        
        Message {
            uid,
            sequence_number,
            flags,
            from,
            to,
            subject,
            body,
            raw_content,
        }
    }

    fn extract_subject(content: &str) -> String {
        for line in content.lines() {
            if line.to_lowercase().starts_with("subject:") {
                return line[8..].trim().to_string();
            }
        }
        "(No Subject)".to_string()
    }

    fn extract_body(content: &str) -> String {
        let mut in_body = false;
        let mut body = String::new();
        
        for line in content.lines() {
            if in_body {
                body.push_str(line);
                body.push('\n');
            } else if line.trim().is_empty() {
                in_body = true;
            }
        }
        
        body
    }

    pub fn has_flag(&self, flag: &MessageFlag) -> bool {
        self.flags.contains(flag)
    }

    pub fn add_flag(&mut self, flag: MessageFlag) {
        self.flags.insert(flag);
    }

    pub fn remove_flag(&mut self, flag: &MessageFlag) {
        self.flags.remove(flag);
    }

    pub fn flags_string(&self) -> String {
        let flags: Vec<String> = self.flags.iter()
            .map(|f| f.to_string().to_string())
            .collect();
        format!("({})", flags.join(" "))
    }
}
