//! Bootstrap loader: `services/access-control/*.yaml` → DB.
//!
//! YAML is the declarative source of truth committed to the source repo.
//! On every server startup the [`PublishPipelineJob`] calls
//! [`load_from_yaml`] which upserts rule and department rows into the runtime
//! database. There is no write-back: dashboard edits live only in the DB of
//! the instance that received them, so multiple deployments sharing the same
//! YAML baseline can drift independently without trampling each other.
//!
//! Two files are read (both optional, missing-file = no-op):
//!
//! - `services/access-control/roles.yaml` — role-scoped allow/deny rules.
//! - `services/access-control/departments.yaml` — department definitions
//!   plus department-scoped allow/deny rules.
//!
//! For backward compatibility the legacy `services/gateway/access.yaml` —
//! a previous gateway-only ACL export — is still read if present, but is
//! never written.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, RuleType, UpsertRuleParams};
use systemprompt_web_shared::error::MarketplaceError;

const ROLES_FILE: &str = "access-control/roles.yaml";
const DEPARTMENTS_FILE: &str = "access-control/departments.yaml";
const LEGACY_GATEWAY_FILE: &str = "gateway/access.yaml";

/// Per-rule entry in `roles.yaml` and the `rules:` block of `departments.yaml`.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct YamlRule {
    /// One of: `gateway_route`, `mcp_server`, `plugin`, `agent`, `marketplace`.
    pub entity_type: String,
    pub entity_id: String,
    /// `allow` or `deny`. Defaults to `allow`.
    #[serde(default = "default_access")]
    pub access: String,
    #[serde(default)]
    pub default_included: bool,
    /// Roles to bind. Used in `roles.yaml`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    /// Department names to bind. Used in `departments.yaml`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub departments: Vec<String>,
}

fn default_access() -> String {
    "allow".to_owned()
}

/// `departments.yaml` top-level shape.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DepartmentsDoc {
    #[serde(default)]
    pub departments: Vec<YamlDepartment>,
    #[serde(default)]
    pub rules: Vec<YamlRule>,
}

/// `roles.yaml` top-level shape.
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

/// Counts surfaced to the publish pipeline log.
#[derive(Debug, Default, Clone, Copy)]
pub struct LoadReport {
    pub departments_upserted: usize,
    pub rules_upserted: usize,
    pub rules_skipped: usize,
}

