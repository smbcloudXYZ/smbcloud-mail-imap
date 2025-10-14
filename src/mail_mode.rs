/// Enum to distinguish between different SMTP mail handling modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailMode {
    /// Receive mode: for incoming mail (port 2525)
    /// Messages received in this mode are stored in INBOX
    Receive,
    
    /// Submit mode: for outgoing mail submission (port 2526)
    /// Messages received in this mode are logged and relayed
    Submit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_mode_equality() {
        assert_eq!(MailMode::Receive, MailMode::Receive);
        assert_eq!(MailMode::Submit, MailMode::Submit);
        assert_ne!(MailMode::Receive, MailMode::Submit);
    }

    #[test]
    fn test_mail_mode_clone() {
        let mode1 = MailMode::Receive;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
    }
}
