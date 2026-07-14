use super::super::constructors::truncate;
use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};

impl NewActivity {
    #[must_use]
    pub fn notification(
        user_id: impl AsRef<str>,
        session_id: &str,
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
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::Notification,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn task_completed_activity(
        user_id: impl AsRef<str>,
        session_id: &str,
        subject: Option<&str>,
    ) -> Self {
        let description = subject.map_or_else(
            || "Completed a task".to_owned(),
            |s| format!("Completed task: '{s}'"),
        );
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::TaskCompletion,
            action: ActivityAction::Ended,
            entity: None,
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn context_compacted(
        user_id: impl AsRef<str>,
        session_id: &str,
        trigger: Option<&str>,
    ) -> Self {
        let description = if trigger == Some("auto") {
            "Context auto-compacted".to_owned()
        } else {
            "Context manually compacted".to_owned()
        };
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::Compaction,
            action: ActivityAction::Used,
            entity: None,
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn permission_requested(user_id: impl AsRef<str>, session_id: &str, tool: &str) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::Notification,
            action: ActivityAction::Submitted,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: Some(session_id.to_owned()),
                name: Some(tool.to_owned()),
            }),
            description: format!("Permission requested for {tool}"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }
}
