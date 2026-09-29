//! The Handlebars shapes of the conversation detail page.

use serde::Serialize;

use crate::handlers::ssr::analysis::help::HelpView;

use crate::handlers::ssr::analysis::conversations::view::SkillChipView;
use crate::handlers::ssr::format::ClientChipView;
use crate::handlers::ssr::types::{BreadcrumbView, SvgLineChartView};

#[derive(Debug, Serialize)]
pub(super) struct ConversationKpiView {
    pub(super) label: &'static str,
    pub(super) icon: &'static str,
    pub(super) value: String,
    pub(super) note: String,
    pub(super) tone: &'static str,
    pub(super) hint: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct JudgeView {
    pub(super) judged: bool,
    pub(super) pending: bool,
    pub(super) completion_display: String,
    pub(super) tone: &'static str,
    pub(super) category_label: &'static str,
    pub(super) outcome: &'static str,
    pub(super) outcome_tone: &'static str,
    pub(super) summary: String,
    pub(super) rationale: String,
    pub(super) tags: Vec<String>,
    pub(super) judged_at: String,
    pub(super) model: String,
    pub(super) cost_display: String,
    pub(super) tokens_display: String,
    pub(super) trigger: String,
    pub(super) judge_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct LedgerTurnView {
    pub(super) index: usize,
    pub(super) request_id: String,
    pub(super) at: String,
    pub(super) kind: &'static str,
    pub(super) is_turn: bool,
    pub(super) model: String,
    pub(super) routed: Option<String>,
    pub(super) status: String,
    pub(super) status_tone: &'static str,
    pub(super) finish_reason: String,
    pub(super) input_display: String,
    pub(super) output_display: String,
    pub(super) cache_display: String,
    pub(super) reasoning_display: String,
    pub(super) cost_display: String,
    pub(super) latency_display: String,
    pub(super) latency_tone: &'static str,
    pub(super) streaming: bool,
    pub(super) tools: i64,
    pub(super) tool_names: String,
    pub(super) safety: i64,
    pub(super) safety_tone: &'static str,
    pub(super) error_message: Option<String>,
    pub(super) href: String,
}

#[derive(Debug, Serialize)]
pub(super) struct LedgerCallView {
    pub(super) tool_name: String,
    pub(super) tool_icon: &'static str,
    pub(super) is_builtin: bool,
    pub(super) server_name: String,
    pub(super) input_summary: String,
    pub(super) state: String,
    pub(super) state_tone: &'static str,
    pub(super) source: String,
    pub(super) status: String,
    pub(super) duration_display: String,
    pub(super) at: String,
    pub(super) artifact_kind: Option<String>,
    pub(super) artifact_icon: &'static str,
    pub(super) artifact_label: &'static str,
    pub(super) artifact_title: Option<String>,
    pub(super) artifact_href: Option<String>,
    pub(super) preview_href: Option<String>,
    pub(super) is_path: bool,
    pub(super) bytes_display: String,
    pub(super) error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct DecisionView {
    pub(super) tool_name: String,
    pub(super) decision: String,
    pub(super) tone: &'static str,
    pub(super) policy: String,
    pub(super) reason: String,
    pub(super) plugin: String,
    pub(super) at: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ConversationSkillView {
    pub(super) chip: SkillChipView,
    pub(super) plugin: String,
    pub(super) marketplace: String,
    pub(super) version_short: String,
    pub(super) version_href: Option<String>,
    pub(super) invocations: i64,
    pub(super) first_at: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SafetyView {
    pub(super) phase: String,
    pub(super) category: String,
    pub(super) severity: String,
    pub(super) scanner: String,
    pub(super) blocked: bool,
    pub(super) tone: &'static str,
    pub(super) at: String,
    pub(super) request_href: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ConversationDetailContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) crumb: String,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) context_key: String,
    pub(super) heading: String,
    pub(super) user_href: String,
    pub(super) display_name: String,
    pub(super) group_name: Option<String>,
    pub(super) project_name: Option<String>,
    pub(super) client: ClientChipView,
    pub(super) wire_protocol: String,
    pub(super) models: Vec<String>,
    pub(super) providers: Vec<String>,
    pub(super) first_display: String,
    pub(super) last_display: String,
    pub(super) duration_display: String,
    pub(super) hook_status: String,
    pub(super) session_href: Option<String>,
    pub(super) context_href: String,
    pub(super) back_url: &'static str,
    pub(super) judge: JudgeView,
    pub(super) can_judge: bool,
    pub(super) kpis: Vec<ConversationKpiView>,
    pub(super) charts: Vec<SvgLineChartView>,
    pub(super) turns: Vec<LedgerTurnView>,
    pub(super) turn_count: usize,
    pub(super) tool_calls: Vec<LedgerCallView>,
    pub(super) tool_count: usize,
    pub(super) artifacts: Vec<LedgerCallView>,
    pub(super) artifact_count: usize,
    pub(super) decisions: Vec<DecisionView>,
    pub(super) decision_count: usize,
    pub(super) skills: Vec<ConversationSkillView>,
    pub(super) skill_count: usize,
    pub(super) safety: Vec<SafetyView>,
    pub(super) safety_count: usize,
    pub(super) help: HelpView,
    pub(super) export: crate::export::ExportView,
}
