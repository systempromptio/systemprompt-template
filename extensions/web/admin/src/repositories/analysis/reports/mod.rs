//! AI reports — the repository behind `/admin/analysis/reports`.
//!
//! A report is one row of `analysis_reports` (schema 46): a request made from
//! the console for a scope (the instance, a marketplace, a skill or a saved
//! page filter) and a window, the deterministic digest that was computed for
//! it at request time (`inputs`), and — once the manual-only
//! `analysis_report` job has run — the model's structured findings
//! (`findings`), with the audit row, tokens and cost of that one call. The
//! digest is pure SQL over `conversation_facts`, `conversation_analyses` and
//! the hook plane (`digest.sql`); no transcript ever leaves the database.
//! The queue functions are fenced by a lease token exactly like the judge's.

mod digest;
mod links;
mod queue;
mod reads;

pub use digest::{DigestScope, DigestTotals, ReportDigest, get_report_digest, render_digest_text};
pub use links::evidence_href;
pub use queue::{
    NewReport, ReportCompletion, fail_report, insert_report_request, update_report_completion,
};
pub use reads::{find_latest_generated_report, find_report, list_reports};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The model's overall verdict on the scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Assessment {
    Ok,
    Watch,
    Degraded,
}

impl Assessment {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Watch => "watch",
            Self::Degraded => "degraded",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "Healthy",
            Self::Watch => "Watch",
            Self::Degraded => "Degraded",
        }
    }

    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Watch => "warn",
            Self::Degraded => "err",
        }
    }
}

/// How serious one theme is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Ok,
    Warn,
    Err,
}

impl Severity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Err => "err",
        }
    }
}

/// How urgent one recommendation is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::High => "err",
            Self::Medium => "warn",
            Self::Low => "muted",
        }
    }
}

/// One link a theme rests on, already resolved to a console page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: String,
    pub label: String,
    pub href: String,
}

/// One finding the model grounded in the digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub kind: String,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
}

/// One action the model proposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub rationale: String,
    pub priority: Priority,
}

/// What the model returned, normalised; stored as `analysis_reports.findings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFindings {
    pub headline: String,
    pub assessment: Assessment,
    #[serde(default)]
    pub themes: Vec<Theme>,
    #[serde(default)]
    pub recommendations: Vec<Recommendation>,
}

/// What the model was given; stored as `analysis_reports.inputs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDigestInputs {
    pub digest: ReportDigest,
    #[serde(default)]
    pub filter_query: Option<String>,
    #[serde(default)]
    pub filter_label: Option<String>,
}

/// One report as the pages and the job read it.
#[derive(Debug, Clone)]
pub struct AnalysisReportRow {
    pub id: String,
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub scope_label: Option<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub status: String,
    pub requested_by: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub ai_request_id: Option<String>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: Option<i64>,
    pub inputs: ReportDigestInputs,
    pub findings: Option<ReportFindings>,
    pub attempts: i32,
    pub lease_token: Option<String>,
    pub last_error: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
