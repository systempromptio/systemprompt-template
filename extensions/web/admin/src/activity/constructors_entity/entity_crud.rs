use super::super::constructors::truncate;
use super::super::enums::{entity_label, ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};

impl NewActivity {
    #[must_use]
    pub fn skill_used(user_id: impl AsRef<str>, tool_name: &str, session_id: &str) -> Self {
        Self {
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::SkillUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Skill,
                entity_id: Some(session_id.to_string()),
                entity_name: Some(tool_name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Created,
            entity: Some(ActivityEntityRef {
                entity_type: entity,
                entity_id: Some(id.to_string()),
                entity_name: Some(name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Updated,
            entity: Some(ActivityEntityRef {
                entity_type: entity,
                entity_id: Some(id.to_string()),
                entity_name: Some(name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Deleted,
            entity: Some(ActivityEntityRef {
                entity_type: entity,
                entity_id: Some(id.to_string()),
                entity_name: Some(name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Created,
            entity: Some(ActivityEntityRef {
                entity_type: entity,
                entity_id: Some(id.to_string()),
                entity_name: Some(name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceEdit,
            action: ActivityAction::Imported,
            entity: Some(ActivityEntityRef {
                entity_type: entity,
                entity_id: Some(id.to_string()),
                entity_name: Some(name.to_string()),
            }),
            description: description.to_string(),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn marketplace_uploaded(user_id: impl AsRef<str>, version: i32) -> Self {
        Self {
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceConnect,
            action: ActivityAction::Uploaded,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Marketplace,
                entity_id: None,
                entity_name: Some(format!("v{version}")),
            }),
            description: format!("Uploaded marketplace v{version}"),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn marketplace_restored(user_id: impl AsRef<str>, version: i32) -> Self {
        Self {
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::MarketplaceConnect,
            action: ActivityAction::Restored,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Marketplace,
                entity_id: None,
                entity_name: Some(format!("v{version}")),
            }),
            description: format!("Restored marketplace to v{version}"),
            metadata: serde_json::json!({}),
        }
    }

    #[must_use]
    pub fn user_skill_imported(
        user_id: impl AsRef<str>,
        bundle_id: &str,
        imported_count: u32,
    ) -> Self {
        let uid = user_id.as_ref();
        Self {
            user_id: uid.to_string(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::UserSkill,
                entity_id: Some(bundle_id.to_string()),
                entity_name: Some(bundle_id.to_string()),
            }),
            description: format!("Imported {imported_count} skills for user '{uid}'"),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::ToolUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Tool,
                entity_id: Some(session_id.to_string()),
                entity_name: Some(tool_name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::SkillUsage,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Skill,
                entity_id: Some(session_id.to_string()),
                entity_name: Some(tool_name.to_string()),
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
            user_id: user_id.as_ref().to_string(),
            category: ActivityCategory::Error,
            action: ActivityAction::Used,
            entity: Some(ActivityEntityRef {
                entity_type: ActivityEntity::Tool,
                entity_id: Some(session_id.to_string()),
                entity_name: Some(tool_name.to_string()),
            }),
            description: format!("{tool_name} failed: {msg}"),
            metadata: serde_json::json!({ "session_id": session_id }),
        }
    }
}
