//! Data assembly for the marketplace catalog pages.
//!
//! The audience matrix asks the resolver one question per (subject,
//! marketplace) pair, where the subject is a group or a role rather than a
//! person. Every row goes through the same
//! [`resolve_subject_matrix`](crate::repositories::users::access_control::resolve_subject_matrix)
//! the per-user matrix uses, so a cell here and a decision at the enforcement
//! point cannot disagree.

use std::path::Path;

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::marketplace::manifests::MarketplaceConfigSummary;
use crate::repositories::users::access_control::{
    MatrixSection, MatrixSubject, group_subject, resolve_subject_matrices, role_subject,
};

use super::view::{
    AudienceCellView, AudienceColumnView, AudienceMatrixView, AudienceRowView, AudienceSubjectView,
};

pub(super) const MARKETPLACE_ENTITY: &str = "marketplace";

pub(super) fn load_manifests(services_path: &Path) -> Vec<MarketplaceConfigSummary> {
    repositories::marketplace::manifests::list_marketplace_configs(services_path).unwrap_or_else(
        |e| {
            tracing::warn!(error = %e, "Failed to load marketplace manifests");
            Vec::new()
        },
    )
}

fn section_input(
    manifests: &[MarketplaceConfigSummary],
) -> Vec<repositories::users::access_control::SectionInput> {
    let rows = manifests
        .iter()
        .map(|m| (m.id.clone(), m.name.clone(), None))
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
                    marketplace_id: row.entity_id,
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
        subjects.push(group_subject(&group.id));
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

fn subject_view(row: AudienceRowView, marketplace_id: &str) -> AudienceSubjectView {
    let cell = row
        .cells
        .into_iter()
        .find(|c| c.marketplace_id == marketplace_id);
    AudienceSubjectView {
        subject: row.subject,
        label: row.label,
        effective: cell
            .as_ref()
            .map_or_else(|| "deny".to_owned(), |c| c.effective.clone()),
        is_allow: cell.as_ref().is_some_and(|c| c.is_allow),
        layer: cell
            .as_ref()
            .map_or_else(|| "default".to_owned(), |c| c.layer.clone()),
        detail: cell.map(|c| c.detail).unwrap_or_default(),
    }
}

// Why: "Who can see this" for one marketplace: the same resolved answer as the
// matrix, sliced to a single column.
pub(super) async fn audience_for(
    pool: &PgPool,
    manifests: &[MarketplaceConfigSummary],
    marketplace_id: &str,
    roles: &[String],
) -> (Vec<AudienceSubjectView>, Vec<AudienceSubjectView>) {
    let matrix = audience_matrix(pool, manifests, roles).await;
    let mut group_rows = Vec::new();
    let mut role_rows = Vec::new();
    for row in matrix.rows {
        let kind = row.kind;
        let view = subject_view(row, marketplace_id);
        if kind == "group" {
            group_rows.push(view);
        } else {
            role_rows.push(view);
        }
    }
    (group_rows, role_rows)
}

// Why: Which groups hold an explicit `allow` rule on each marketplace.
//
// This is the *declared* half of entitlement — the rule a person clicked into
// existence — and it is what the assign toggle reflects. The resolved half
// (what the policy chain actually decides) is [`audience_for`], and the two
// are shown side by side because a deny written elsewhere can close a
// marketplace this map says is open.
pub(super) async fn group_grants(pool: &PgPool) -> std::collections::HashMap<String, Vec<String>> {
    let rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: rule listing failed"))
        .unwrap_or_default();
    let mut out: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for rule in rules {
        if rule.entity_type == MARKETPLACE_ENTITY
            && rule.rule_type.as_str() == "group"
            && rule.access.to_string() == "allow"
        {
            out.entry(rule.entity_id).or_default().push(rule.rule_value);
        }
    }
    for ids in out.values_mut() {
        ids.sort();
    }
    out
}

// Why: Every group, paired with the grant it declares and the verdict it gets.
pub(super) async fn group_assignments(
    pool: &PgPool,
    marketplace_id: &str,
    resolved: &[AudienceSubjectView],
) -> Vec<super::view::GroupAssignmentView> {
    let granted = group_grants(pool)
        .await
        .remove(marketplace_id)
        .unwrap_or_default();
    repositories::groups::crud::list_group_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: group listing failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|g| {
            let verdict = resolved
                .iter()
                .find(|r| r.subject == format!("group:{}", g.id));
            super::view::GroupAssignmentView {
                assigned: granted.contains(&g.id),
                resolved_allow: verdict.is_some_and(|r| r.is_allow),
                resolved_layer: verdict.map(|r| r.layer.clone()).unwrap_or_default(),
                detail_url: format!("/admin/groups/{}", g.id),
                id: g.id,
                name: g.name,
                member_count: g.member_count,
            }
        })
        .collect()
}
