//! Bootstrap loader: `services/access-control/*.yaml` → DB.
//!
//! YAML is the declarative source of truth committed to the source repo.
//! On every server startup the publish pipeline calls [`load_from_yaml`],
//! which upserts entity and rule rows into the runtime database. There is no
//! write-back: dashboard edits live only in the DB of the instance that
//! received them, so deployments sharing the same YAML baseline can drift
//! independently without trampling each other.
//!
//! Two files are read (both optional; missing-file = no-op):
//!
//! - `services/access-control/roles.yaml` — role-scoped allow/deny rules.
//! - `services/access-control/departments.yaml` — department definitions
//!   plus department-scoped allow/deny rules.
//!
//! Wire shapes ([`YamlRule`] etc.) deserialise straight into the typed
//! `EntityKind` / `Access` / `RuleType` enums from `systemprompt_security`,
//! so invalid `entity_type` / `access` values fail at parse time rather than
//! producing a runtime skip in the loader.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_security::authz::{
    Access, AccessControlRepository, EntityKind, RuleType, UpsertRuleParams,
};
use systemprompt_web_shared::error::MarketplaceError;

const ROLES_FILE: &str = "access-control/roles.yaml";
const DEPARTMENTS_FILE: &str = "access-control/departments.yaml";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct YamlRule {
    pub entity_type: EntityKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_match: Option<String>,
    #[serde(default = "default_allow")]
    pub access: Access,
    #[serde(default)]
    pub default_included: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub departments: Vec<String>,
}

impl YamlRule {
    /// Enforce exactly-one of `entity_id` / `entity_match`. Operators can fix
    /// the offending stanza from this message alone — both field names are
    /// surfaced verbatim.
    fn validate_target(&self) -> Result<(), MarketplaceError> {
        match (&self.entity_id, &self.entity_match) {
            (Some(_), Some(_)) => Err(MarketplaceError::Internal(format!(
                "rule for entity_type={} sets both entity_id and entity_match; pick one",
                self.entity_type.as_str()
            ))),
            (None, None) => Err(MarketplaceError::Internal(format!(
                "rule for entity_type={} sets neither entity_id nor entity_match",
                self.entity_type.as_str()
            ))),
            _ => Ok(()),
        }
    }
}

/// Tiny `*`-glob matcher. The only patterns we expect in practice are `"*"`
/// and `"prefix*"` / `"*suffix"`; pulling in `globset` for this is overkill.
fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == candidate;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !candidate.starts_with(first) || !candidate.ends_with(last) {
        return false;
    }
    if first.len() + last.len() > candidate.len() {
        return false;
    }
    let mut cursor = first.len();
    let end = candidate.len() - last.len();
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        match candidate[cursor..end].find(part) {
            Some(pos) => cursor += pos + part.len(),
            None => return false,
        }
    }
    true
}

