//! Which reply is a session's latest, as far as the transcript has shown.
//! Message ids sort in creation order, so the greatest id seen per session is
//! its latest reply.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::collections::HashMap;

use harness_ids::{MessageId, SessionId};

/// The greatest reply id seen per session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LatestReplies {
    sessions: HashMap<SessionId, MessageId>,
}

impl LatestReplies {
    pub fn observe(&mut self, session: &SessionId, reply: &MessageId) -> bool {
        match self.sessions.get_mut(session) {
            Some(latest) if *latest > *reply => false,
            Some(latest) => {
                if *latest != *reply {
                    latest.clone_from(reply);
                }
                true
            },
            None => {
                self.sessions.insert(session.clone(), reply.clone());
                true
            },
        }
    }
}
