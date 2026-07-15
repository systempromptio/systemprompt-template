use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use super::super::constructors::truncate;
use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity, entity_label};
use super::super::types::{ActivityEntityRef, NewActivity};

/// Empty metadata payload `{}`, matching the prior `json!({})`.
fn empty_meta() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

/// Shared shape for events that only carry the session id.
#[derive(Debug, Serialize)]
struct SessionMeta<'a> {
    session_id: &'a str,
}

impl NewActivity {
    #[must_use]
    pub fn skill_used(user_id: &UserId, tool_name: &str, session_id: &SessionId) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::SkillUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Skill,
                id: Some(session_id.as_str().to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("Used skill '{tool_name}'"),
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn entity_created(user_id: &UserId, entity: ActivityEntity, id: &str, name: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Created,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Created {} '{name}'", entity_label(entity)),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn entity_updated(user_id: &UserId, entity: ActivityEntity, id: &str, name: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Updated,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Updated {} '{name}'", entity_label(entity)),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn entity_deleted(user_id: &UserId, entity: ActivityEntity, id: &str, name: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Deleted,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Deleted a {}", entity_label(entity)),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn entity_forked(user_id: &UserId, entity: ActivityEntity, id: &str, name: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Created,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Forked {} '{name}'", entity_label(entity)),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn entity_imported(
        user_id: &UserId,
        entity: ActivityEntity,
        id: &str,
        name: &str,
        description: &str,
    ) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Imported,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: description.to_owned(),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn marketplace_uploaded(user_id: &UserId, version: i32) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceConnect,
            action: ActivityAction::Uploaded,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Marketplace,
                id: None,
                name: Some(format!("v{version}")),
            }),
            description: format!("Uploaded marketplace v{version}"),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn marketplace_restored(user_id: &UserId, version: i32) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::MarketplaceConnect,
            action: ActivityAction::Restored,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Marketplace,
                id: None,
                name: Some(format!("v{version}")),
            }),
            description: format!("Restored marketplace to v{version}"),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn tool_used(
        user_id: &UserId,
        tool_name: &str,
        session_id: &SessionId,
        detail: &str,
    ) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::ToolUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: Some(session_id.as_str().to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("{tool_name}: {detail}"),
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn skill_used_rich(
        user_id: &UserId,
        tool_name: &str,
        session_id: &SessionId,
        detail: &str,
    ) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::SkillUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Skill,
                id: Some(session_id.as_str().to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("Used {tool_name}: {detail}"),
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn tool_error(
        user_id: &UserId,
        tool_name: &str,
        session_id: &SessionId,
        error: Option<&str>,
    ) -> Self {
        let msg = truncate(error.unwrap_or("unknown error"), 60);
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::Error,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: Some(session_id.as_str().to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("{tool_name} failed: {msg}"),
            metadata: serde_json::to_value(SessionMeta {
                session_id: session_id.as_str(),
            })
            .unwrap_or_default(),
        }
    }
}