const fn default_allow() -> Access {
    Access::Allow
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DepartmentsDoc {
    #[serde(default)]
    pub departments: Vec<YamlDepartment>,
    #[serde(default)]
    pub rules: Vec<YamlRule>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RolesDoc {
    #[serde(default)]
    pub rules: Vec<YamlRule>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct YamlDepartment {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadReport {
    pub departments_upserted: usize,
    pub rules_upserted: usize,
    pub rules_skipped: usize,
    pub glob_rules_expanded: usize,
    pub glob_entities_matched: usize,
}

pub async fn load_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<LoadReport, MarketplaceError> {
    let mut report = LoadReport::default();

    load_departments_file(pool, services_path, &mut report).await?;
    load_roles_file(pool, services_path, &mut report).await?;

    tracing::info!(
        departments = report.departments_upserted,
        rules = report.rules_upserted,
        skipped = report.rules_skipped,
        glob_rules = report.glob_rules_expanded,
        glob_entities = report.glob_entities_matched,
        "bootstrap_yaml_loaded"
    );
    Ok(report)
}

async fn read_yaml<T: for<'de> Deserialize<'de> + Default>(
    services_path: &Path,
    rel: &str,
) -> Result<Option<T>, MarketplaceError> {
    let path = services_path.join(rel);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(T::default())),
        Ok(s) => serde_yaml::from_str::<T>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::Internal(format!("{rel}: {e}"))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn load_departments_file(
    pool: &PgPool,
    services_path: &Path,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let Some(doc) = read_yaml::<DepartmentsDoc>(services_path, DEPARTMENTS_FILE).await? else {
        return Ok(());
    };

    for dept in &doc.departments {
        upsert_department(pool, dept).await?;
        report.departments_upserted += 1;
    }
    apply_rules(pool, &doc.rules, RuleScope::DepartmentsOnly, report).await
}

async fn load_roles_file(
    pool: &PgPool,
    services_path: &Path,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let Some(doc) = read_yaml::<RolesDoc>(services_path, ROLES_FILE).await? else {
        return Ok(());
    };
    apply_rules(pool, &doc.rules, RuleScope::RolesOnly, report).await
}

#[derive(Debug, Clone, Copy)]
enum RuleScope {
    RolesOnly,
    DepartmentsOnly,
}

async fn apply_one_rule(
    repo: &AccessControlRepository,
    rule: &YamlRule,
    scope: RuleScope,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    rule.validate_target()?;
    let source_label = match scope {
        RuleScope::RolesOnly => "roles.yaml",
        RuleScope::DepartmentsOnly => "departments.yaml",
    };

    let target_ids = resolve_target_ids(repo, rule).await?;
    if target_ids.is_empty() {
        report.rules_skipped += 1;
        return Ok(());
    }
    if rule.entity_match.is_some() {
        report.glob_rules_expanded += 1;
        report.glob_entities_matched += target_ids.len();
    }

    let (rule_type, values, justification) = match scope {
        RuleScope::RolesOnly => (
            RuleType::Role,
            &rule.roles,
            "services/access-control/roles.yaml",
        ),
        RuleScope::DepartmentsOnly => (
            RuleType::Department,
            &rule.departments,
            "services/access-control/departments.yaml",
        ),
    };

    for target_id in &target_ids {
        repo.upsert_entity(
            rule.entity_type,
            target_id,
            rule.default_included,
            source_label,
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        for value in values {
            repo.upsert_rule(UpsertRuleParams {
                entity_type: rule.entity_type,
                entity_id: target_id,
                rule_type,
                rule_value: value,
                access: rule.access,
                justification: Some(justification),
            })
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
            report.rules_upserted += 1;
        }
    }
    Ok(())
}

async fn resolve_target_ids(
    repo: &AccessControlRepository,
    rule: &YamlRule,
) -> Result<Vec<String>, MarketplaceError> {
    if let Some(literal) = rule.entity_id.as_deref() {
        return Ok(vec![literal.to_owned()]);
    }
    let Some(pattern) = rule.entity_match.as_deref() else {
        return Ok(Vec::new());
    };
    let catalog = repo
        .list_entities(rule.entity_type)
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    Ok(catalog
        .into_iter()
        .filter(|row| glob_matches(pattern, &row.id))
        .map(|row| row.id)
        .collect())
}

async fn apply_rules(
    pool: &PgPool,
    rules: &[YamlRule],
    scope: RuleScope,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    for rule in rules {
        apply_one_rule(&repo, rule, scope, report).await?;
    }
    Ok(())
}

async fn upsert_department(pool: &PgPool, dept: &YamlDepartment) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO departments (name, description)
         VALUES ($1, $2)
         ON CONFLICT (name) DO UPDATE
            SET description = EXCLUDED.description,
                updated_at = NOW()",
        dept.name,
        dept.description,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Serialize)]
struct EntityKey {
    entity_type: EntityKind,
    entity_id: String,
    access: Access,
    default_included: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    roles: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    departments: Vec<String>,
}

#[derive(Serialize)]
struct Snapshot {
    rules: Vec<EntityKey>,
}

pub async fn render_yaml_snapshot(pool: &PgPool) -> Result<String, MarketplaceError> {
    use std::collections::BTreeMap;

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
           WHERE r.rule_type IN ('role', 'department')
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
            departments: Vec::new(),
        });
        match row.rule_type {
            RuleType::Role => entry.roles.push(row.rule_value),
            RuleType::Department => entry.departments.push(row.rule_value),
            RuleType::User => {}
        }
    }

    let snap = Snapshot {
        rules: by_key.into_values().collect(),
    };
    serde_yaml::to_string(&snap)
        .map_err(|e| MarketplaceError::Internal(format!("yaml render failed: {e}")))
}
