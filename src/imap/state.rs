use super::mailbox::Mailbox;

#[derive(Debug, Clone)]
pub enum ImapState {
    NotAuthenticated,
    Authenticated,
    Selected(Mailbox),
    Logout,
}

impl PartialEq for ImapState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ImapState::NotAuthenticated, ImapState::NotAuthenticated) => true,
            (ImapState::Authenticated, ImapState::Authenticated) => true,
            (ImapState::Logout, ImapState::Logout) => true,
            (ImapState::Selected(_), ImapState::Selected(_)) => true,
            _ => false,
        }
    }
}

impl ImapState {
    pub fn is_authenticated(&self) -> bool {
        matches!(self, ImapState::Authenticated | ImapState::Selected(_))
    }

    pub fn is_selected(&self) -> bool {
        matches!(self, ImapState::Selected(_))
    }

    pub fn get_selected_mailbox(&self) -> Option<&Mailbox> {
        if let ImapState::Selected(mailbox) = self {
            Some(mailbox)
        } else {
            None
        }
    }
}
