//! Per-user access matrix: the effective grant for every catalog entity.
//!
//! This calls the same [`systemprompt_security::authz::resolve`] that
//! `POST /govern/authz` calls, over the same rules and the same subject
//! dimensions, so a cell here and a decision at the enforcement point cannot
//! disagree. It used to carry its own forked `user > department > role`
//! implementation; that fork is gone, and department is now a subject
//! dimension the resolver understands (see [`crate::authz::department`]).
//!
//! `MatrixSource::layer` names which band decided, mapped back from the
//! resolver's `MatchedBy` / `DenyReason`.

use std::collections::HashMap;
use std::str::FromStr;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{RuleId, UserId};
use systemprompt_security::authz::{
    Access, AccessRule, Decision, DenyReason, EntityKind, MatchedBy, ResolveInput,
    SubjectAttributes, SubjectDimension, resolve,
};

use super::rules::list_all_rules;
use crate::authz::{dimensions, subject_attributes_for};
use crate::marketplace_filter::entity_ref_for;
use crate::types::access_control::{AccessControlRule, AccessDecision};

#[derive(Debug, Serialize)]
pub struct UserMatrix {
    pub user: UserMatrixUser,
    pub sections: Vec<MatrixSection>,
}

#[derive(Debug, Serialize)]
pub struct UserMatrixUser {
    pub id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub department: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MatrixSection {
    pub entity_type: String,
    pub label: String,
    pub rows: Vec<MatrixRow>,
}

#[derive(Debug, Serialize)]
pub struct MatrixRow {
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
    pub entity_id: String,
    pub entity_name: String,
    pub description: Option<String>,
    pub effective: String,
    pub source: MatrixSource,
    pub default_included: bool,
}

#[derive(Debug, Serialize)]
pub struct MatrixSource {
    pub layer: String,
    pub detail: String,
}

/// Section definition supplied by the caller — list of entities of a given
/// kind that exist on this deployment.
pub type SectionInput = (String, String, Vec<(String, String, Option<String>)>);

pub async fn filter_catalog_for_user(
    pool: &PgPool,
    user_id: &UserId,
    sections_in: Vec<SectionInput>,
) -> Result<UserMatrix, sqlx::Error> {
    resolve_user_matrix(pool, user_id, sections_in).await
}

pub async fn resolve_user_matrix(
    pool: &PgPool,
    user_id: &UserId,
    sections_in: Vec<SectionInput>,
) -> Result<UserMatrix, sqlx::Error> {
    let user = fetch_user_for_matrix(pool, user_id).await?;
    let all_rules = list_all_rules(pool).await?;
    let defaults = load_entity_defaults(pool).await?;
    // The same lookup the enforcement webhook performs, so the matrix and the
    // decision see identical subject values.
    let attributes = subject_attributes_for(pool, user_id).await;
    let dimensions = dimensions(pool);

    let mut sections: Vec<MatrixSection> = Vec::with_capacity(sections_in.len());
    for (entity_type, label, rows_in) in sections_in {
        let mut out_rows = Vec::with_capacity(rows_in.len());
        for (entity_id, name, desc) in rows_in {
            let default_included = defaults
                .get(&(entity_type.clone(), entity_id.clone()))
                .copied()
                .unwrap_or(false);
            let (effective, source) = resolve_effective(&MatrixCell {
                all_rules: &all_rules,
                entity_type: &entity_type,
                entity_id: &entity_id,
                user: &user,
                attributes: &attributes,
                dimensions,
                default_included,
            });
            out_rows.push(MatrixRow {
                entity_id,
                entity_name: name,
                description: desc,
                effective,
                source,
                default_included,
            });
        }
        sections.push(MatrixSection {
            entity_type,
            label,
            rows: out_rows,
        });
    }

    Ok(UserMatrix { user, sections })
}

async fn load_entity_defaults(
    pool: &PgPool,
) -> Result<HashMap<(String, String), bool>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT entity_type, entity_id, default_included
           FROM access_control_entities"#
    )
    .fetch_all(pool)
    .await?;
    let mut out = HashMap::with_capacity(rows.len());
    for row in rows {
        out.insert((row.entity_type, row.entity_id), row.default_included);
    }
    Ok(out)
}

async fn fetch_user_for_matrix(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<UserMatrixUser, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT u.id,
                  u.email,
                  COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                  u.roles AS "roles!: Vec<String>",
                  COALESCE(upe.department, 'Default') AS "department!"
           FROM users u
           LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
           WHERE u.id = $1"#,
        user_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    let dept = if row.department.trim().is_empty() {
        None
    } else {
        Some(row.department)
    };
    Ok(UserMatrixUser {
        id: row.id,
        email: Some(row.email),
        display_name: row.display_name,
        roles: row.roles,
        department: dept,
    })
}

