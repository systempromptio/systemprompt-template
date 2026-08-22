//! Read/write models for the Evals page.
//!
//! The gateway spine (`ai_requests` + `ai_request_payloads`) is the input:
//! [`sampling`] draws candidates from it, [`distribution`] summarises it, and
//! [`scores`] reports what the judge made of it. The eval tables themselves are
//! written through [`runs`], [`results`], and [`cases`].

use serde::{Deserialize, Serialize};

pub mod cases;
pub mod distribution;
pub mod results;
pub mod runs;
pub mod sampling;
pub mod scores;

/// What an eval run does. Stored in `eval_runs.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvalRunKind {
    Judge,
    Replay,
    Pairwise,
}

impl EvalRunKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Judge => "judge",
            Self::Replay => "replay",
            Self::Pairwise => "pairwise",
        }
    }

    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "judge" => Some(Self::Judge),
            "replay" => Some(Self::Replay),
            "pairwise" => Some(Self::Pairwise),
            _ => None,
        }
    }
}

/// Terminal state of a scored item. `Skipped` covers items the deterministic
/// pre-pass rejected before any judge token was spent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvalVerdict {
    Pass,
    Partial,
    Fail,
    Skipped,
}

impl EvalVerdict {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Partial => "partial",
            Self::Fail => "fail",
            Self::Skipped => "skipped",
        }
    }
}

/// Lifecycle of a run row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalRunStatus {
    Running,
    Completed,
    Failed,
}

impl EvalRunStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// Which side won a pairwise comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PairWinner {
    A,
    B,
    Tie,
}

impl PairWinner {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::Tie => "tie",
        }
    }
}
