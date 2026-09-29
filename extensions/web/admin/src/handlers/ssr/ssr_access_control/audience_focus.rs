//! The inspector beside the audience grid: one subject in focus lists every
//! entity it reaches or is refused, one entity in drill lists every subject —
//! both bucketed by decision, each line naming the rule that decided and the
//! `why` the ledger recorded for it.
//!
//! Buckets are built from the cells already narrowed for the grid, so the
//! inspector composes with the other filters ("skills the uk group is denied")
//! and never shows a line the grid does not.

use chrono::Utc;

use super::audience::{audience_url, decision_of, rule_type_of};
use super::entities::{AcQuery, BASE_URL};
use super::view::{
    AcAudienceBucketItemView, AcAudienceBucketView, AcAudienceCellView, AcAudienceColumnView,
    AcAudienceFocusView, AcAudienceKpiView, AcAudienceRowView,
};
use crate::handlers::ssr::people_view::expires_soon;
use crate::repositories::access_control::rules::LedgerRuleRow;

const BUCKETS: [(&str, &str); 4] = [
    ("allow", "Allowed"),
    ("warn", "Warn"),
    ("pending", "Pending approval"),
    ("deny", "Denied"),
];

fn rules_url(row: &AcAudienceRowView) -> String {
    format!(
        "{BASE_URL}?entity_kind={}&q={}",
        urlencoding::encode(&row.entity_type),
        urlencoding::encode(&row.entity_id)
    )
}

// Why: a cell names the band and the subject it matched; the ledger row
// behind it is the one on the same entity with that band and subject, and
// only it carries the operator's `why` and the expiry.
fn ledger_rule<'a>(
    ledger: &'a [LedgerRuleRow],
    row: &AcAudienceRowView,
    cell: &AcAudienceCellView,
    subject: &str,
) -> Option<&'a LedgerRuleRow> {
    let rule_type = rule_type_of(&cell.layer);
    ledger.iter().find(|r| {
        r.entity_type == row.entity_type
            && r.entity_id == row.entity_id
            && r.rule_type == rule_type
            && r.rule_value == subject
    })
}

// Why: when the default decided, the resolver's detail is a paragraph of
// remediation advice; in a list of eighty such lines the band word already
// says it, so the line states the fact in one sentence.
fn default_detail(layer: &str) -> Option<String> {
    match layer {
        "closed" => Some("no rule names this subject; the entity is closed".to_owned()),
        "open" => Some("no rule refuses this subject; the entity is open".to_owned()),
        _ => None,
    }
}

struct ItemHead {
    label: String,
    note: Option<String>,
    kind_label: &'static str,
}

fn item(
    head: ItemHead,
    row: &AcAudienceRowView,
    cell: &AcAudienceCellView,
    rule: Option<&LedgerRuleRow>,
) -> AcAudienceBucketItemView {
    let expires = rule.and_then(|r| r.valid_until);
    AcAudienceBucketItemView {
        label: head.label,
        note: head.note,
        kind_label: head.kind_label,
        layer: cell.layer.clone(),
        detail: default_detail(&cell.layer).unwrap_or_else(|| cell.detail.clone()),
        why: rule
            .and_then(|r| r.justification.clone())
            .filter(|w| !w.trim().is_empty()),
        expires_at: expires.map(|t| {
            if expires_soon(Some(t), Utc::now()) {
                format!("{} · soon", t.format("%Y-%m-%d"))
            } else {
                t.format("%Y-%m-%d").to_string()
            }
        }),
        rules_url: rules_url(row),
    }
}

