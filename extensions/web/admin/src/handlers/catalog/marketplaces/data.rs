//! Data assembly for the marketplace catalog pages.
//!
//! The audience matrix asks the resolver one question per (subject,
//! marketplace) pair, where the subject is a group or a role rather than a
//! person. Every row goes through the same
//! [`resolve_subject_matrix`](crate::repositories::users::access_control::resolve_subject_matrix)
//! the per-user matrix uses, so a cell here and a decision at the enforcement
//! point cannot disagree.

use std::collections::HashMap;
use std::path::Path;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_shared::GroupId;

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::marketplace::manifests::MarketplaceConfigSummary;
use crate::repositories::users::access_control::{
    MatrixSection, MatrixSubject, group_subject, resolve_subject_matrices, role_subject,
};

use super::view::{AudienceCellView, AudienceColumnView, AudienceMatrixView, AudienceRowView};

pub(crate) const MARKETPLACE_ENTITY: &str = "marketplace";

// Why: the manifests name the marketplaces; the database says who reaches
// them. Both are needed for every card and every audience cell.
pub(super) async fn load_manifests(
    pool: &PgPool,
    services_path: &Path,
) -> Vec<MarketplaceConfigSummary> {
    let mut manifests =
        repositories::marketplace::manifests::list_marketplace_configs(services_path)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to load marketplace manifests");
                Vec::new()
            });
    repositories::marketplace::manifests_access::attach_access(pool, &mut manifests).await;
    manifests
}

fn section_input(
    manifests: &[MarketplaceConfigSummary],
) -> Vec<repositories::users::access_control::SectionInput> {
    let rows = manifests
        .iter()
        .map(|m| (m.id.as_str().to_owned(), m.name.clone(), None))
        .collect();
    vec![(
        MARKETPLACE_ENTITY.to_owned(),
        "Marketplaces".to_owned(),
        rows,
    )]
}

fn cells_of(sections: Vec<MatrixSection>) -> Vec<AudienceCellView> {
    sections
        .into_iter()
        .next()
        .map_or_else(Vec::new, |section| {
            section
                .rows
                .into_iter()
                .map(|row| AudienceCellView {
                    marketplace_id: MarketplaceId::new(row.entity_id),
                    is_allow: row.effective == "allow",
                    effective: row.effective,
                    layer: row.source.layer,
                    detail: row.source.detail,
                })
                .collect()
        })
}

// Why: The full audience matrix: every group, then every role, against every
// marketplace. Groups come first because entitlement is granted on groups
// here; the role rows are the baseline that a group row either widens or
// leaves alone.
pub(super) async fn audience_matrix(
    pool: &PgPool,
    manifests: &[MarketplaceConfigSummary],
    roles: &[String],
) -> AudienceMatrixView {
    let groups = repositories::groups::crud::list_group_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "audience matrix: group listing failed"))
        .unwrap_or_default();

    let mut subjects: Vec<MatrixSubject> = Vec::with_capacity(groups.len() + roles.len());
    let mut labels: Vec<(String, &'static str)> = Vec::with_capacity(subjects.capacity());
    for group in &groups {
        subjects.push(group_subject(group.id.as_str()));
        labels.push((group.name.clone(), "group"));
    }
    for role in roles {
        subjects.push(role_subject(role));
        labels.push((role.clone(), "role"));
    }

    let resolved = resolve_subject_matrices(pool, &subjects, &section_input(manifests))
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "audience matrix failed to resolve"))
        .unwrap_or_default();

    let mut resolved = resolved.into_iter();
    let rows: Vec<AudienceRowView> = subjects
        .iter()
        .zip(labels)
        .map(|(subject, (label, kind))| AudienceRowView {
            subject: subject.id.as_str().to_owned(),
            label,
            kind,
            cells: resolved.next().map_or_else(Vec::new, cells_of),
        })
        .collect();

    AudienceMatrixView {
        columns: manifests
            .iter()
            .map(|m| AudienceColumnView {
                id: m.id.clone(),
                name: m.name.clone(),
            })
            .collect(),
        has_rows: !rows.is_empty(),
        rows,
    }
}

// Why: Which groups hold an explicit `allow` rule on each marketplace.
//
// This is the *declared* half of entitlement — the rule a person clicked into
// existence. The resolved half is the audience matrix, and the list page shows
// both because a deny written elsewhere can close a marketplace this map says
// is open.
pub(super) async fn group_grants(pool: &PgPool) -> HashMap<MarketplaceId, Vec<GroupId>> {
    let rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: rule listing failed"))
        .unwrap_or_default();
    let mut out: HashMap<MarketplaceId, Vec<GroupId>> = HashMap::new();
    for rule in rules {
        if rule.entity_type == MARKETPLACE_ENTITY
            && rule.rule_type.as_str() == "group"
            && rule.access.to_string() == "allow"
        {
            out.entry(MarketplaceId::new(rule.entity_id))
                .or_default()
                .push(GroupId::new(rule.rule_value));
        }
    }
    for ids in out.values_mut() {
        ids.sort();
    }
    out
}
