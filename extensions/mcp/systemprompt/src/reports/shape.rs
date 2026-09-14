//! Public report request and response contracts.

use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// JSON: upstream report envelopes carry heterogeneous table cells.
use serde_json::Value;
use std::collections::BTreeMap;
use systemprompt::mcp::McpOutputSchema;

// Why: a FLAT struct with a string discriminator, not a serde-tagged enum. A
// tagged enum emits a top-level `oneOf` with no `properties`, and Claude Code
// drops any tool whose inputSchema is shaped that way — `admin_report` was
// invisible to the CLI client for exactly that reason while the sibling tool,
// a plain struct, came through fine.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportInput {
    #[schemars(
        description = "Which report to run. `costs` is the only kind: AI usage, spend, models and sessions from this platform's own audit tables."
    )]
    pub report: ReportKind,
    #[schemars(description = "Lookback window in days, 1-90. Defaults to 7.")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    Costs,
}

impl ReportInput {
    #[must_use]
    pub fn days_or_default(&self) -> u16 {
        self.days.unwrap_or_else(seven_days)
    }
}

const fn seven_days() -> u16 {
    7
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SourceStatus {
    pub source: String,
    pub complete: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReportTable {
    pub title: String,
    pub columns: Vec<String>,
    // JSON: upstream CLI and Atlassian table cells are heterogeneous protocol values.
    pub rows: Vec<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReportOutput {
    pub report: String,
    pub title: String,
    pub checked_at: String,
    pub period: String,
    pub complete: bool,
    pub sources: Vec<SourceStatus>,
    pub metrics: BTreeMap<String, u64>,
    #[serde(default)]
    pub highlights: BTreeMap<String, String>,
    pub tables: Vec<ReportTable>,
    // Why: `default` so a stored artifact written before this field existed —
    // or by a version that omitted it — still deserialises. The renderer reads
    // this back out of the A2A data part, and a hard failure there costs the
    // whole dashboard.
    #[serde(default)]
    pub request: Option<ReportInput>,
}

impl ReportOutput {
    pub(crate) fn new(report: &str, title: &str, period: String) -> Self {
        Self {
            report: report.into(),
            title: title.into(),
            checked_at: Utc::now().to_rfc3339(),
            period,
            complete: true,
            sources: Vec::new(),
            metrics: BTreeMap::new(),
            highlights: BTreeMap::new(),
            tables: Vec::new(),
            request: None,
        }
    }

    pub(crate) fn source(&mut self, source: &str, complete: bool, warning: Option<String>) {
        self.complete &= complete;
        self.sources.push(SourceStatus {
            source: source.into(),
            complete,
            warning,
        });
    }
}

impl McpOutputSchema for ReportOutput {
    fn artifact_type() -> &'static str {
        "dashboard"
    }
    fn artifact_title(&self) -> Option<String> {
        Some(self.title.clone())
    }
    fn text_body(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

pub(crate) fn table(title: &str, values: Vec<Value>) -> ReportTable {
    let rows: Vec<BTreeMap<String, Value>> = values
        .into_iter()
        .map(|v| match v {
            Value::Object(map) => map.into_iter().collect(),
            other => BTreeMap::from([("value".into(), other)]),
        })
        .collect();
    let columns = rows
        .iter()
        .flat_map(|r| r.keys().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    ReportTable {
        title: title.into(),
        columns,
        rows,
    }
}
