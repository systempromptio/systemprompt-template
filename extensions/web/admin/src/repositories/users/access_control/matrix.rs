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

use super::matrix_resolution::{MatrixCell, resolve_effective_with};
use super::matrix_subject::{MatrixSubject, user_subject};
use super::rules::list_all_rules;
use crate::authz::dimensions;
use crate::types::access_control::AccessControlRule;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::SubjectDimension;

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
    let subject = user_subject(pool, user_id, user.roles.clone()).await?;
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
    if sections_in.is_empty() {
        return Ok(Vec::new());
    }
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
    chains: systemprompt_security::authz::ParentChainIndex,
}

pub(super) async fn resolution_inputs(pool: &PgPool) -> Result<ResolutionInputs, sqlx::Error> {
    use systemprompt_security::authz::{AccessControlRepository, ChainSources, ParentChainIndex};
    let services = systemprompt::loader::ServicesBootstrap::get()
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let repo = AccessControlRepository::from_pool(std::sync::Arc::new(pool.clone()));
    let chains = ParentChainIndex::load(
        &repo,
        std::sync::Arc::new(ChainSources::from_services(services)),
    )
    .await
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    Ok(ResolutionInputs {
        chains,
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
            let (effective, source) = resolve_effective_with(
                &MatrixCell {
                    all_rules: &inputs.rules,
                    entity_type: &entity_type,
                    entity_id: &entity_id,
                    subject_id: &subject.id,
                    subject_roles: &subject.roles,
                    attributes: &subject.attributes,
                    dimensions,
                    default_included,
                },
                &inputs.chains,
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
    Ok(row.map(|row| UserMatrixUser {
        id: row.id,
        email: Some(row.email),
        display_name: row.display_name,
        roles: row.roles,
        group_ids: row.group_ids,
        project_ids: row.project_ids,
    }))
}
