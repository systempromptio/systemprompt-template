//! The skill page's Runs sections: each kit release's run record, newest
//! first with the change from the release before it, and the latest runs.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::handlers::ssr::analysis::skill_runs::{
    ReleaseImpact, agent_s, human_wait_s, release_impact, run_success, version_label,
};
use crate::handlers::ssr::analysis_urls::analysis_conversation_url;
use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::repositories::analysis::skills::SkillRunRow;

pub(super) const RUNS_SHOWN: usize = 25;
const LOW_SAMPLE: i64 = 5;

#[derive(Debug, Serialize)]
pub(super) struct ReleaseRowView {
    version: String,
    runs: i64,
    low_sample: bool,
    success_display: String,
    success_tone: &'static str,
    success_change: String,
    median_wall: String,
    wall_change: String,
    p90_wall: String,
    median_agent: String,
    median_wait: String,
    median_prompts: String,
    median_questions: String,
    cost_per_run: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RunRowView {
    href: String,
    context_key: String,
    started_display: String,
    started_full: String,
    version: String,
    wall: String,
    agent: String,
    wait: String,
    prompts: i64,
    questions: i64,
    tool_failures: i64,
    failures_tone: &'static str,
    cost: String,
    state: String,
    phases: String,
    verdict: &'static str,
    verdict_tone: &'static str,
}

fn secs(s: f64) -> String {
    format_duration_ms((s * 1000.0).round() as i64)
}

fn success_tone(rate: Option<f64>) -> &'static str {
    match rate {
        None => "muted",
        Some(r) if r >= 0.8 => "ok",
        Some(r) if r >= 0.5 => "warn",
        Some(_) => "err",
    }
}

fn change(now: f64, before: Option<f64>, unit: impl Fn(f64) -> String) -> String {
    before.map_or_else(String::new, |b| {
        let d = now - b;
        if d.abs() < f64::EPSILON {
            "no change".to_owned()
        } else if d > 0.0 {
            format!("+{}", unit(d))
        } else {
            format!("−{}", unit(-d))
        }
    })
}

fn release_view(r: &ReleaseImpact, before: Option<&ReleaseImpact>) -> ReleaseRowView {
    ReleaseRowView {
        version: r.kit_version.clone(),
        runs: r.runs,
        low_sample: r.runs < LOW_SAMPLE,
        success_display: r
            .success_rate
            .map_or_else(|| "—".to_owned(), |v| format!("{:.0}%", v * 100.0)),
        success_tone: success_tone(r.success_rate),
        success_change: match (r.success_rate, before.and_then(|b| b.success_rate)) {
            (Some(now), Some(b)) => change(now * 100.0, Some(b * 100.0), |d| format!("{d:.0} pts")),
            _ => String::new(),
        },
        median_wall: secs(r.median_wall_s),
        wall_change: change(r.median_wall_s, before.map(|b| b.median_wall_s), secs),
        p90_wall: secs(r.p90_wall_s),
        median_agent: secs(r.median_agent_s),
        median_wait: secs(r.median_wait_s),
        median_prompts: format!("{:.0}", r.median_prompts),
        median_questions: format!("{:.0}", r.median_questions),
        cost_per_run: format_cost(r.cost_per_run_microdollars),
    }
}

pub(super) fn release_views(runs: &[SkillRunRow], now: DateTime<Utc>) -> Vec<ReleaseRowView> {
    let releases = release_impact(runs, false, now);
    releases
        .iter()
        .enumerate()
        .map(|(i, r)| release_view(r, releases.get(i + 1)))
        .collect()
}

fn phases(r: &SkillRunRow) -> String {
    [
        ("req", r.requirements_s),
        ("design", r.design_s),
        ("build", r.build_s),
        ("verify", r.verify_s),
        ("ship", r.ship_s),
    ]
    .iter()
    .filter_map(|(label, s)| s.map(|s| format!("{label} {}", secs(s))))
    .collect::<Vec<_>>()
    .join(" · ")
}

fn run_view(r: &SkillRunRow, now: DateTime<Utc>) -> RunRowView {
    let (verdict, verdict_tone) = match run_success(r, now) {
        Some(true) => ("succeeded", "ok"),
        Some(false) => ("did not finish", "err"),
        None => ("unknown", "muted"),
    };
    RunRowView {
        href: analysis_conversation_url(&r.context_id),
        context_key: r.context_id.as_str().to_owned(),
        started_display: r.started_at.format("%d %b %H:%M").to_string(),
        started_full: r.started_at.to_rfc3339(),
        version: version_label(r),
        wall: secs(r.wall_s),
        agent: secs(agent_s(r)),
        wait: secs(human_wait_s(r)),
        prompts: r.prompts,
        questions: r.questions,
        tool_failures: r.tool_failures,
        failures_tone: if r.tool_failures == 0 {
            "muted"
        } else {
            "warn"
        },
        cost: format_cost(r.cost_microdollars),
        state: r
            .terminal_state
            .clone()
            .or_else(|| r.outcome.clone())
            .unwrap_or_else(|| "—".to_owned()),
        phases: phases(r),
        verdict,
        verdict_tone,
    }
}

pub(super) fn run_views(runs: &[SkillRunRow], now: DateTime<Utc>) -> Vec<RunRowView> {
    runs.iter()
        .take(RUNS_SHOWN)
        .map(|r| run_view(r, now))
        .collect()
}
