//! Per-user access matrix: the effective grant for every catalog entity.
//!
//! This calls the same [`systemprompt_security::authz::resolver::resolve`] that
//! `POST /govern/authz` calls, over the same rules and the same subject
//! dimensions, so a cell here and a decision at the enforcement point cannot
//! disagree. Every subject dimension this extension declares (see
//! [`crate::authz`]) takes part, exactly as at the enforcement point.
//!
//! `MatrixSource::layer` names which band decided, mapped back from the
//! resolver's `MatchedBy` / `DenyReason`.

use std::collections::HashMap;
use std::str::FromStr;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{RuleId, UserId};
use systemprompt_security::authz::resolver::{ResolveInput, resolve};
use systemprompt_security::authz::{
    Access, AccessRule, Decision, EntityKind, EntityRef, SubjectAttributes, SubjectDimension,
};

use super::matrix_source::{allow_source, deny_source};
use super::matrix_subject::{MatrixSubject, user_subject};
use super::rules::list_all_rules;
use crate::authz::dimensions;
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
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
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
) -> Result<Option<UserMatrix>, sqlx::Error> {
    resolve_user_matrix(pool, user_id, sections_in).await
}

pub async fn resolve_user_matrix(
    pool: &PgPool,
    user_id: &UserId,
    sections_in: Vec<SectionInput>,
) -> Result<Option<UserMatrix>, sqlx::Error> {
    let Some(user) = find_user_for_matrix(pool, user_id).await? else {
        return Ok(None);
    };
    // Why: the same lookup the enforcement webhook performs, so the matrix and
    // the decision see identical subject values.
    let subject = user_subject(pool, user_id, user.roles.clone()).await;
    let sections = resolve_sections_for(pool, &subject, sections_in).await?;
    Ok(Some(UserMatrix { user, sections }))
}

// Why: Resolve every supplied section for one subject — a person, a group, or a
// role. The subject abstraction is what lets the audience matrix ask the
// question of a group without inventing a second resolver.
pub(super) async fn resolve_sections_for(
    pool: &PgPool,
    subject: &MatrixSubject,
    sections_in: Vec<SectionInput>,
) -> Result<Vec<MatrixSection>, sqlx::Error> {
    let inputs = resolution_inputs(pool).await?;
    Ok(sections_with(
        &inputs,
        dimensions(pool),
        subject,
        sections_in,
    ))
}

// Why: The rule set and entity defaults every cell resolves against. Loaded
// once per page rather than once per subject: the audience matrix asks the same
// question of a dozen subjects, and re-reading the whole rule table for each
// of them turns one page render into two dozen round trips.
pub(super) struct ResolutionInputs {
    rules: Vec<AccessControlRule>,
    defaults: HashMap<(String, String), bool>,
}

pub(super) async fn resolution_inputs(pool: &PgPool) -> Result<ResolutionInputs, sqlx::Error> {
    Ok(ResolutionInputs {
        rules: list_all_rules(pool).await?,
        defaults: load_entity_defaults(pool).await?,
    })
}

pub(super) fn sections_with(
    inputs: &ResolutionInputs,
    dimensions: &[SubjectDimension],
    subject: &MatrixSubject,
    sections_in: Vec<SectionInput>,
) -> Vec<MatrixSection> {
    let mut sections: Vec<MatrixSection> = Vec::with_capacity(sections_in.len());
    for (entity_type, label, rows_in) in sections_in {
        let mut out_rows = Vec::with_capacity(rows_in.len());
        for (entity_id, name, desc) in rows_in {
            let default_included = inputs
                .defaults
                .get(&(entity_type.clone(), entity_id.clone()))
                .copied()
                .unwrap_or(false);
            let (effective, source) = resolve_effective(&MatrixCell {
                all_rules: &inputs.rules,
                entity_type: &entity_type,
                entity_id: &entity_id,
                subject_id: &subject.id,
                subject_roles: &subject.roles,
                attributes: &subject.attributes,
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
    sections
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

async fn find_user_for_matrix(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserMatrixUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT u.id,
                  u.email,
                  COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                  u.roles AS "roles!: Vec<String>",
                  ARRAY(SELECT ug.group_id FROM user_groups ug
                        WHERE ug.user_id = u.id) AS "group_ids!: Vec<String>",
                  ARRAY(SELECT DISTINCT pm.project_id FROM project_members pm
                        WHERE pm.user_id = u.id) AS "project_ids!: Vec<String>"
           FROM users u
           WHERE u.id = $1"#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    let department = if row.is_some() {
        crate::repositories::users::queries::find_user_roles_department(pool, user_id)
            .await?
            .map(|(_, department)| department)
    } else {
        None
    };
    Ok(row.map(|row| UserMatrixUser {
        id: row.id,
        email: Some(row.email),
        display_name: row.display_name,
        roles: row.roles,
        department,
        group_ids: row.group_ids,
        project_ids: row.project_ids,
    }))
}

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

// Why: the cell names a *subject*, not a user — the group and role rows of the
// audience matrix resolve through the same resolver call as a person does, and
// only the id and the role list differ between them.
pub(crate) struct MatrixCell<'a> {
    pub all_rules: &'a [AccessControlRule],
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub subject_id: &'a UserId,
    pub subject_roles: &'a [String],
    pub attributes: &'a SubjectAttributes,
    pub dimensions: &'a [SubjectDimension],
    pub default_included: bool,
}

pub(crate) fn resolve_effective(cell: &MatrixCell<'_>) -> (String, MatrixSource) {
    let Ok(kind) = EntityKind::from_str(cell.entity_type) else {
        return (
            if cell.default_included {
                "allow"
            } else {
                "deny"
            }
            .to_owned(),
            MatrixSource {
                layer: "default".into(),
                detail: format!("unknown entity type: {}", cell.entity_type),
            },
        );
    };
    let entity = EntityRef::from_kind_and_id(kind, cell.entity_id);
    let rules: Vec<AccessRule> = cell
        .all_rules
        .iter()
        .filter(|r| r.entity_type == cell.entity_type && r.entity_id == cell.entity_id)
        .map(as_access_rule)
        .collect();

    let uid = cell.subject_id;
    let decision = resolve(ResolveInput {
        entity: &entity,
        rules: &rules,
        user_id: uid,
        user_roles: cell.subject_roles,
        default_included: Some(cell.default_included),
        parents: &[],
        attributes: cell.attributes,
        dimensions: cell.dimensions,
    });

    match decision {
        Decision::Allow { matched_by } => ("allow".to_owned(), allow_source(uid, &matched_by)),
        // Why: a warning is a reach, but not a clean one. The cell names it so
        // an operator reading the matrix under warn mode sees which cells are
        // only open because enforcement is currently off.
        Decision::Warn { reason } => (
            "warn".to_owned(),
            MatrixSource {
                layer: "warn".into(),
                detail: reason.to_string(),
            },
        ),
        Decision::Deny { reason } => ("deny".to_owned(), deny_source(uid, &reason)),
        // Why: a hold is neither reach nor refusal, and flattening it into
        // either would misreport the matrix. The cell names it.
        Decision::Pending { reason } => (
            "pending".to_owned(),
            MatrixSource {
                layer: "approval".into(),
                detail: reason.to_string(),
            },
        ),
    }
}
