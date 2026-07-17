//! DB -> YAML snapshot of role access-control rules.
//!
//! Inverse of [`super::acl_yaml_loader`]: collapses joined
//! `access_control_rules` and `access_control_entities` rows back into the
//! flat YAML shape that bootstrap consumes. Output is sorted deterministically
//! so dashboard exports diff cleanly against the source-of-truth YAML.

use std::collections::BTreeMap;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt_security::authz::{Access, EntityKind, RuleType};
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Serialize)]
struct EntityKey {
    entity_type: EntityKind,
    entity_id: String,
    access: Access,
    default_included: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    roles: Vec<String>,
}

#[derive(Serialize)]
struct Snapshot {
    rules: Vec<EntityKey>,
}

pub async fn render_yaml_snapshot(pool: &PgPool) -> Result<String, MarketplaceError> {
    let rows = sqlx::query!(
        r#"SELECT r.entity_type,
                  r.entity_id,
                  r.rule_type as "rule_type: RuleType",
                  r.rule_value,
                  r.access as "access: Access",
                  COALESCE(e.default_included, false) as "default_included!"
           FROM access_control_rules r
           LEFT JOIN access_control_entities e
              ON e.entity_type = r.entity_type AND e.entity_id = r.entity_id
           WHERE r.rule_type = 'role'
           ORDER BY r.entity_type, r.entity_id, r.access, r.rule_type, r.rule_value"#,
    )
    .fetch_all(pool)
    .await?;

    let mut by_key: BTreeMap<(String, String, String), EntityKey> = BTreeMap::new();
    for row in rows {
        let entity_type: EntityKind = row.entity_type.parse().map_err(|e| {
            MarketplaceError::Internal(format!("unknown entity_type in DB row: {e}"))
        })?;
        let key = (
            entity_type.as_str().to_owned(),
            row.entity_id.clone(),
            row.access.to_string(),
        );
        let entry = by_key.entry(key).or_insert_with(|| EntityKey {
            entity_type,
            entity_id: row.entity_id,
            access: row.access,
            default_included: row.default_included,
            roles: Vec::new(),
        });
        match row.rule_type {
            RuleType::Role => entry.roles.push(row.rule_value),
            RuleType::User => {},
        }
    }

    let snap = Snapshot {
        rules: by_key.into_values().collect(),
    };
    serde_yaml::to_string(&snap).map_err(MarketplaceError::Yaml)
}
