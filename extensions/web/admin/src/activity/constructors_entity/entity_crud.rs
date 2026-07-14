use super::super::constructors::truncate;
use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity, entity_label};
use super::super::types::{ActivityEntityRef, NewActivity};

impl NewActivity {
    #[must_use]
    pub fn skill_used(user_id: impl AsRef<str>, tool_name: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::SkillUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Skill,
                id: Some(session_id.to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("Used skill '{tool_name}'"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn entity_created(
        user_id: impl AsRef<str>,
        entity: ActivityEntity,
        id: &str,
        name: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Created,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Created {} '{name}'", entity_label(entity)),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn entity_updated(
        user_id: impl AsRef<str>,
        entity: ActivityEntity,
        id: &str,
        name: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Updated,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Updated {} '{name}'", entity_label(entity)),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn entity_deleted(
        user_id: impl AsRef<str>,
        entity: ActivityEntity,
        id: &str,
        name: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Deleted,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Deleted a {}", entity_label(entity)),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn entity_forked(
        user_id: impl AsRef<str>,
        entity: ActivityEntity,
        id: &str,
        name: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Created,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: format!("Forked {} '{name}'", entity_label(entity)),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn entity_imported(
        user_id: impl AsRef<str>,
        entity: ActivityEntity,
        id: &str,
        name: &str,
        description: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Imported,
            entity: Some(ActivityEntityRef {
                kind: entity,
                id: Some(id.to_owned()),
                name: Some(name.to_owned()),
            }),
            description: description.to_owned(),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn marketplace_uploaded(user_id: impl AsRef<str>, version: i32) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceConnect,
            action: ActivityAction::Uploaded,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Marketplace,
                id: None,
                name: Some(format!("v{version}")),
            }),
            description: format!("Uploaded marketplace v{version}"),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn marketplace_restored(user_id: impl AsRef<str>, version: i32) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::MarketplaceConnect,
            action: ActivityAction::Restored,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Marketplace,
                id: None,
                name: Some(format!("v{version}")),
            }),
            description: format!("Restored marketplace to v{version}"),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn tool_used(
        user_id: impl AsRef<str>,
        tool_name: &str,
        session_id: &str,
        detail: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::ToolUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: Some(session_id.to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("{tool_name}: {detail}"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn skill_used_rich(
        user_id: impl AsRef<str>,
        tool_name: &str,
        session_id: &str,
        detail: &str,
    ) -> Self {
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::SkillUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Skill,
                id: Some(session_id.to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("Used {tool_name}: {detail}"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }

    #[must_use]
    pub fn tool_error(
        user_id: impl AsRef<str>,
        tool_name: &str,
        session_id: &str,
        error: Option<&str>,
    ) -> Self {
        let msg = truncate(error.unwrap_or("unknown error"), 60);
        Self {
            user_id: user_id.as_ref().to_owned(),
            category: ActivityCategory::Error,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Tool,
                id: Some(session_id.to_owned()),
                name: Some(tool_name.to_owned()),
            }),
            description: format!("{tool_name} failed: {msg}"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }
}
