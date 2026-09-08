//! What each role opens: the `access_control_rules` rows written at the
//! `role` band, joined to the catalog entity they name.
//!
//! `default_included` rides along because it decides what an *absent* rule
//! means — an entity open by default grants everyone whatever no rule denies,
//! so a role card listing only its allows would overstate how closed the
//! entity is.

use sqlx::PgPool;

use crate::types::access_control::AccessDecision;

/// One entity one role reaches, or is denied.
#[derive(Debug, Clone)]
pub struct RoleEntitlementRow {
    pub role: String,
    pub entity_type: String,
    pub entity_id: String,
    pub access: AccessDecision,
    pub default_included: bool,
}

pub async fn list_role_entitlements(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RoleEntitlementRow>, sqlx::Error> {
    sqlx::query_as!(
        RoleEntitlementRow,
        r#"SELECT
                r.rule_value AS "role!",
                r.entity_type AS "entity_type!",
                r.entity_id AS "entity_id!",
                r.access AS "access!: AccessDecision",
                COALESCE(e.default_included, false) AS "default_included!"
           FROM access_control_rules r
           LEFT JOIN access_control_entities e
                  ON e.entity_type = r.entity_type AND e.entity_id = r.entity_id
           WHERE r.rule_type = 'role'
           ORDER BY r.rule_value, r.entity_type, r.entity_id
           LIMIT $1"#,
        limit,
    )
    .fetch_all(pool)
    .await
}
