//! "Reaches: …" — who the resolver lets in, and the sentence that says so.
//!
//! Groups, projects and roles are asked of the resolver one subject at a
//! time, through the same batch the audience matrix uses, so an entity a
//! plugin inherits from its marketplace reads as reached here too. Bands the
//! resolver cannot be asked about without a real account — a person, a
//! connected server, a linked org — are listed from their allow rules.

use sqlx::PgPool;

use super::view::{OptionView, ReachView, SubjectOptions};
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::repositories::sync::access_control_rows::band_label;
use crate::repositories::users::access_control::{
    MatrixSubject, SectionInput, group_subject, project_subject, resolve_subject_matrices,
    role_subject,
};
use crate::types::access_control::AccessDecision;

const RESOLVED_BANDS: [&str; 3] = ["group", "project", "role"];

pub(super) async fn resolved_reach(
    pool: &PgPool,
    entity: (&str, &str),
    subjects: &SubjectOptions,
) -> Vec<ReachView> {
    let mut asked: Vec<(MatrixSubject, &OptionView, &'static str)> = Vec::new();
    for g in &subjects.groups {
        asked.push((group_subject(&g.value), g, "group"));
    }
    for p in &subjects.projects {
        asked.push((project_subject(&p.value), p, "project"));
    }
    for r in &subjects.roles {
        asked.push((role_subject(&r.value), r, "role"));
    }
    let only: Vec<MatrixSubject> = asked.iter().map(|(s, _, _)| s.clone()).collect();
    let section: SectionInput = (
        entity.0.to_owned(),
        entity.0.to_owned(),
        vec![(entity.1.to_owned(), entity.1.to_owned(), None)],
    );
    let resolved = resolve_subject_matrices(pool, &only, &[section])
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity panel: reach failed to resolve"))
        .unwrap_or_default();
    asked
        .iter()
        .zip(resolved)
        .filter(|(_, sections)| {
            sections
                .iter()
                .flat_map(|s| s.rows.iter())
                .any(|row| row.effective == "allow")
        })
        .map(|((_, option, band), _)| ReachView {
            label: option.label.clone(),
            band_label: band_label(band),
        })
        .collect()
}

pub(super) fn ruled_reach(rows: &[&LedgerRuleRow]) -> Vec<ReachView> {
    rows.iter()
        .filter(|r| {
            r.access == AccessDecision::Allow && !RESOLVED_BANDS.contains(&r.rule_type.as_str())
        })
        .map(|r| ReachView {
            label: r.rule_value.clone(),
            band_label: band_label(&r.rule_type),
        })
        .collect()
}

#[must_use]
pub(super) fn headline(open: bool, reaches: &[ReachView]) -> String {
    if open {
        return "Open by default — everyone reaches this unless a rule denies them.".to_owned();
    }
    if reaches.is_empty() {
        return "Nobody reaches this. It is closed by default and no rule allows anyone."
            .to_owned();
    }
    let names: Vec<String> = reaches
        .iter()
        .map(|r| format!("{} ({})", r.label, r.band_label))
        .collect();
    format!("Closed by default. Reaches: {}.", names.join(", "))
}
