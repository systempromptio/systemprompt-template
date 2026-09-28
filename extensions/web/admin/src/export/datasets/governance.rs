//! `/admin/governance` — the decision log, the safety findings and the
//! secrets audit, one row per evaluation.

use async_trait::async_trait;

use super::requests::time_range;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::governance::{export_decisions, export_findings};
use crate::handlers::ssr::secrets_audit;
use crate::repositories::governance::decision_log::DecisionLogRow;
use crate::repositories::governance::findings::SafetyFindingLogRow;
use crate::repositories::governance::secret_audit_log::SecretAuditRow;
use crate::types::governance_labels::plane_of;
use systemprompt::identifiers::{CallId, SessionId, UserId};

pub(crate) struct Decisions;
pub(crate) struct Findings;
pub(crate) struct Secrets;

const DECISION_COLUMNS: &[Column] = &[
    Column::new("at", "Time", CellKind::Timestamp),
    Column::new("decision", "Decision", CellKind::Text),
    Column::new("policy", "Policy", CellKind::Text),
    Column::new("stage", "Stage", CellKind::Text),
    Column::new("tool", "Tool", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("scope", "Agent scope", CellKind::Text),
    Column::new("reason", "Reason", CellKind::Text),
    Column::new("trace_id", "Trace", CellKind::Text),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("call_id", "Call", CellKind::Text).optional(),
    Column::new("record_id", "Record", CellKind::Text).optional(),
    Column::new("evaluated_rules", "Evaluated rules", CellKind::Text).optional(),
];

fn decision_row(r: &DecisionLogRow) -> Vec<Cell> {
    vec![
        r.created_at.into(),
        r.decision.as_str().into(),
        r.policy.as_str().into(),
        plane_of(&r.policy).into(),
        r.tool_name.as_str().into(),
        r.user_id.as_str().into(),
        Cell::opt_text(r.agent_scope.as_deref()),
        r.reason.as_str().into(),
        Cell::opt_text(r.trace_id.as_deref()),
        r.session_id.as_str().into(),
        Cell::opt_text(r.call_id.as_ref().map(CallId::as_str)),
        r.id.as_str().into(),
        r.evidence.as_str().into(),
    ]
}

const FINDING_COLUMNS: &[Column] = &[
    Column::new("at", "Time", CellKind::Timestamp),
    Column::new("outcome", "Outcome", CellKind::Text),
    Column::new("category", "Category", CellKind::Text),
    Column::new("scanner", "Scanner", CellKind::Text),
    Column::new("severity", "Severity", CellKind::Text),
    Column::new("phase", "Phase", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("excerpt", "Excerpt", CellKind::Text),
    Column::new("request_id", "Request", CellKind::Text),
    Column::new("trace_id", "Trace", CellKind::Text),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("record_id", "Record", CellKind::Text).optional(),
];

fn finding_row(r: &SafetyFindingLogRow) -> Vec<Cell> {
    vec![
        r.created_at.into(),
        if r.blocked { "blocked" } else { "audited" }.into(),
        r.category.as_str().into(),
        r.scanner.as_str().into(),
        r.severity.as_str().into(),
        r.phase.as_str().into(),
        Cell::opt_text(r.model.as_deref()),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        Cell::opt_text(r.excerpt.as_deref()),
        r.ai_request_id.as_str().into(),
        Cell::opt_text(r.trace_id.as_deref()),
        Cell::opt_text(r.session_id.as_ref().map(SessionId::as_str)),
        r.id.as_str().into(),
    ]
}

const SECRET_COLUMNS: &[Column] = &[
    Column::new("at", "Time", CellKind::Timestamp),
    Column::new("action", "Action", CellKind::Text),
    Column::new("variable", "Variable", CellKind::Text),
    Column::new("plugin", "Plugin", CellKind::Text),
    Column::new("owner", "Owner", CellKind::Text),
    Column::new("actor", "Actor", CellKind::Text),
    Column::new("third_party", "Third party", CellKind::Bool),
    Column::new("ip", "IP address", CellKind::Text),
    Column::new("record_id", "Record", CellKind::Text).optional(),
];

fn secret_row(r: &SecretAuditRow) -> Vec<Cell> {
    vec![
        r.created_at.into(),
        r.action.as_str().into(),
        r.var_name.as_str().into(),
        r.plugin_id.as_str().into(),
        r.user_id.as_str().into(),
        r.actor_id.as_str().into(),
        (r.actor_id != r.user_id).into(),
        Cell::opt_text(r.ip_address.as_deref()),
        r.id.as_str().into(),
    ]
}

#[async_trait]
impl DataSet for Decisions {
    fn id(&self) -> &'static str {
        "governance-decisions"
    }
    fn title(&self) -> &'static str {
        "Governance decisions"
    }
    fn description(&self) -> &'static str {
        "One row per policy evaluation: decision, policy, tool, person and reason."
    }
    fn columns(&self) -> &'static [Column] {
        DECISION_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (rows, total) = export_decisions(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(decision_row).collect(),
            total,
        })
    }
}

#[async_trait]
impl DataSet for Findings {
    fn id(&self) -> &'static str {
        "governance-findings"
    }
    fn title(&self) -> &'static str {
        "Safety findings"
    }
    fn description(&self) -> &'static str {
        "One row per scanner finding: category, scanner, severity, outcome and the request it hit."
    }
    fn columns(&self) -> &'static [Column] {
        FINDING_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (rows, total) = export_findings(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(finding_row).collect(),
            total,
        })
    }
}

#[async_trait]
impl DataSet for Secrets {
    fn id(&self) -> &'static str {
        "governance-secrets"
    }
    fn title(&self) -> &'static str {
        "Secrets audit"
    }
    fn description(&self) -> &'static str {
        "One row per secret read or change: variable, plugin, owner and who acted."
    }
    fn columns(&self) -> &'static [Column] {
        SECRET_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let (rows, total) = secrets_audit::export_rows(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(secret_row).collect(),
            total,
        })
    }
}
