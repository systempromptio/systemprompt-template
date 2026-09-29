//! The parts of the Evaluation view that answer "is anything going wrong":
//! a rollup per plugin, the selected version day by day, every tool its
//! skills called, and an attention list drawn by fixed thresholds. Every
//! figure is a sum over the same rows the exports return.

use std::collections::BTreeMap;

use serde::Serialize;
use systemprompt::identifiers::{ContextId, PluginId};

use super::figures::{Figures, Summary, summary};
use crate::repositories::analysis::plugin_eval::{PluginEvalRow, PluginEvalToolRow};

// Why: the thresholds a reader can quote back; changing one changes what the
// page calls a problem, so they live together and the docs list them.
const TOOL_FAILURE_SHARE: f64 = 0.2;
const TOOL_MIN_CALLS: i64 = 3;
const REPEATED_CALLS_PER_RUN: i64 = 3;
const COST_RISE: f64 = 0.25;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PluginRollup {
    pub plugin_id: PluginId,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DayEval {
    pub day: String,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolEval {
    pub skill: String,
    pub tool: String,
    pub calls: i64,
    pub failed_calls: i64,
    pub failure_share: String,
    pub schema_errors: i64,
    pub access_errors: i64,
    pub upstream_errors: i64,
    pub bad_arguments: i64,
    pub timeouts: i64,
    pub repeated_calls: i64,
    pub flagged: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Attention {
    pub tone: &'static str,
    pub title: String,
    pub detail: String,
    pub context_id: Option<ContextId>,
}

pub(super) fn plugins(rows: &[&PluginEvalRow], base: &[&PluginEvalRow]) -> Vec<PluginRollup> {
    let group = |rs: &[&PluginEvalRow]| {
        let mut m: BTreeMap<PluginId, Figures> = BTreeMap::new();
        for r in rs {
            m.entry(r.plugin_id.clone()).or_default().add(r);
        }
        m
    };
    let (cur, prev) = (group(rows), group(base));
    cur.iter()
        .map(|(id, f)| PluginRollup {
            plugin_id: id.clone(),
            summary: summary(f, prev.get(id)),
        })
        .collect()
}

// Why: day by day within the selected version, each day read against the
// day before it, so a bad afternoon shows without waiting for a new version.
pub(super) fn days(rows: &[&PluginEvalRow]) -> Vec<DayEval> {
    let mut by_day: BTreeMap<String, Figures> = BTreeMap::new();
    for r in rows {
        by_day
            .entry(r.first_invoked_at.format("%Y-%m-%d").to_string())
            .or_default()
            .add(r);
    }
    let mut out = Vec::with_capacity(by_day.len());
    let mut previous: Option<&Figures> = None;
    for (day, f) in &by_day {
        out.push(DayEval {
            day: day.clone(),
            summary: summary(f, previous),
        });
        previous = Some(f);
    }
    out.reverse();
    out
}

pub(super) fn tools(rows: &[&PluginEvalToolRow]) -> Vec<ToolEval> {
    let mut out: Vec<ToolEval> = rows
        .iter()
        .map(|t| {
            let share = if t.calls == 0 {
                0.0
            } else {
                t.failed_calls as f64 / t.calls as f64
            };
            ToolEval {
                skill: t.skill.clone(),
                tool: t.tool.rsplit("__").next().unwrap_or(&t.tool).to_owned(),
                calls: t.calls,
                failed_calls: t.failed_calls,
                failure_share: format!("{:.0}%", share * 100.0),
                schema_errors: t.schema_errors,
                access_errors: t.access_errors,
                upstream_errors: t.upstream_errors,
                bad_arguments: t.bad_arguments,
                timeouts: t.timeouts,
                repeated_calls: t.repeated_calls,
                flagged: t.calls >= TOOL_MIN_CALLS && share >= TOOL_FAILURE_SHARE,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.failed_calls
            .cmp(&a.failed_calls)
            .then(b.calls.cmp(&a.calls))
    });
    out
}

fn item(
    tone: &'static str,
    title: String,
    detail: String,
    row: Option<&PluginEvalRow>,
) -> Attention {
    Attention {
        tone,
        title,
        detail,
        context_id: row.map(|r| r.context_id.clone()),
    }
}

fn latest<'a>(
    rows: &[&'a PluginEvalRow],
    skill: &str,
    hit: impl Fn(&PluginEvalRow) -> bool,
) -> Option<&'a PluginEvalRow> {
    rows.iter().copied().find(|r| r.skill == skill && hit(r))
}

// Why: rows arrive newest first, so the first match is the latest
// conversation that shows the problem.
pub(super) fn attention(
    rows: &[&PluginEvalRow],
    base: &[&PluginEvalRow],
    tools: &[ToolEval],
) -> Vec<Attention> {
    let fig = |rs: &[&PluginEvalRow]| {
        let mut m: BTreeMap<String, Figures> = BTreeMap::new();
        for r in rs {
            m.entry(r.skill.clone()).or_default().add(r);
        }
        m
    };
    let (cur, prev) = (fig(rows), fig(base));
    let mut out = skill_items(rows, &cur, &prev);
    for t in tools.iter().filter(|t| t.flagged) {
        out.push(item(
            "warning",
            format!("{} fails in {}", t.tool, t.skill),
            format!(
                "{} of {} calls failed ({})",
                t.failed_calls, t.calls, t.failure_share
            ),
            latest(rows, &t.skill, |r| r.failed_calls > 0),
        ));
    }
    out.sort_by_key(|a| a.tone != "danger");
    out
}

fn skill_items(
    rows: &[&PluginEvalRow],
    cur: &BTreeMap<String, Figures>,
    prev: &BTreeMap<String, Figures>,
) -> Vec<Attention> {
    let mut out = Vec::new();
    for (skill, f) in cur {
        if f.writes > 0 {
            out.push(item(
                "danger",
                format!("{skill} wrote to a connector"),
                format!("{} write calls", f.writes),
                latest(rows, skill, |r| r.writes > 0),
            ));
        }
        if f.tools_unavailable > 0 {
            out.push(item(
                "danger",
                format!("{skill} ran without its tools"),
                format!(
                    "{} of {} runs said a tool or connector was unavailable",
                    f.tools_unavailable, f.runs
                ),
                latest(rows, skill, |r| r.tools_unavailable),
            ));
        }
        if let Some(p) = prev.get(skill) {
            if f.success_rate() + 1e-9 < p.success_rate() {
                out.push(item(
                    "danger",
                    format!("{skill} succeeds less often"),
                    format!("{:.0}% → {:.0}%", p.success_rate(), f.success_rate()),
                    latest(rows, skill, |r| !r.success),
                ));
            }
            if p.avg_cost() > 0.0 && f.avg_cost() >= p.avg_cost() * (1.0 + COST_RISE) {
                out.push(item(
                    "warning",
                    format!("{skill} costs more per run"),
                    format!(
                        "up {:.0}% on the baseline",
                        (f.avg_cost() / p.avg_cost() - 1.0) * 100.0
                    ),
                    None,
                ));
            }
        }
        let loops = rows
            .iter()
            .filter(|r| r.skill == *skill && r.repeated_calls >= REPEATED_CALLS_PER_RUN)
            .count();
        if loops > 0 {
            out.push(item(
                "warning",
                format!("{skill} repeats calls"),
                format!(
                    "{loops} runs re-issued the same call {REPEATED_CALLS_PER_RUN} or more times"
                ),
                latest(rows, skill, |r| r.repeated_calls >= REPEATED_CALLS_PER_RUN),
            ));
        }
    }
    out
}
