//! The Evaluation view: how well a marketplace's skills ran, version by
//! version. Every figure is an aggregate of `PluginEvalRow`, the same rows
//! the `analysis-plugin-eval` export returns, so the page and a downloaded
//! file always agree. A version is compared with the one before it that has
//! conversations, skill by skill.

mod figures;
mod insights;

use std::collections::BTreeMap;

use serde::Serialize;

use super::history::{VersionView, short};
use crate::repositories::analysis::plugin_eval::{PluginEvalRow, PluginEvalToolRow};
use figures::{Figures, Summary, summary};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VersionEval {
    pub hash: String,
    pub hash_short: String,
    pub declared_version: Option<String>,
    pub first_seen_at: String,
    pub is_current: bool,
    pub is_selected: bool,
    pub is_baseline: bool,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillEval {
    pub skill: String,
    pub summary: Summary,
    pub last_context_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EvaluationView {
    pub versions: Vec<VersionEval>,
    pub selected: Option<VersionEval>,
    pub baseline: Option<VersionEval>,
    pub skills: Vec<SkillEval>,
    pub plugins: Vec<insights::PluginRollup>,
    pub days: Vec<insights::DayEval>,
    pub tools: Vec<insights::ToolEval>,
    pub attention: Vec<insights::Attention>,
    pub has_runs: bool,
}

struct Totals<'a> {
    by_version: BTreeMap<&'a str, Figures>,
    by_skill: BTreeMap<(&'a str, &'a str), Figures>,
    last_context: BTreeMap<(&'a str, &'a str), &'a str>,
}

// Why: rows arrive newest first, so the first context seen for a skill is
// its latest run.
fn totals(rows: &[PluginEvalRow]) -> Totals<'_> {
    let mut t = Totals {
        by_version: BTreeMap::new(),
        by_skill: BTreeMap::new(),
        last_context: BTreeMap::new(),
    };
    for r in rows {
        t.by_version.entry(&r.marketplace_hash).or_default().add(r);
        let key = (r.marketplace_hash.as_str(), r.skill.as_str());
        t.by_skill.entry(key).or_default().add(r);
        t.last_context.entry(key).or_insert(r.context_id.as_str());
    }
    t
}

// Why: `b` names the version to read and `a` its baseline. With neither, the
// newest version with conversations is read against the one before it that
// has conversations, which is the question a reader arrives with: did the
// latest change help.
fn pick(
    with_runs: &[&VersionView],
    (a, b): (Option<String>, Option<String>),
) -> (Option<String>, Option<String>) {
    let selected = b
        .as_deref()
        .and_then(|h| with_runs.iter().find(|v| v.hash == h))
        .or_else(|| with_runs.first())
        .map(|v| v.hash.clone());
    let baseline = a
        .as_deref()
        .and_then(|h| with_runs.iter().find(|v| v.hash == h))
        .map(|v| v.hash.clone())
        .or_else(|| {
            let pos = with_runs
                .iter()
                .position(|v| Some(&v.hash) == selected.as_ref())?;
            with_runs.get(pos + 1).map(|v| v.hash.clone())
        });
    (selected, baseline)
}

// Why: oldest first, so the table reads as the plugin's history; each
// version is compared with the one before it that had conversations.
fn version_rows(
    with_runs: &[&VersionView],
    t: &Totals<'_>,
    (selected, baseline): (Option<&str>, Option<&str>),
) -> Vec<VersionEval> {
    let mut versions = Vec::with_capacity(with_runs.len());
    let mut previous: Option<&Figures> = None;
    for v in with_runs.iter().rev() {
        let Some(f) = t.by_version.get(v.hash.as_str()) else {
            continue;
        };
        versions.push(VersionEval {
            hash: v.hash.clone(),
            hash_short: short(&v.hash),
            declared_version: v.declared_version.clone(),
            first_seen_at: v.first_seen_at.clone(),
            is_current: v.is_current,
            is_selected: selected == Some(v.hash.as_str()),
            is_baseline: baseline == Some(v.hash.as_str()),
            summary: summary(f, previous),
        });
        previous = Some(f);
    }
    versions
}

fn skill_rows(t: &Totals<'_>, selected: &str, baseline: Option<&str>) -> Vec<SkillEval> {
    t.by_skill
        .iter()
        .filter(|((h, _), _)| *h == selected)
        .map(|((_, skill), f)| SkillEval {
            skill: (*skill).to_owned(),
            summary: summary(f, baseline.and_then(|b| t.by_skill.get(&(b, *skill)))),
            last_context_id: t
                .last_context
                .get(&(selected, *skill))
                .map(|c| (*c).to_owned()),
        })
        .collect()
}

pub(crate) fn build(
    history: &[VersionView],
    (rows, tool_rows): (&[PluginEvalRow], &[PluginEvalToolRow]),
    wanted: (Option<String>, Option<String>),
) -> EvaluationView {
    let t = totals(rows);
    let with_runs: Vec<&VersionView> = history
        .iter()
        .filter(|v| t.by_version.contains_key(v.hash.as_str()))
        .collect();
    let (selected, baseline) = pick(&with_runs, wanted);
    let (sel, base) = (selected.as_deref(), baseline.as_deref());
    let versions = version_rows(&with_runs, &t, (sel, base));
    // Why: the headline reads the selected version against the chosen
    // baseline, which need not be the version immediately before it.
    let selected_view = versions
        .iter()
        .find(|v| v.is_selected)
        .cloned()
        .map(|mut v| {
            if let Some(f) = sel.and_then(|h| t.by_version.get(h)) {
                v.summary = summary(f, base.and_then(|h| t.by_version.get(h)));
            }
            v
        });
    let of = |h: Option<&str>| -> Vec<&PluginEvalRow> {
        rows.iter()
            .filter(|r| Some(r.marketplace_hash.as_str()) == h)
            .collect()
    };
    let (sel_rows, base_rows) = (of(sel), of(base));
    let tools = insights::tools(
        &tool_rows
            .iter()
            .filter(|t| Some(t.marketplace_hash.as_str()) == sel)
            .collect::<Vec<_>>(),
    );
    EvaluationView {
        attention: insights::attention(&sel_rows, &base_rows, &tools),
        plugins: insights::plugins(&sel_rows, &base_rows),
        days: insights::days(&sel_rows),
        tools,
        has_runs: !rows.is_empty(),
        selected: selected_view,
        baseline: versions.iter().find(|v| v.is_baseline).cloned(),
        skills: sel.map_or_else(Vec::new, |s| skill_rows(&t, s, base)),
        versions,
    }
}
