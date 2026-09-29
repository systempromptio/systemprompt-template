//! The audience grid: every entity down the side, every role, group and
//! project across the top, and in each cell what the resolver decides for a
//! subject that holds exactly that one thing — decision first, with the band
//! that decided as the small word underneath.
//!
//! It is the picture the rule list cannot give: who reaches what, at a
//! glance, straight from the enforcement path rather than from a reading of
//! the rules. Cells are resolved in one batch so the page reads the rule
//! table once, however many columns it draws. Narrowing (`audience_filter`)
//! and the inspector for one subject or one entity (`audience_focus`) read
//! the same resolved cells, so a filtered grid and its inspector never
//! disagree.

use std::collections::HashMap;

use sqlx::PgPool;

use super::audience_filter::{self, Narrowed};
use super::audience_focus;
use super::entities::AcQuery;
use super::view::{
    AcAudienceCellView, AcAudienceColumnGroupView, AcAudienceColumnView, AcAudienceLegendView,
    AcAudienceRowGroupView, AcAudienceRowView, AudienceGridView,
};
use crate::handlers::ssr::entity_kind::{entity_kind_label, entity_kind_plural, entity_kind_rank};
use crate::repositories;
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::repositories::users::access_control::{
    MatrixSubject, SectionInput, group_subject, project_subject, resolve_subject_matrices,
    role_subject,
};
use crate::types::Role;

pub(super) const AUDIENCE_URL: &str = "/admin/access-control?tab=audience";

pub(super) struct Audience {
    pub subjects: Vec<MatrixSubject>,
    pub columns: Vec<AcAudienceColumnView>,
}

pub(super) fn subject_kind_label(kind: &str) -> &'static str {
    match kind {
        "role" => "Roles",
        "group" => "Groups",
        "project" => "Projects",
        _ => "Subjects",
    }
}

pub(super) fn audience_url(params: &[(&str, &str)]) -> String {
    let mut url = AUDIENCE_URL.to_owned();
    for (key, value) in params {
        url.push('&');
        url.push_str(key);
        url.push('=');
        url.push_str(&urlencoding::encode(value));
    }
    url
}

fn column(subject: &str, label: String, kind: &'static str) -> AcAudienceColumnView {
    let id = format!("{kind}:{subject}");
    AcAudienceColumnView {
        focus_url: audience_url(&[("subject", &id)]),
        id,
        subject: subject.to_owned(),
        label,
        kind,
        kind_label: subject_kind_label(kind),
        is_focused: false,
        allowed: 0,
        denied: 0,
    }
}

// Why: roles first as the baseline every person carries, then groups (where
// entitlement is granted here), then projects (the narrowest band).
pub(super) async fn audience(pool: &PgPool) -> Audience {
    let mut subjects = Vec::new();
    let mut columns = Vec::new();
    for role in Role::ALL {
        subjects.push(role_subject(role.as_str()));
        columns.push(column(role.as_str(), role.as_str().to_owned(), "role"));
    }
    let groups = repositories::groups::crud::list_group_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: group listing failed"))
        .unwrap_or_default();
    for group in groups {
        subjects.push(group_subject(group.id.as_str()));
        columns.push(column(group.id.as_str(), group.name, "group"));
    }
    let projects = repositories::projects::crud::list_project_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: project listing failed"))
        .unwrap_or_default();
    for project in projects {
        subjects.push(project_subject(project.id.as_str()));
        columns.push(column(project.id.as_str(), project.name, "project"));
    }
    Audience { subjects, columns }
}

// Why: the resolver reports both outcomes of the default as the same layer;
// to a reader "✕ open" is a contradiction, so a refusal by default is worded
// as the closed default it is.
pub(super) fn short_layer(layer: &str, effective: &str) -> String {
    match layer {
        "user" => "person".to_owned(),
        "default" | "default_included" if effective == "allow" => "open".to_owned(),
        "default" | "default_included" => "closed".to_owned(),
        other => other.replace('_', " "),
    }
}

// Why: the inverse of `short_layer`, so an inspector row can find the ledger
// rule that produced a cell by the ledger's own band name.
pub(super) fn rule_type_of(short: &str) -> String {
    match short {
        "person" => "user".to_owned(),
        "open" | "closed" => "default".to_owned(),
        other => other.replace(' ', "_"),
    }
}

pub(super) fn decision_of(effective: &str) -> (&'static str, &'static str, &'static str) {
    match effective {
        "allow" => ("allow", "✓", "ok"),
        "warn" => ("warn", "⚠", "warn"),
        "pending" => ("pending", "◌", "info"),
        _ => ("deny", "✕", "err"),
    }
}

