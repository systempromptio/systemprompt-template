use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use super::super::constructors::truncate;
use super::super::enums::{ActivityAction, ActivityCategory};
use super::super::types::NewActivity;

/// Shared shape for events that only carry the session id.
#[derive(Debug, Serialize)]
struct SessionMeta<'a> {
    session_id: &'a str,
}

impl NewActivity {
    #[must_use]
    pub fn notification(
        user_id: &UserId,
        session_id: &SessionId,
        ntype: Option<&str>,
        message: Option<&str>,
    ) -> Self {
        let description = match (ntype, message) {
            (Some("permission_prompt"), Some(msg)) => {
                format!("Permission prompt: {}", truncate(msg, 80))
            },
            (Some(t), Some(msg)) => format!("{t}: {}", truncate(msg, 80)),
            (Some(t), None) => t.to_owned(),
            (None, Some(msg)) => truncate(msg, 80),
            (None, None) => "Notification received".to_owned(),
        };
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::Notification,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }
}
