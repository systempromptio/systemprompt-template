use super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::types::{ActivityEntityRef, NewActivity};
use super::constructors::truncate;

impl NewActivity {
    #[must_use]
    pub fn login(user_id: &str, display_name: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Login,
            action: ActivityAction::LoggedIn,
            entity: None,
            description: format!("{display_name} logged in"),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn session_started(
        user_id: &str,
        session_id: &str,
        model: &str,
        project_path: Option<&str>,
    ) -> Self {
        let mut meta = serde_json::json!({ "session_id": session_id, "model": model });
        if let Some(p) = project_path {
            meta["project_path"] = serde_json::json!(p);
        }
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Session,
                entity_id: Some(session_id.to_string()),
                entity_name: None,
            }),
            description: format!("Started a session ({model})"),
            metadata: meta,
        }
    }

    #[must_use]
    pub fn session_ended(user_id: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Session,
                entity_id: Some(session_id.to_string()),
                entity_name: None,
            }),
            description: "Ended a session".to_string(),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn prompt_submitted(user_id: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Prompt,
            action: ActivityAction::Submitted,
            entity: None,
            description: "Sent a prompt".to_string(),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn prompt_submitted_rich(
        user_id: &str,
        session_id: &str,
        prompt_preview: Option<&str>,
    ) -> Self {
        let description = match prompt_preview {
            Some(text) => format!("Asked: \"{}\"", truncate(text, 80)),
            None => "Sent a prompt".to_string(),
        };
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Prompt,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
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
            format!("Started session on {} ({model})", super::constructors::extract_project_name(path))
        } else {
            format!("Started a session ({model})")
        };
        let mut meta = serde_json::json!({ "session_id": session_id, "model": model });
        if let Some(p) = project_path {
            meta["project_path"] = serde_json::json!(p);
        }
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Session,
                entity_id: Some(session_id.to_string()),
                entity_name: None,
            }),
            description,
            metadata: meta,
        }
    }

    #[must_use]
    pub fn session_ended_rich(
        user_id: &str,
        session_id: &str,
        reason: Option<&str>,
    ) -> Self {
        let description = match reason {
            Some(r) => format!("Ended a session ({r})"),
            None => "Ended a session".to_string(),
        };
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Session,
                entity_id: Some(session_id.to_string()),
                entity_name: None,
            }),
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn agent_response(
        user_id: &str,
        session_id: &str,
        message_preview: Option<&str>,
    ) -> Self {
        let description = match message_preview {
            Some(msg) => format!("Claude responded: \"{}\"", truncate(msg, 80)),
            None => "Claude finished responding".to_string(),
        };
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::AgentResponse,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn subagent_started(
        user_id: &str,
        session_id: &str,
        agent_type: Option<&str>,
    ) -> Self {
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Started,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Agent,
                entity_id: Some(session_id.to_string()),
                entity_name: agent_type.map(std::string::ToString::to_string),
            }),
            description: format!("Spawned {} agent", agent_type.unwrap_or("unknown")),
            metadata: serde_json::json!({ "session_id": session_id }),
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
        let description = match msg {
            Some(m) => format!("{agent} agent stopped: {}", truncate(m, 60)),
            None => format!("{agent} agent stopped"),
        };
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Agent,
                entity_id: Some(session_id.to_string()),
                entity_name: agent_type.map(std::string::ToString::to_string),
            }),
            description,
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn teammate_idle(
        user_id: &str,
        session_id: &str,
        name: Option<&str>,
    ) -> Self {
        let who = name.unwrap_or("unknown");
        Self {
            user_id: user_id.to_string(),
            category: ActivityCategory::Session,
            action: ActivityAction::Ended,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Agent,
                entity_id: Some(session_id.to_string()),
                entity_name: name.map(std::string::ToString::to_string),
            }),
            description: format!("Teammate {who} went idle"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }
}
