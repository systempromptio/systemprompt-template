//! What a skill run achieved, and each kit release's run record.
//!
//! A run succeeded when its workflow log reached `PR_CREATED`; a logged run
//! that stopped anywhere else did not. A run with no workflow log falls back
//! to the judge: `achieved` succeeded, any other verdict did not, and an
//! unjudged run is unknown rather than a failure. A logged run whose last
//! activity is under half an hour old is still in flight, and also unknown.
//!
//! The release record groups runs by the kit version that was live when the
//! run started, so a release's effect reads as the change from the version
//! before it. Both the Runs section of the skill page and the
//! `analysis-kit-release-impact` export read these figures from here.

use chrono::{DateTime, Duration, Utc};

use crate::repositories::analysis::skills::SkillRunRow;

const SUCCESS_STATE: &str = "PR_CREATED";
const FAILED_STATES: &[&str] = &["STOPPED", "POLICY_REJECTED"];
const IN_FLIGHT_MINUTES: i64 = 30;

pub(crate) fn run_success(run: &SkillRunRow, now: DateTime<Utc>) -> Option<bool> {
    if let Some(state) = run.terminal_state.as_deref() {
        if state == SUCCESS_STATE {
            return Some(true);
        }
        if FAILED_STATES.contains(&state) {
            return Some(false);
        }
        if now - run.ended_at < Duration::minutes(IN_FLIGHT_MINUTES) {
            return None;
        }
        return Some(false);
    }
    run.outcome.as_deref().map(|o| o == "achieved")
}

pub(crate) fn human_wait_s(run: &SkillRunRow) -> f64 {
    (run.prompt_wait_s + run.question_wait_s).min(run.wall_s)
}

pub(crate) fn agent_s(run: &SkillRunRow) -> f64 {
    (run.wall_s - human_wait_s(run)).max(0.0)
}

pub(crate) fn version_label(run: &SkillRunRow) -> String {
    run.kit_version
        .clone()
        .or_else(|| {
            run.kit_hash
                .as_deref()
                .map(|h| h.chars().take(12).collect())
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

#[derive(Debug, Clone)]
pub(crate) struct ReleaseImpact {
    pub(crate) kit_version: String,
    pub(crate) skill: Option<String>,
    pub(crate) first_run_at: DateTime<Utc>,
    pub(crate) runs: i64,
    pub(crate) succeeded: i64,
    pub(crate) failed: i64,
    pub(crate) unknown: i64,
    pub(crate) success_rate: Option<f64>,
    pub(crate) median_wall_s: f64,
    pub(crate) p90_wall_s: f64,
    pub(crate) median_agent_s: f64,
    pub(crate) median_wait_s: f64,
    pub(crate) median_prompts: f64,
    pub(crate) median_questions: f64,
    pub(crate) tool_failure_rate: Option<f64>,
    pub(crate) cost_per_run_microdollars: i64,
    pub(crate) abandoned: i64,
}

// Why: grouped by (version, skill) when `per_skill`, else by version alone;
// newest release first, so a reader compares each row with the one below it.
// The version label, and the skill as well when grouping per skill.
type GroupKey = (String, Option<String>);

pub(crate) fn release_impact(
    runs: &[SkillRunRow],
    per_skill: bool,
    now: DateTime<Utc>,
) -> Vec<ReleaseImpact> {
    let mut groups: Vec<(GroupKey, Vec<&SkillRunRow>)> = Vec::new();
    for run in runs {
        let key = (version_label(run), per_skill.then(|| run.skill.clone()));
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(run),
            None => groups.push((key, vec![run])),
        }
    }
    let mut out: Vec<ReleaseImpact> = groups
        .into_iter()
        .map(|((kit_version, skill), list)| summarise(kit_version, skill, &list, now))
        .collect();
    out.sort_by(|a, b| {
        b.first_run_at
            .cmp(&a.first_run_at)
            .then_with(|| a.skill.cmp(&b.skill))
    });
    out
}

fn summarise(
    kit_version: String,
    skill: Option<String>,
    list: &[&SkillRunRow],
    now: DateTime<Utc>,
) -> ReleaseImpact {
    let verdicts: Vec<Option<bool>> = list.iter().map(|r| run_success(r, now)).collect();
    let succeeded = verdicts.iter().filter(|v| **v == Some(true)).count();
    let failed = verdicts.iter().filter(|v| **v == Some(false)).count();
    let decided = succeeded + failed;
    let tool_calls: i64 = list.iter().map(|r| r.tool_calls).sum();
    let tool_failures: i64 = list.iter().map(|r| r.tool_failures).sum();
    let cost: i64 = list.iter().map(|r| r.cost_microdollars).sum();
    let runs = count(list.len());
    ReleaseImpact {
        kit_version,
        skill,
        first_run_at: list.iter().map(|r| r.started_at).min().unwrap_or(now),
        runs,
        succeeded: count(succeeded),
        failed: count(failed),
        unknown: count(list.len() - decided),
        success_rate: (decided > 0).then(|| ratio(succeeded, decided)),
        median_wall_s: percentile(list.iter().map(|r| r.wall_s), 0.5),
        p90_wall_s: percentile(list.iter().map(|r| r.wall_s), 0.9),
        median_agent_s: percentile(list.iter().map(|r| agent_s(r)), 0.5),
        median_wait_s: percentile(list.iter().map(|r| human_wait_s(r)), 0.5),
        median_prompts: percentile(list.iter().map(|r| r.prompts as f64), 0.5),
        median_questions: percentile(list.iter().map(|r| r.questions as f64), 0.5),
        tool_failure_rate: (tool_calls > 0).then(|| tool_failures as f64 / tool_calls as f64),
        cost_per_run_microdollars: if runs > 0 { cost / runs } else { 0 },
        abandoned: count(
            list.iter()
                .filter(|r| r.outcome.as_deref() == Some("abandoned"))
                .count(),
        ),
    }
}

fn count(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn ratio(part: usize, whole: usize) -> f64 {
    part as f64 / whole as f64
}

// Why: nearest-rank on the sorted values — the figures are shown rounded to
// the minute or the whole prompt, so interpolation would only add noise.
fn percentile(values: impl Iterator<Item = f64>, p: f64) -> f64 {
    let mut sorted: Vec<f64> = values.filter(|v| v.is_finite()).collect();
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.sort_by(f64::total_cmp);
    let rank = (p * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}