pub(super) fn legend() -> Vec<AcAudienceLegendView> {
    [
        ("allow", "Allowed"),
        ("deny", "Denied"),
        ("warn", "Warn"),
        ("pending", "Pending approval"),
    ]
    .into_iter()
    .map(|(decision, label)| {
        let (decision, glyph, tone) = decision_of(decision);
        AcAudienceLegendView {
            decision,
            glyph,
            tone,
            label,
        }
    })
    .collect()
}

fn resolved_rows(
    resolved: &[Vec<repositories::users::access_control::MatrixSection>],
    columns: &[AcAudienceColumnView],
    sections: &[SectionInput],
) -> Vec<AcAudienceRowView> {
    // Why: the resolver answers per subject; the grid reads per entity, so the
    // answers are transposed through one map keyed on the entity.
    let mut cells: HashMap<(String, String), Vec<AcAudienceCellView>> = HashMap::new();
    for (matrix, col) in resolved.iter().zip(columns) {
        for section in matrix {
            for row in &section.rows {
                let (decision, glyph, tone) = decision_of(&row.effective);
                cells
                    .entry((section.entity_type.clone(), row.entity_id.clone()))
                    .or_default()
                    .push(AcAudienceCellView {
                        decision,
                        glyph,
                        tone,
                        layer: short_layer(&row.source.layer, &row.effective),
                        detail: row.source.detail.clone(),
                        subject_id: col.id.clone(),
                    });
            }
        }
    }
    let mut rows = Vec::new();
    for (entity_type, _, entities) in sections {
        for (entity_id, name, description) in entities {
            let Some(cells) = cells.remove(&(entity_type.clone(), entity_id.clone())) else {
                continue;
            };
            let entity_ref = format!("{entity_type}/{entity_id}");
            rows.push(AcAudienceRowView {
                entity_type_label: entity_kind_label(entity_type),
                entity_type: entity_type.clone(),
                entity_id: entity_id.clone(),
                entity_name: name.clone(),
                // Why: the matrix section carries a route's "pattern →
                // provider" line as its description; other kinds' prose
                // descriptions are too long for a grid row.
                entity_sub: if entity_type == "gateway_route" {
                    description.clone()
                } else {
                    None
                },
                drill_url: audience_url(&[("entity", &entity_ref)]),
                allowed: 0,
                denied: 0,
                cells,
            });
        }
    }
    rows
}

fn column_groups(columns: &[AcAudienceColumnView]) -> Vec<AcAudienceColumnGroupView> {
    let mut groups: Vec<AcAudienceColumnGroupView> = Vec::new();
    for col in columns {
        match groups.last_mut() {
            Some(g) if g.kind == col.kind => g.span += 1,
            _ => groups.push(AcAudienceColumnGroupView {
                kind: col.kind,
                kind_label: col.kind_label,
                span: 1,
            }),
        }
    }
    groups
}

fn row_groups(rows: Vec<AcAudienceRowView>) -> Vec<AcAudienceRowGroupView> {
    let mut groups: Vec<AcAudienceRowGroupView> = Vec::new();
    for row in rows {
        match groups.iter_mut().find(|g| g.kind == row.entity_type) {
            Some(g) => g.rows.push(row),
            None => groups.push(AcAudienceRowGroupView {
                kind_label: entity_kind_plural(&row.entity_type),
                kind: row.entity_type.clone(),
                count: 0,
                rows: vec![row],
            }),
        }
    }
    groups.sort_by_key(|g| entity_kind_rank(&g.kind));
    for g in &mut groups {
        g.rows.sort_by(|a, b| {
            a.entity_name
                .to_lowercase()
                .cmp(&b.entity_name.to_lowercase())
        });
        g.count = g.rows.len();
    }
    groups
}

pub(super) async fn grid(
    pool: &PgPool,
    audience: Audience,
    sections: &[SectionInput],
    query: &AcQuery,
    ledger: &[LedgerRuleRow],
) -> AudienceGridView {
    if audience.subjects.is_empty() || sections.is_empty() {
        return AudienceGridView::default();
    }
    let resolved = resolve_subject_matrices(pool, &audience.subjects, sections)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: audience grid failed"))
        .unwrap_or_default();
    if resolved.len() != audience.subjects.len() {
        return AudienceGridView::default();
    }
    let all_rows = resolved_rows(&resolved, &audience.columns, sections);
    let filters = audience_filter::filters(&audience.columns, &all_rows, query);
    let Narrowed { columns, rows } = audience_filter::narrow(audience.columns, all_rows, query);
    let focus = audience_focus::build(&columns, &rows, query, ledger);
    let kpis = audience_focus::kpis(&columns, &rows, query);
    AudienceGridView {
        column_groups: column_groups(&columns),
        has_rows: !rows.is_empty(),
        resolved: true,
        total_rows: rows.len(),
        total_columns: columns.len(),
        colspan: columns.len() + 1,
        row_groups: row_groups(rows),
        columns,
        kpis,
        legend: legend(),
        filters,
        focus,
    }
}
