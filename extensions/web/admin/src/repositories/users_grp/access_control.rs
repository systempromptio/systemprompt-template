use serde::Serialize;
use sqlx::PgPool;

use crate::types::access_control::{
    AccessControlRule, AccessControlRuleInput, AccessDecision, RuleType,
};

pub async fn list_all_rules(pool: &PgPool) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    sqlx::query_as::<_, AccessControlRule>(
        "SELECT id, entity_type, entity_id, rule_type, rule_value, access, default_included, created_at, updated_at
         FROM access_control_rules
         ORDER BY entity_type, entity_id, rule_type, rule_value",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_rules_for_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    sqlx::query_as::<_, AccessControlRule>(
        "SELECT id, entity_type, entity_id, rule_type, rule_value, access, default_included, created_at, updated_at
         FROM access_control_rules
         WHERE entity_type = $1 AND entity_id = $2
         ORDER BY rule_type, rule_value",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
}

pub async fn set_entity_rules(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
    rules: &[AccessControlRuleInput],
) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM access_control_rules WHERE entity_type = $1 AND entity_id = $2",
        entity_type,
        entity_id
    )
    .execute(&mut *tx)
    .await?;

    let mut results = Vec::new();
    for rule in rules {
        let id = uuid::Uuid::new_v4().to_string();
        let rule_type_str = rule.rule_type.to_string();
        let access_str = rule.access.to_string();
        let row = sqlx::query_as::<_, AccessControlRule>(
            r"INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, default_included)
              VALUES ($1, $2, $3, $4, $5, $6, $7)
              RETURNING id, entity_type, entity_id, rule_type, rule_value, access, default_included, created_at, updated_at",
        )
        .bind(&id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(&rule_type_str)
        .bind(&rule.rule_value)
        .bind(&access_str)
        .bind(rule.default_included)
        .fetch_one(&mut *tx)
        .await?;
        results.push(row);
    }

    tx.commit().await?;
    Ok(results)
}

pub async fn bulk_set_rules(
    pool: &PgPool,
    entities: &[(String, String)],
    rules: &[AccessControlRuleInput],
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut count = 0usize;

    for (entity_type, entity_id) in entities {
        sqlx::query!(
            "DELETE FROM access_control_rules WHERE entity_type = $1 AND entity_id = $2",
            entity_type,
            entity_id
        )
        .execute(&mut *tx)
        .await?;

        for rule in rules {
            let id = uuid::Uuid::new_v4().to_string();
            let rule_type_str = rule.rule_type.to_string();
            let access_str = rule.access.to_string();
            sqlx::query!(
                r"INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, default_included)
                  VALUES ($1, $2, $3, $4, $5, $6, $7)",
                id,
                entity_type,
                entity_id,
                rule_type_str,
                rule.rule_value,
                access_str,
                rule.default_included,
            )
            .execute(&mut *tx)
            .await?;
        }
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

/// Five-section access matrix for one user. Each section lists every entity of
/// a given kind that exists on this deployment, paired with the access
/// resolution chain for the target user.
///
/// Resolution precedence (highest first):
///   1. user-scoped rule (deny > allow)
///   2. department-scoped rule (deny > allow)
///   3. role-scoped rule (deny > allow)
///   4. entity's `default_included` flag
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

pub async fn resolve_user_matrix(
    pool: &PgPool,
    user_id: &str,
    sections_in: Vec<SectionInput>,
) -> Result<UserMatrix, sqlx::Error> {
    let user = fetch_user_for_matrix(pool, user_id).await?;
    let all_rules = list_all_rules(pool).await?;

    let mut sections: Vec<MatrixSection> = Vec::with_capacity(sections_in.len());
    for (entity_type, label, rows_in) in sections_in {
        let mut out_rows = Vec::with_capacity(rows_in.len());
        for (entity_id, name, desc) in rows_in {
            let (effective, source, default_included) =
                resolve_effective(&all_rules, &entity_type, &entity_id, &user);
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

async fn fetch_user_for_matrix(
    pool: &PgPool,
    user_id: &str,
) -> Result<UserMatrixUser, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT id,
                  email,
                  COALESCE(display_name, full_name, name) AS display_name,
                  roles AS "roles!: Vec<String>",
                  department
           FROM users WHERE id = $1"#,
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

fn resolve_effective(
    all_rules: &[AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    user: &UserMatrixUser,
) -> (String, MatrixSource, bool) {
    let mut default_included = false;
    let mut user_rule: Option<&AccessControlRule> = None;
    let mut dept_rule: Option<&AccessControlRule> = None;
    let mut role_rule: Option<&AccessControlRule> = None;

    for r in all_rules {
        if r.entity_type != entity_type || r.entity_id != entity_id {
            continue;
        }
        if r.default_included {
            default_included = true;
        }
        let is_deny = r.access == AccessDecision::Deny;
        match r.rule_type {
            RuleType::User if r.rule_value == user.id => {
                if user_rule.is_none() || is_deny {
                    user_rule = Some(r);
                }
            }
            RuleType::Department => {
                if let Some(d) = &user.department {
                    if &r.rule_value == d && (dept_rule.is_none() || is_deny) {
                        dept_rule = Some(r);
                    }
                }
            }
            RuleType::Role => {
                if user.roles.iter().any(|x| x == &r.rule_value)
                    && (role_rule.is_none() || is_deny)
                {
                    role_rule = Some(r);
                }
            }
            RuleType::User => {}
        }
    }

    if let Some(r) = user_rule {
        return (
            r.access.to_string(),
            MatrixSource {
                layer: "user".into(),
                detail: format!("user:{} {}", user.id, r.access),
            },
            default_included,
        );
    }
    if let Some(r) = dept_rule {
        return (
            r.access.to_string(),
            MatrixSource {
                layer: "department".into(),
                detail: format!("department:{} {}", r.rule_value, r.access),
            },
            default_included,
        );
    }
    if let Some(r) = role_rule {
        return (
            r.access.to_string(),
            MatrixSource {
                layer: "role".into(),
                detail: format!("role:{} {}", r.rule_value, r.access),
            },
            default_included,
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
        default_included,
    )
}
