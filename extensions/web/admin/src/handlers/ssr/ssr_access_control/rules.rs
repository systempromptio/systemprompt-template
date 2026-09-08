//! The rule ledger the access-control page renders above its editor.
//!
//! The editor answers "what may this person reach"; the ledger answers the
//! question an auditor asks instead — what rules exist at all, at which
//! subject band, and which of them this instance's database holds without the
//! YAML in the source repository declaring them. That last column is the one
//! an operator cannot get anywhere else: a rule outside the declarative set
//! disappears the next time the estate is rebuilt from source.
//!
//! The query string, the filter menus and the totals are
//! [`super::rules_controls`].

use serde::Serialize;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::repositories::access_control::yaml_declared::DeclaredRules;

use super::rules_controls::{
    AcKpiView, AcOptionView, BASE_URL, MANUAL, PAGE_SIZE, RulesQuery, YAML, distinct, kpis,
    options, sort_headers,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcRuleView {
    pub entity_type: String,
    pub entity_type_label: String,
    pub entity_id: String,
    pub subject_kind: String,
    pub subject: String,
    pub access: String,
    pub access_tone: &'static str,
    pub source: &'static str,
    pub source_tone: &'static str,
    pub default_label: &'static str,
    pub justification: String,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct AcRulesView {
    pub rows: Vec<AcRuleView>,
    pub total: i64,
    pub pagination: Option<Pagination>,
    pub kpis: Vec<AcKpiView>,
    pub sort_headers: Vec<SortHeaderView>,
    pub subject_options: Vec<AcOptionView>,
    pub entity_options: Vec<AcOptionView>,
    pub source_options: Vec<AcOptionView>,
    pub search: String,
    pub filters_applied: bool,
    pub clear_url: &'static str,
    pub capped: bool,
}

fn source_of(row: &LedgerRuleRow, declared: &DeclaredRules) -> &'static str {
    if declared.declares(
        &row.entity_type,
        &row.entity_id,
        &row.rule_type,
        &row.rule_value,
    ) {
        YAML
    } else {
        MANUAL
    }
}

fn matches(row: &LedgerRuleRow, source: &str, query: &RulesQuery) -> bool {
    if let Some(kind) = query.selected("subject_kind")
        && row.rule_type != kind
    {
        return false;
    }
    if let Some(kind) = query.selected("entity_kind")
        && row.entity_type != kind
    {
        return false;
    }
    if let Some(want) = query.selected("source") {
        let is_yaml = source == YAML;
        if is_yaml != (want == "yaml") {
            return false;
        }
    }
    if let Some(needle) = query.selected("q") {
        let needle = needle.to_lowercase();
        let haystack = format!("{} {}", row.entity_id, row.rule_value).to_lowercase();
        if !haystack.contains(&needle) {
            return false;
        }
    }
    true
}

// Why: Build the whole ledger view from one read plus the declarative set.
#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub(super) fn build(
    rows: &[LedgerRuleRow],
    declared: &DeclaredRules,
    open_entities: i64,
    query: &RulesQuery,
    capped: bool,
) -> AcRulesView {
    let sources: Vec<&str> = rows.iter().map(|r| source_of(r, declared)).collect();
    let mut matching: Vec<(&LedgerRuleRow, &str)> = rows
        .iter()
        .zip(sources.iter().copied())
        .filter(|(row, source)| matches(row, source, query))
        .collect();

    match query.sort_key() {
        "entity" => matching.sort_by(|a, b| a.0.entity_id.cmp(&b.0.entity_id)),
        "subject_kind" => matching.sort_by(|a, b| a.0.rule_type.cmp(&b.0.rule_type)),
        "subject" => matching.sort_by(|a, b| a.0.rule_value.cmp(&b.0.rule_value)),
        "access" => matching.sort_by_key(|(row, _)| row.access.to_string()),
        "source" => matching.sort_by_key(|(_, source)| *source),
        _ => matching.sort_by(|a, b| a.0.entity_type.cmp(&b.0.entity_type)),
    }
    if query.descending() {
        matching.reverse();
    }

    let total = i64::try_from(matching.len()).unwrap_or(i64::MAX);
    let last_page = (total.max(1) - 1) / PAGE_SIZE;
    let index = query.page.unwrap_or(0).clamp(0, last_page);
    let start = usize::try_from(index * PAGE_SIZE).unwrap_or(0);
    let page: Vec<AcRuleView> = matching
        .iter()
        .skip(start)
        .take(usize::try_from(PAGE_SIZE).unwrap_or(50))
        .map(|(row, source)| AcRuleView {
            entity_type_label: row.entity_type.replace('_', " "),
            entity_type: row.entity_type.clone(),
            entity_id: row.entity_id.clone(),
            subject_kind: row.rule_type.clone(),
            subject: row.rule_value.clone(),
            access: row.access.to_string(),
            access_tone: match row.access {
                crate::types::access_control::AccessDecision::Allow => "ok",
                crate::types::access_control::AccessDecision::Deny => "err",
            },
            source,
            source_tone: if *source == YAML { "muted" } else { "warn" },
            default_label: if row.default_included {
                "open"
            } else {
                "closed"
            },
            justification: row.justification.clone().unwrap_or_default(),
        })
        .collect();

    let window = PageWindow::new(
        index,
        PAGE_SIZE,
        total,
        i64::try_from(page.len()).unwrap_or(0),
        "rules",
    );
    let (first_row, last_row) = window.bounds();
    let prev_url = (index > 0).then(|| query.url_with(&[("page", &(index - 1).to_string())]));
    let next_url = (index + 1 < window.total_pages)
        .then(|| query.url_with(&[("page", &(index + 1).to_string())]));

    AcRulesView {
        rows: page,
        total,
        pagination: Some(Pagination {
            current_page: index + 1,
            total_pages: window.total_pages,
            first_row,
            last_row,
            total_rows: total,
            noun: window.noun,
            has_prev: prev_url.is_some(),
            has_next: next_url.is_some(),
            prev_url,
            next_url,
        }),
        kpis: kpis(rows, &sources, open_entities),
        sort_headers: sort_headers(query),
        subject_options: options(
            &distinct(rows, |r| &r.rule_type),
            "All subject kinds",
            query.selected("subject_kind"),
        ),
        entity_options: options(
            &distinct(rows, |r| &r.entity_type),
            "All entity kinds",
            query.selected("entity_kind"),
        ),
        source_options: options(
            &[
                ("yaml".to_owned(), "Declared in YAML".to_owned()),
                ("manual".to_owned(), "Only in this database".to_owned()),
            ],
            "Any rule source",
            query.selected("source"),
        ),
        search: query.selected("q").unwrap_or_default().to_owned(),
        filters_applied: query.any_applied(),
        clear_url: BASE_URL,
        capped,
    }
}
