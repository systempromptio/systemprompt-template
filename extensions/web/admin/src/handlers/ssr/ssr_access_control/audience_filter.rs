//! Narrowing the audience grid by query — the same `<form method="get">`
//! mechanism the Rules tab uses, applied to both axes at once.
//!
//! Columns are narrowed first (subject kind, one subject in focus, search),
//! then rows (entity kind, one entity in drill, search), and only then the
//! cross-axis filters: a decision or band keeps a row if any *visible* cell
//! carries it, and drops a column none of the remaining rows use it in. The
//! counts on each header are of the cells left on screen, so they always
//! agree with what the eye can check.

use super::audience::{AUDIENCE_URL, subject_kind_label};
use super::bands::band_label;
use super::entities::AcQuery;
use super::summary::options;
use super::view::{AcAudienceColumnView, AcAudienceFiltersView, AcAudienceRowView};
use crate::handlers::ssr::entity_kind::{entity_kind_plural, entity_kind_rank};

pub(super) struct Narrowed {
    pub columns: Vec<AcAudienceColumnView>,
    pub rows: Vec<AcAudienceRowView>,
}

fn contains(hay: &str, needle: &str) -> bool {
    hay.to_lowercase().contains(needle)
}

fn column_matches(col: &AcAudienceColumnView, query: &AcQuery) -> bool {
    if let Some(kind) = AcQuery::pick(query.subject_kind.as_ref())
        && col.kind != kind
    {
        return false;
    }
    if let Some(subject) = AcQuery::pick(query.subject.as_ref())
        && col.id != subject
    {
        return false;
    }
    true
}

fn row_matches(row: &AcAudienceRowView, query: &AcQuery, needle: Option<&str>) -> bool {
    if let Some(kind) = AcQuery::pick(query.entity_kind.as_ref())
        && row.entity_type != kind
    {
        return false;
    }
    if let Some(entity) = AcQuery::pick(query.entity.as_ref())
        && format!("{}/{}", row.entity_type, row.entity_id) != entity
    {
        return false;
    }
    if let Some(needle) = needle {
        let hay = format!(
            "{} {} {}",
            row.entity_id,
            row.entity_name,
            row.entity_sub.as_deref().unwrap_or_default()
        );
        if !contains(&hay, needle) {
            return false;
        }
    }
    true
}

fn cell_matches(
    decision: Option<&str>,
    band: Option<&str>,
    cell_decision: &str,
    layer: &str,
) -> bool {
    decision.is_none_or(|d| d == cell_decision) && band.is_none_or(|b| b == layer)
}

fn count(row: &mut AcAudienceRowView) {
    row.allowed = row.cells.iter().filter(|c| c.decision == "allow").count();
    row.denied = row.cells.iter().filter(|c| c.decision == "deny").count();
}

pub(super) fn narrow(
    columns: Vec<AcAudienceColumnView>,
    rows: Vec<AcAudienceRowView>,
    query: &AcQuery,
) -> Narrowed {
    let needle = AcQuery::pick(query.q.as_ref()).map(str::to_lowercase);
    let needle = needle.as_deref();
    let focused = AcQuery::pick(query.subject.as_ref());
    let decision = AcQuery::pick(query.decision.as_ref());
    let band = AcQuery::pick(query.band.as_ref());

    let mut columns: Vec<AcAudienceColumnView> = columns
        .into_iter()
        .filter(|c| column_matches(c, query))
        .collect();
    let keep: Vec<String> = columns.iter().map(|c| c.id.clone()).collect();

    let mut rows: Vec<AcAudienceRowView> = rows
        .into_iter()
        .filter(|r| row_matches(r, query, needle))
        .map(|mut r| {
            r.cells.retain(|c| keep.contains(&c.subject_id));
            r
        })
        .filter(|r| {
            r.cells
                .iter()
                .any(|c| cell_matches(decision, band, c.decision, &c.layer))
        })
        .collect();

    // Why: with a decision or band applied, a column no visible row carries
    // it in is noise — every cell in it would be the other kind.
    if decision.is_some() || band.is_some() {
        columns.retain(|col| {
            rows.iter().any(|r| {
                r.cells.iter().any(|c| {
                    c.subject_id == col.id && cell_matches(decision, band, c.decision, &c.layer)
                })
            })
        });
        let keep: Vec<String> = columns.iter().map(|c| c.id.clone()).collect();
        for row in &mut rows {
            row.cells.retain(|c| keep.contains(&c.subject_id));
        }
    }

    for row in &mut rows {
        count(row);
    }
    for col in &mut columns {
        col.is_focused = focused == Some(col.id.as_str());
        col.allowed = rows
            .iter()
            .flat_map(|r| &r.cells)
            .filter(|c| c.subject_id == col.id && c.decision == "allow")
            .count();
        col.denied = rows
            .iter()
            .flat_map(|r| &r.cells)
            .filter(|c| c.subject_id == col.id && c.decision == "deny")
            .count();
    }
    Narrowed { columns, rows }
}

pub(super) fn filters(
    columns: &[AcAudienceColumnView],
    rows: &[AcAudienceRowView],
    query: &AcQuery,
) -> AcAudienceFiltersView {
    let mut subject_kinds: Vec<&str> = columns.iter().map(|c| c.kind).collect();
    subject_kinds.dedup();
    let mut kinds: Vec<String> = rows.iter().map(|r| r.entity_type.clone()).collect();
    kinds.sort_by_key(|k| entity_kind_rank(k));
    kinds.dedup();
    let mut bands: Vec<String> = rows
        .iter()
        .flat_map(|r| r.cells.iter().map(|c| c.layer.clone()))
        .collect();
    bands.sort();
    bands.dedup();
    AcAudienceFiltersView {
        subject_kind_options: options(
            subject_kinds
                .into_iter()
                .map(|k| (k.to_owned(), subject_kind_label(k).to_owned()))
                .collect(),
            "All subject kinds",
            AcQuery::pick(query.subject_kind.as_ref()),
        ),
        subject_options: options(
            columns
                .iter()
                .map(|c| (c.id.clone(), format!("{} · {}", c.label, c.kind)))
                .collect(),
            "Every subject",
            AcQuery::pick(query.subject.as_ref()),
        ),
        entity_options: options(
            kinds
                .into_iter()
                .map(|k| (k.clone(), entity_kind_plural(&k).to_owned()))
                .collect(),
            "All entity kinds",
            AcQuery::pick(query.entity_kind.as_ref()),
        ),
        decision_options: options(
            vec![
                ("allow".to_owned(), "Allowed somewhere".to_owned()),
                ("deny".to_owned(), "Denied somewhere".to_owned()),
                ("warn".to_owned(), "Warn somewhere".to_owned()),
                ("pending".to_owned(), "Pending somewhere".to_owned()),
            ],
            "Any decision",
            AcQuery::pick(query.decision.as_ref()),
        ),
        band_options: options(
            bands
                .into_iter()
                .map(|b| {
                    let label = band_label(&super::audience::rule_type_of(&b));
                    let label = if label == "band" {
                        b.clone()
                    } else {
                        label.to_owned()
                    };
                    (b, label)
                })
                .collect(),
            "Decided by any band",
            AcQuery::pick(query.band.as_ref()),
        ),
        search: AcQuery::pick(query.q.as_ref())
            .unwrap_or_default()
            .to_owned(),
        filters_applied: query.audience_applied(),
        clear_url: AUDIENCE_URL,
    }
}
