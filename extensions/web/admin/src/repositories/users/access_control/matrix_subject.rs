//! Resolving the access matrix for a subject that is not necessarily a person.
//!
//! The resolver takes a user id, a role list and a bag of attribute values. A
//! *group* has none of those, but the audience matrix on the marketplaces page
//! asks exactly the same question of a group that the per-user matrix asks of a
//! person: which entities does this subject reach, and which band decided?
//!
//! [`MatrixSubject`] is that question's subject. A real user carries their own
//! id, roles and attributes; a synthetic group or role subject carries a
//! placeholder id and exactly one attribute band, so a `group` rule fires for
//! it and nothing else does. The placeholder id can never collide with a real
//! account: `UserId` values are emails or directory identifiers, and `group:`
//! is not a legal prefix for either.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{RuleType, SubjectAttributes};

use super::matrix::{
    MatrixSection, SectionInput, resolution_inputs, resolve_sections_for, sections_with,
};
use crate::authz::subject_attributes_for;

#[derive(Debug, Clone)]
pub struct MatrixSubject {
    pub id: UserId,
    pub roles: Vec<String>,
    pub attributes: SubjectAttributes,
}

fn banded_subject(prefix: &str, band: &str, value: &str) -> MatrixSubject {
    let mut attributes = SubjectAttributes::new();
    if let Ok(rule_type) = RuleType::extension(band.to_owned()) {
        attributes.insert(rule_type, vec![value.to_owned()]);
    }
    MatrixSubject {
        id: UserId::new(format!("{prefix}:{value}")),
        roles: Vec::new(),
        attributes,
    }
}

// Why: A subject that holds one group membership and nothing else.
#[must_use]
pub fn group_subject(slug: &str) -> MatrixSubject {
    banded_subject("group", "group", slug)
}

// Why: A subject that holds exactly one role and no attributes.
#[must_use]
pub fn role_subject(role: &str) -> MatrixSubject {
    MatrixSubject {
        id: UserId::new(format!("role:{role}")),
        roles: vec![role.to_owned()],
        attributes: SubjectAttributes::new(),
    }
}

// Why: The real subject behind a signed-in account: its own id, roles, and
// every registered dimension's values gathered from the database.
pub async fn user_subject(pool: &PgPool, user_id: &UserId, roles: Vec<String>) -> MatrixSubject {
    MatrixSubject {
        id: user_id.clone(),
        roles,
        attributes: subject_attributes_for(pool, user_id).await,
    }
}

// Why: Resolve one subject against the supplied entity sections.
pub async fn resolve_subject_matrix(
    pool: &PgPool,
    subject: &MatrixSubject,
    sections: Vec<SectionInput>,
) -> Result<Vec<MatrixSection>, sqlx::Error> {
    resolve_sections_for(pool, subject, sections).await
}

// Why: Resolve many subjects against the same sections, reading the rule table
// and the entity defaults once for the whole batch. The audience matrix asks
// the same question of every group and every role, so the single-subject call
// in a loop would re-read both tables for each row.
pub async fn resolve_subject_matrices(
    pool: &PgPool,
    subjects: &[MatrixSubject],
    sections: &[SectionInput],
) -> Result<Vec<Vec<MatrixSection>>, sqlx::Error> {
    let inputs = resolution_inputs(pool).await?;
    let dimensions = crate::authz::dimensions(pool);
    Ok(subjects
        .iter()
        .map(|subject| sections_with(&inputs, dimensions, subject, sections.to_vec()))
        .collect())
}
