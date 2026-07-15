use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use super::super::constructors::truncate;
use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};

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

    #[must_use]
    pub fn task_completed_activity(
        user_id: &UserId,
        session_id: &SessionId,
        subject: Option<&str>,
    ) -> Self {
        let description = subject.map_or_else(
            || "Completed a task".to_owned(),
            |s| format!("Completed task: '{s}'"),
        );
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::TaskCompletion,
            action: ActivityAction::Ended,
            entity: None,
            description,
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn context_compacted(
        user_id: &UserId,
        session_id: &SessionId,
        trigger: Option<&str>,
    ) -> Self {
        let description = if trigger == Some("auto") {
            "Context auto-compacted".to_owned()
        } else {
            "Context manually compacted".to_owned()
        };
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::Compaction,
            action: ActivityAction::Used,
            entity: None,
            description,
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn permission_requested(user_id: &UserId, session_id: &SessionId, tool: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::Notification,
            action: ActivityAction::Submitted,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: Some(session_id.as_str().to_owned()),
                name: Some(tool.to_owned()),
            }),
            description: format!("Permission requested for {tool}"),
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }
}
