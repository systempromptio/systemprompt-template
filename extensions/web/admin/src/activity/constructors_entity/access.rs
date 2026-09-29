//! Activity constructors for one access-control rule written or removed from
//! an entity's "Who gets this" panel: which entity, which subject, and why.

use systemprompt::identifiers::UserId;

use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};

/// One rule change as the trail records it.
#[derive(Debug, Clone, Copy)]
pub struct RuleChange<'a> {
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub subject: &'a str,
    pub access: Option<&'a str>,
    pub reason: Option<&'a str>,
}

// Why: an entity kind the activity table has no word for is filed under
// the access-control plane, so the row still reads as an access change.
fn entity_ref(entity_type: &str, entity_id: &str) -> ActivityEntityRef {
    let kind = match entity_type {
        "marketplace" => ActivityEntity::Marketplace,
        "plugin" => ActivityEntity::Plugin,
        "skill" => ActivityEntity::Skill,
        "mcp_server" => ActivityEntity::McpServer,
        "gateway_route" => ActivityEntity::GatewayRoute,
        _ => ActivityEntity::Sync,
    };
    ActivityEntityRef {
        kind,
        id: Some(entity_id.to_owned()),
        name: Some(format!("{entity_type}/{entity_id}")),
    }
}

impl NewActivity {
    #[must_use]
    pub fn access_rule_changed(user_id: &UserId, change: RuleChange<'_>) -> Self {
        let RuleChange {
            entity_type,
            entity_id,
            subject,
            access,
            reason,
        } = change;
        let because = reason.map_or_else(String::new, |r| format!(" — {r}"));
        let (action, verb) = access.map_or_else(
            || {
                (
                    ActivityAction::Deleted,
                    format!("Removed the rule for {subject}"),
                )
            },
            |a| (ActivityAction::Updated, format!("Set {a} for {subject}")),
        );
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action,
            entity: Some(entity_ref(entity_type, entity_id)),
            description: format!("{verb} on {entity_type}/{entity_id}{because}"),
            // JSON: JSONB column — the rule key and the stated reason
            metadata: serde_json::json!({
                "entity_type": entity_type,
                "entity_id": entity_id,
                "subject": subject,
                "access": access,
                "reason": reason,
            }),
        }
    }
}