fn buckets(items: &[(&'static str, AcAudienceBucketItemView)]) -> Vec<AcAudienceBucketView> {
    BUCKETS
        .into_iter()
        .map(|(decision, label)| {
            let (decision, glyph, tone) = decision_of(decision);
            AcAudienceBucketView {
                decision,
                glyph,
                tone,
                label,
                count: 0,
                items: Vec::new(),
            }
        })
        .map(|mut bucket| {
            bucket.items = items
                .iter()
                .filter(|(d, _)| *d == bucket.decision)
                .map(|(_, i)| i.clone())
                .collect();
            bucket.count = bucket.items.len();
            bucket
        })
        .filter(|b| b.count > 0)
        .collect()
}

fn subject_focus(
    col: &AcAudienceColumnView,
    rows: &[AcAudienceRowView],
    ledger: &[LedgerRuleRow],
) -> AcAudienceFocusView {
    let items: Vec<(&'static str, AcAudienceBucketItemView)> = rows
        .iter()
        .filter_map(|row| {
            let cell = row.cells.iter().find(|c| c.subject_id == col.id)?;
            let rule = ledger_rule(ledger, row, cell, &col.subject);
            Some((
                cell.decision,
                item(
                    ItemHead {
                        label: row.entity_name.clone(),
                        note: row.entity_sub.clone(),
                        kind_label: row.entity_type_label,
                    },
                    row,
                    cell,
                    rule,
                ),
            ))
        })
        .collect();
    AcAudienceFocusView {
        kind_label: col.kind_label,
        label: col.label.clone(),
        note: Some(col.subject.clone()).filter(|s| *s != col.label),
        back_url: audience_url(&[("subject_kind", col.kind)]),
        item_noun: "entities",
        item_label: "Entity",
        buckets: buckets(&items),
    }
}

fn entity_drill(
    row: &AcAudienceRowView,
    columns: &[AcAudienceColumnView],
    ledger: &[LedgerRuleRow],
) -> AcAudienceFocusView {
    let items: Vec<(&'static str, AcAudienceBucketItemView)> = columns
        .iter()
        .filter_map(|col| {
            let cell = row.cells.iter().find(|c| c.subject_id == col.id)?;
            let rule = ledger_rule(ledger, row, cell, &col.subject);
            Some((
                cell.decision,
                item(
                    ItemHead {
                        label: col.label.clone(),
                        note: None,
                        kind_label: col.kind_label,
                    },
                    row,
                    cell,
                    rule,
                ),
            ))
        })
        .collect();
    AcAudienceFocusView {
        kind_label: row.entity_type_label,
        label: row.entity_name.clone(),
        note: row
            .entity_sub
            .clone()
            .or_else(|| Some(row.entity_id.clone())),
        back_url: audience_url(&[("entity_kind", &row.entity_type)]),
        item_noun: "subjects",
        item_label: "Subject",
        buckets: buckets(&items),
    }
}

pub(super) fn build(
    columns: &[AcAudienceColumnView],
    rows: &[AcAudienceRowView],
    query: &AcQuery,
    ledger: &[LedgerRuleRow],
) -> Option<AcAudienceFocusView> {
    if let Some(subject) = AcQuery::pick(query.subject.as_ref()) {
        return columns
            .iter()
            .find(|c| c.id == subject)
            .map(|col| subject_focus(col, rows, ledger));
    }
    if let Some(entity) = AcQuery::pick(query.entity.as_ref()) {
        return rows
            .iter()
            .find(|r| format!("{}/{}", r.entity_type, r.entity_id) == entity)
            .map(|row| entity_drill(row, columns, ledger));
    }
    None
}

fn decision_kpi(
    rows: &[AcAudienceRowView],
    query: &AcQuery,
    decision: &'static str,
    label: &'static str,
    tone: &'static str,
) -> AcAudienceKpiView {
    let cells = rows.iter().flat_map(|r| &r.cells);
    let count = cells.filter(|c| c.decision == decision).count();
    let active = AcQuery::pick(query.decision.as_ref()) == Some(decision);
    AcAudienceKpiView {
        label,
        value: count.to_string(),
        note: format!("cells · click to keep rows with a {decision}"),
        tone,
        href: Some(if active {
            audience_url(&[])
        } else {
            audience_url(&[("decision", decision)])
        }),
        active,
    }
}

pub(super) fn kpis(
    columns: &[AcAudienceColumnView],
    rows: &[AcAudienceRowView],
    query: &AcQuery,
) -> Vec<AcAudienceKpiView> {
    let roles = columns.iter().filter(|c| c.kind == "role").count();
    let groups = columns.iter().filter(|c| c.kind == "group").count();
    let projects = columns.iter().filter(|c| c.kind == "project").count();
    vec![
        AcAudienceKpiView {
            label: "Subjects",
            value: columns.len().to_string(),
            note: format!("{roles} roles · {groups} groups · {projects} projects"),
            tone: "accent",
            href: None,
            active: false,
        },
        AcAudienceKpiView {
            label: "Entities",
            value: rows.len().to_string(),
            note: "governed entities on screen".to_owned(),
            tone: "accent",
            href: None,
            active: false,
        },
        decision_kpi(rows, query, "allow", "Allowed", "ok"),
        decision_kpi(rows, query, "deny", "Denied", "err"),
        decision_kpi(rows, query, "warn", "Warn", "warn"),
        decision_kpi(rows, query, "pending", "Pending", "info"),
    ]
}
