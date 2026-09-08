//! The whole rule ledger, joined to the catalog entity each rule names.
//!
//! Read whole rather than paged in SQL: the filters and the sort the page
//! offers are over four columns, and a rule set that outgrows [`RULE_CAP`]
//! is a configuration to fix rather than a page to scroll. The cap is
//! reported to the caller so the page can say the listing is short.

use sqlx::PgPool;

use crate::types::access_control::AccessDecision;

// Why: Every rule the page will ever render in one read.
pub const RULE_CAP: i64 = 1000;

/// One `access_control_rules` row with the entity context it needs.
#[derive(Debug, Clone)]
pub struct LedgerRuleRow {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub rule_type: String,
    pub rule_value: String,
    pub access: AccessDecision,
    pub justification: Option<String>,
    pub default_included: bool,
    pub entity_source: Option<String>,
}

pub async fn list_ledger_rules(pool: &PgPool) -> Result<Vec<LedgerRuleRow>, sqlx::Error> {
    sqlx::query_as!(
        LedgerRuleRow,
        r#"SELECT
                r.id AS "id!",
                r.entity_type AS "entity_type!",
                r.entity_id AS "entity_id!",
                r.rule_type AS "rule_type!",
                r.rule_value AS "rule_value!",
                r.access AS "access!: AccessDecision",
                r.justification,
                COALESCE(e.default_included, false) AS "default_included!",
                e.source AS "entity_source?"
           FROM access_control_rules r
           LEFT JOIN access_control_entities e
                  ON e.entity_type = r.entity_type AND e.entity_id = r.entity_id
           ORDER BY r.entity_type, r.entity_id, r.rule_type, r.rule_value
           LIMIT $1"#,
        RULE_CAP,
    )
    .fetch_all(pool)
    .await
}

// Why: Entities that grant everyone whatever no rule denies.
pub async fn count_open_entities(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
           FROM access_control_entities
           WHERE default_included = true"#,
    )
    .fetch_one(pool)
    .await
}
