//! Per-user access matrix: resolves the effective grant for every catalog
//! entity by walking user / department / role rules and the entity default.

use std::collections::HashMap;

use serde::Serialize;
use sqlx::PgPool;

use super::rules::list_all_rules;
use crate::types::access_control::{AccessControlRule, AccessDecision, RuleType};

/// Five-section access matrix for one user. Each section lists every entity of
/// a given kind that exists on this deployment, paired with the access
/// resolution chain for the target user.
///
/// Resolution precedence (highest first):
///   1. user-scoped rule (deny > allow)
///   2. department-scoped rule (deny > allow)
///   3. role-scoped rule (deny > allow)
///   4. entity's `default_included` flag (from `access_control_entities`)
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

/// Filter a catalog snapshot down to entries `user_id` is allowed to see.
///
/// Wraps [`resolve_user_matrix`] without altering the matrix shape —
/// callers project the section rows themselves.
pub async fn filter_catalog_for_user(
    pool: &PgPool,
    user_id: &str,
    sections_in: Vec<SectionInput>,
) -> Result<UserMatrix, sqlx::Error> {
    resolve_user_matrix(pool, user_id, sections_in).await
}

pub async fn resolve_user_matrix(
    pool: &PgPool,
    user_id: &str,
    sections_in: Vec<SectionInput>,
) -> Result<UserMatrix, sqlx::Error> {
    let user = fetch_user_for_matrix(pool, user_id).await?;
    let all_rules = list_all_rules(pool).await?;
    let defaults = load_entity_defaults(pool).await?;

    let mut sections: Vec<MatrixSection> = Vec::with_capacity(sections_in.len());
    for (entity_type, label, rows_in) in sections_in {
        let mut out_rows = Vec::with_capacity(rows_in.len());
        for (entity_id, name, desc) in rows_in {
            let default_included = defaults
                .get(&(entity_type.clone(), entity_id.clone()))
                .copied()
                .unwrap_or(false);
            let (effective, source) = resolve_effective(
                &all_rules,
                &entity_type,
                &entity_id,
                &user,
                default_included,
            );
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
    user_id: &str,
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
        user_id
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

/// Highest-priority matching rule at each scope (user / department / role).
struct ScopedRules<'a> {
    user: Option<&'a AccessControlRule>,
    dept: Option<&'a AccessControlRule>,
    role: Option<&'a AccessControlRule>,
}

fn select_scoped_rules<'a>(
    all_rules: &'a [AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    user: &UserMatrixUser,
) -> ScopedRules<'a> {
    let mut scoped = ScopedRules {
        user: None,
        dept: None,
        role: None,
    };

    for r in all_rules {
        if r.entity_type != entity_type || r.entity_id != entity_id {
            continue;
        }
        let is_deny = r.access == AccessDecision::Deny;
        match r.rule_type {
            RuleType::User if r.rule_value == user.id => {
                if scoped.user.is_none() || is_deny {
                    scoped.user = Some(r);
                }
            }
            RuleType::Department => {
                if let Some(d) = &user.department {
                    if &r.rule_value == d && (scoped.dept.is_none() || is_deny) {
                        scoped.dept = Some(r);
                    }
                }
            }
            RuleType::Role => {
                if user.roles.iter().any(|x| x == &r.rule_value)
                    && (scoped.role.is_none() || is_deny)
                {
                    scoped.role = Some(r);
                }
            }
            RuleType::User => {}
        }
    }

    scoped
}

fn resolve_effective(
    all_rules: &[AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    user: &UserMatrixUser,
    default_included: bool,
) -> (String, MatrixSource) {
    let scoped = select_scoped_rules(all_rules, entity_type, entity_id, user);

    if let Some(r) = scoped.user {
        return (
            r.access.to_string(),
            MatrixSource {
                layer: "user".into(),
                detail: format!("user:{} {}", user.id, r.access),
            },
        );
    }
    if let Some(r) = scoped.dept {
        return (
            r.access.to_string(),
            MatrixSource {
                layer: "department".into(),
                detail: format!("department:{} {}", r.rule_value, r.access),
            },
        );
    }
    if let Some(r) = scoped.role {
        return (
            r.access.to_string(),
            MatrixSource {
                layer: "role".into(),
                detail: format!("role:{} {}", r.rule_value, r.access),
            },
        );
    }

    let effective = if default_included { "allow" } else { "deny" };
    (
        effective.to_string(),
        MatrixSource {
            layer: "default".into(),
            detail: if default_included {
                "default-included".into()
            } else {
                "no rule matched".into()
            },
        },
    )
}
