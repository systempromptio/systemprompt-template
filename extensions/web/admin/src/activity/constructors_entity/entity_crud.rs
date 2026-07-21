use systemprompt::identifiers::UserId;

use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity, entity_label};
use super::super::types::{ActivityEntityRef, NewActivity};

/// Empty metadata payload `{}`, matching the prior `json!({})`.
fn empty_meta() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

impl NewActivity {
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
}
