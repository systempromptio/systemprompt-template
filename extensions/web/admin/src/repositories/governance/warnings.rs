//! Warn-mode reads: what governance and the safety scanners would have refused.
//!
//! A policy configured `mode: warn` writes a `governance_decisions` row with
//! `decision = 'warn'` carrying the reason it would have denied on, and the
//! gateway's scanners under `safety.mode: warn` write `ai_safety_findings`
//! rows whose `blocked` column stays false. Both are ordinary audit history;
//! this module is the read side that turns them into something an operator can
//! retune a threshold from.
//!
//! The rollup is grouped by policy, tool and user at once and re-aggregated in
//! Rust rather than issued three times with a different `GROUP BY`. The
//! combinations are bounded by the policy count times the tool count, not by
//! traffic, and one query keeps the three views of a window provably
//! consistent with each other.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct WarningRollupRow {
    pub policy: String,
    pub tool_name: String,
    pub user_id: UserId,
    pub count: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub example_reason: String,
}

/// Which dimension of the warn rollup to collapse onto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningGroupBy {
    Policy,
    Tool,
    User,
}

impl WarningGroupBy {
    #[must_use]
    pub fn parse_group_by(value: Option<&str>) -> Self {
        match value {
            Some("tool") => Self::Tool,
            Some("user") => Self::User,
            _ => Self::Policy,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Policy => "policy",
            Self::Tool => "tool",
            Self::User => "user",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Policy => "Policy",
            Self::Tool => "Tool",
            Self::User => "User",
        }
    }
}

/// One re-aggregated group of the warn rollup.
#[derive(Debug, Clone)]
pub struct WarningGroup {
    pub group: String,
    pub warnings: i64,
    pub tools: i64,
    pub users: i64,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub example_reason: String,
}

// Why: distinct tool and user counts accumulate into sets rather than summing
// the SQL rollup's per-combination counts. Summing would report one tool used
// by two users as two tools, which is the number an operator would act on.
#[derive(Default)]
struct Accumulator {
    warnings: i64,
    tools: BTreeSet<String>,
    users: BTreeSet<String>,
    first_seen: Option<DateTime<Utc>>,
    last_seen: Option<DateTime<Utc>>,
    example_reason: String,
}

#[must_use]
pub fn group_warning_rollup(rows: &[WarningRollupRow], by: WarningGroupBy) -> Vec<WarningGroup> {
    let mut acc: BTreeMap<String, Accumulator> = BTreeMap::new();
    for row in rows {
        let key = match by {
            WarningGroupBy::Policy => row.policy.clone(),
            WarningGroupBy::Tool => row.tool_name.clone(),
            WarningGroupBy::User => row.user_id.as_str().to_owned(),
        };
        let entry = acc.entry(key).or_default();
        entry.warnings += row.count;
        entry.tools.insert(row.tool_name.clone());
        entry.users.insert(row.user_id.as_str().to_owned());
        if entry.first_seen.is_none_or(|seen| row.first_seen < seen) {
            entry.first_seen = Some(row.first_seen);
        }
        if entry.last_seen.is_none_or(|seen| seen < row.last_seen) {
            entry.last_seen = Some(row.last_seen);
            entry.example_reason.clone_from(&row.example_reason);
        }
    }

    let mut out: Vec<WarningGroup> = acc
        .into_iter()
        .map(|(group, a)| WarningGroup {
            group,
            warnings: a.warnings,
            tools: i64::try_from(a.tools.len()).unwrap_or(i64::MAX),
            users: i64::try_from(a.users.len()).unwrap_or(i64::MAX),
            first_seen: a.first_seen,
            last_seen: a.last_seen,
            example_reason: a.example_reason,
        })
        .collect();
    out.sort_by(|a, b| {
        b.warnings
            .cmp(&a.warnings)
            .then_with(|| a.group.cmp(&b.group))
    });
    out
}