/// Adapts an admin-CRUD row to the resolver's rule type.
///
/// The two differ only in what the matrix screen needs on top of a decision
/// (timestamps, entity coordinates). `justification` is not selected by the
/// matrix query, so the resolver sees `None` and the matrix renders its own
/// detail string.
fn as_access_rule(row: &AccessControlRule) -> AccessRule {
    AccessRule {
        id: RuleId::new(row.id.clone()),
        rule_type: row.rule_type.clone(),
        rule_value: row.rule_value.clone(),
        access: match row.access {
            AccessDecision::Allow => Access::Allow,
            AccessDecision::Deny => Access::Deny,
        },
        justification: None,
    }
}

struct MatrixCell<'a> {
    all_rules: &'a [AccessControlRule],
    entity_type: &'a str,
    entity_id: &'a str,
    user: &'a UserMatrixUser,
    attributes: &'a SubjectAttributes,
    dimensions: &'a [SubjectDimension],
    default_included: bool,
}

/// Runs the shared resolver for one cell and translates its verdict into the
/// screen's `(effective, source)` pair.
///
/// An `entity_type` the core catalog does not recognise cannot be resolved at
/// all, so the cell reports the entity default and says so rather than
/// implying a rule decided it.
fn resolve_effective(cell: &MatrixCell<'_>) -> (String, MatrixSource) {
    let Ok(kind) = EntityKind::from_str(cell.entity_type) else {
        return (
            if cell.default_included { "allow" } else { "deny" }.to_owned(),
            MatrixSource {
                layer: "default".into(),
                detail: format!("unknown entity type: {}", cell.entity_type),
            },
        );
    };
    let entity = entity_ref_for(kind, cell.entity_id);
    let rules: Vec<AccessRule> = cell
        .all_rules
        .iter()
        .filter(|r| r.entity_type == cell.entity_type && r.entity_id == cell.entity_id)
        .map(as_access_rule)
        .collect();

    let uid = UserId::new(&cell.user.id);
    let decision = resolve(ResolveInput {
        entity: &entity,
        rules: &rules,
        user_id: &uid,
        user_roles: &cell.user.roles,
        default_included: Some(cell.default_included),
        parents: &[],
        attributes: cell.attributes,
        dimensions: cell.dimensions,
    });

    match decision {
        Decision::Allow { matched_by } => ("allow".to_owned(), allow_source(&uid, &matched_by)),
        Decision::Deny { reason } => ("deny".to_owned(), deny_source(&uid, &reason)),
    }
}

fn allow_source(user_id: &UserId, matched_by: &MatchedBy) -> MatrixSource {
    match matched_by {
        MatchedBy::UserAllow => MatrixSource {
            layer: "user".into(),
            detail: format!("user:{user_id} allow"),
        },
        MatchedBy::RoleAllow { role } => MatrixSource {
            layer: "role".into(),
            detail: format!("role:{role} allow"),
        },
        MatchedBy::AttributeAllow { rule_type, value } => MatrixSource {
            layer: rule_type.to_string(),
            detail: format!("{rule_type}:{value} allow"),
        },
        MatchedBy::DefaultIncluded => MatrixSource {
            layer: "default".into(),
            detail: "default-included".into(),
        },
        MatchedBy::PolicyAllow { policy_id, detail } => MatrixSource {
            layer: "policy".into(),
            detail: format!("{policy_id}: {detail}"),
        },
    }
}

fn deny_source(user_id: &UserId, reason: &DenyReason) -> MatrixSource {
    match reason {
        DenyReason::UserDeny { .. } => MatrixSource {
            layer: "user".into(),
            detail: format!("user:{user_id} deny"),
        },
        DenyReason::RoleDeny { role, .. } => MatrixSource {
            layer: "role".into(),
            detail: format!("role:{role} deny"),
        },
        DenyReason::AttributeDeny {
            rule_type, value, ..
        } => MatrixSource {
            layer: rule_type.to_string(),
            detail: format!("{rule_type}:{value} deny"),
        },
        // Everything else is the resolver closing the default rather than a
        // rule firing, so the cell reports the default layer and lets the
        // reason speak for itself.
        other => MatrixSource {
            layer: "default".into(),
            detail: other.to_string(),
        },
    }
}
