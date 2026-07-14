use serde::Serialize;

use super::constructors::truncate;
use super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::types::{ActivityEntityRef, NewActivity};

/// Empty metadata payload `{}`, matching the prior `json!({})`.
fn empty_meta() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

/// Shared shape for events that only carry the session id.
#[derive(Debug, Serialize)]
struct SessionMeta<'a> {
    session_id: &'a str,
}

#[derive(Debug, Serialize)]
struct SessionStartedMeta<'a> {
    session_id: &'a str,
    model: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_path: Option<&'a str>,
}

impl NewActivity {
    #[must_use]
    pub fn login(user_id: &str, display_name: &str) -> Self {
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Login,
            action: ActivityAction::LoggedIn,
            entity: None,
            description: format!("{display_name} logged in"),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn session_started(
        user_id: &str,
        session_id: &str,
        model: &str,
        project_path: Option<&str>,
    ) -> Self {
        let meta = serde_json::to_value(SessionStartedMeta {
            session_id,
            model,
            project_path,
        })
        .unwrap_or_default();
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Session,
                id: Some(session_id.to_owned()),
                name: None,
            }),
            description: format!("Started a session ({model})"),
            metadata: meta,
        }
    }

    #[must_use]
    pub fn session_ended(user_id: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Session,
                id: Some(session_id.to_owned()),
                name: None,
            }),
            description: "Ended a session".to_owned(),
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn prompt_submitted(user_id: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Prompt,
            action: ActivityAction::Submitted,
            entity: None,
            description: "Sent a prompt".to_owned(),
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn prompt_submitted_rich(
        user_id: &str,
        session_id: &str,
        prompt_preview: Option<&str>,
    ) -> Self {
        let description = prompt_preview.map_or_else(
            || "Sent a prompt".to_owned(),
            |text| format!("Asked: \"{}\"", truncate(text, 80)),
        );
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Prompt,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn session_started_rich(
        user_id: &str,
        session_id: &str,
        model: &str,
        project_path: Option<&str>,
        source: Option<&str>,
    ) -> Self {
        let description = if source == Some("resume") {
            format!("Resumed session ({model})")
        } else if let Some(path) = project_path {
            format!(
                "Started session on {} ({model})",
                super::constructors::extract_project_name(path)
            )
        } else {
            format!("Started a session ({model})")
        };
        let meta = serde_json::to_value(SessionStartedMeta {
            session_id,
            model,
            project_path,
        })
        .unwrap_or_default();
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Session,
                id: Some(session_id.to_owned()),
                name: None,
            }),
            description,
            metadata: meta,
        }
    }

    #[must_use]
    pub fn session_ended_rich(user_id: &str, session_id: &str, reason: Option<&str>) -> Self {
        let description = reason.map_or_else(
            || "Ended a session".to_owned(),
            |r| format!("Ended a session ({r})"),
        );
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Session,
                id: Some(session_id.to_owned()),
                name: None,
            }),
            description,
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn agent_response(user_id: &str, session_id: &str, message_preview: Option<&str>) -> Self {
        let description = message_preview.map_or_else(
            || "Claude finished responding".to_owned(),
            |msg| format!("Claude responded: \"{}\"", truncate(msg, 80)),
        );
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::AgentResponse,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn subagent_started(user_id: &str, session_id: &str, agent_type: Option<&str>) -> Self {
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Agent,
                id: Some(session_id.to_owned()),
                name: agent_type.map(str::to_owned),
            }),
            description: format!("Spawned {} agent", agent_type.unwrap_or("unknown")),
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn subagent_stopped(
        user_id: &str,
        session_id: &str,
        agent_type: Option<&str>,
        msg: Option<&str>,
    ) -> Self {
        let agent = agent_type.unwrap_or("unknown");
        let description = msg.map_or_else(
            || format!("{agent} agent stopped"),
            |m| format!("{agent} agent stopped: {}", truncate(m, 60)),
        );
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Agent,
                id: Some(session_id.to_owned()),
                name: agent_type.map(str::to_owned),
            }),
            description,
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn teammate_idle(user_id: &str, session_id: &str, name: Option<&str>) -> Self {
        let who = name.unwrap_or("unknown");
        Self {
            user_id: user_id.to_owned(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Agent,
                id: Some(session_id.to_owned()),
                name: name.map(str::to_owned),
            }),
            description: format!("Teammate {who} went idle"),
            metadata: serde_json::to_value(SessionMeta { session_id }).unwrap_or_default(),
        }
    }
}