/// Read all YAML files under `services_path` and apply them to the DB.
/// Idempotent. Missing files are no-ops.
pub async fn load_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<LoadReport, MarketplaceError> {
    let mut report = LoadReport::default();

    load_departments_file(pool, services_path, &mut report).await?;
    load_roles_file(pool, services_path, &mut report).await?;
    load_legacy_gateway_file(pool, services_path, &mut report).await?;

    tracing::info!(
        departments = report.departments_upserted,
        rules = report.rules_upserted,
        skipped = report.rules_skipped,
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

/// Backward-compat reader for the previous DB→YAML export shape.
async fn load_legacy_gateway_file(
    pool: &PgPool,
    services_path: &Path,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let path = services_path.join(LEGACY_GATEWAY_FILE);
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    let doc: LegacyDoc = serde_yaml::from_str(&content)
        .map_err(|e| MarketplaceError::Internal(format!("{LEGACY_GATEWAY_FILE}: {e}")))?;

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    for (route_id, entry) in &doc.routes {
        repo.set_default_included(
            systemprompt_security::authz::EntityKind::GatewayRoute,
            route_id,
            entry.default_included,
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        for (values, rule_type, access) in [
            (&entry.allow.roles, RuleType::Role, Access::Allow),
            (&entry.allow.departments, RuleType::Department, Access::Allow),
            (&entry.deny.roles, RuleType::Role, Access::Deny),
            (&entry.deny.departments, RuleType::Department, Access::Deny),
        ] {
            for value in values {
                repo.upsert_rule(UpsertRuleParams {
                    entity_type: systemprompt_security::authz::EntityKind::GatewayRoute,
                    entity_id: route_id,
                    rule_type,
                    rule_value: value,
                    access,
                    justification: Some("legacy gateway/access.yaml"),
                })
                .await
                .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
                report.rules_upserted += 1;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default, Deserialize)]
struct LegacyBuckets {
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    departments: Vec<String>,
}
#[derive(Debug, Default, Deserialize)]
struct LegacyEntry {
    #[serde(default)]
    default_included: bool,
    #[serde(default)]
    allow: LegacyBuckets,
    #[serde(default)]
    deny: LegacyBuckets,
}
#[derive(Debug, Default, Deserialize)]
struct LegacyDoc {
    #[serde(default)]
    routes: std::collections::BTreeMap<String, LegacyEntry>,
}

#[derive(Debug, Clone, Copy)]
enum RuleScope {
    RolesOnly,
    DepartmentsOnly,
}

fn parse_rule_kind_access(
    rule: &YamlRule,
    report: &mut LoadReport,
) -> Option<(systemprompt_security::authz::EntityKind, Access)> {
    let kind = match rule.entity_type.parse::<systemprompt_security::authz::EntityKind>() {
        Ok(k) => k,
        Err(e) => {
            tracing::warn!(error = %e, entity_type = %rule.entity_type, entity_id = %rule.entity_id, "yaml rule has invalid entity_type, skipping");
            report.rules_skipped += 1;
            return None;
        }
    };
    let access = match rule.access.as_str() {
        "allow" => Access::Allow,
        "deny" => Access::Deny,
        other => {
            tracing::warn!(access = other, entity_id = %rule.entity_id, "yaml rule has invalid access, skipping");
            report.rules_skipped += 1;
            return None;
        }
    };
    Some((kind, access))
}

async fn apply_one_rule(
    repo: &AccessControlRepository,
    rule: &YamlRule,
    scope: RuleScope,
    kind: systemprompt_security::authz::EntityKind,
    access: Access,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    repo.set_default_included(kind, &rule.entity_id, rule.default_included)
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

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

    for value in values {
        repo.upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &rule.entity_id,
            rule_type,
            rule_value: value,
            access,
            justification: Some(justification),
        })
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
        report.rules_upserted += 1;
    }
    Ok(())
}

async fn apply_rules(
    pool: &PgPool,
    rules: &[YamlRule],
    scope: RuleScope,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    for rule in rules {
        let Some((kind, access)) = parse_rule_kind_access(rule, report) else {
            continue;
        };
        apply_one_rule(&repo, rule, scope, kind, access, report).await?;
    }
    Ok(())
}

async fn upsert_department(
    pool: &PgPool,
    dept: &YamlDepartment,
) -> Result<(), MarketplaceError> {
    sqlx::query(
        "INSERT INTO departments (name, description)
         VALUES ($1, $2)
         ON CONFLICT (name) DO UPDATE
            SET description = EXCLUDED.description,
                updated_at = NOW()",
    )
    .bind(&dept.name)
    .bind(&dept.description)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Default, Serialize)]
struct EntityKey {
    entity_type: String,
    entity_id: String,
    access: String,
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

/// Read-only "Show as YAML" affordance for the admin UI.
///
/// Renders the current DB ACL state as a YAML snippet matching the loader's
/// schema, so admins can copy-paste instance-local edits back into the
/// committed baseline.
pub async fn render_yaml_snapshot(pool: &PgPool) -> Result<String, MarketplaceError> {
    use std::collections::BTreeMap;

    let rows: Vec<(String, String, String, String, String, bool)> = sqlx::query_as(
        "SELECT entity_type, entity_id, rule_type, rule_value, access, default_included
         FROM access_control_rules
         WHERE rule_type IN ('role', 'department')
         ORDER BY entity_type, entity_id, access, rule_type, rule_value",
    )
    .fetch_all(pool)
    .await?;

    let mut by_key: BTreeMap<(String, String, String), EntityKey> = BTreeMap::new();
    for (entity_type, entity_id, rule_type, rule_value, access, default_included) in rows {
        let key = (entity_type.clone(), entity_id.clone(), access.clone());
        let entry = by_key.entry(key).or_insert_with(|| EntityKey {
            entity_type: entity_type.clone(),
            entity_id: entity_id.clone(),
            access: access.clone(),
            default_included,
            roles: Vec::new(),
            departments: Vec::new(),
        });
        if rule_type == "role" {
            entry.roles.push(rule_value);
        } else if rule_type == "department" {
            entry.departments.push(rule_value);
        }
    }

    let snap = Snapshot {
        rules: by_key.into_values().collect(),
    };
    serde_yaml::to_string(&snap)
        .map_err(|e| MarketplaceError::Internal(format!("yaml render failed: {e}")))
}
