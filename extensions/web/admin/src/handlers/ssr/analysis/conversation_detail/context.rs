//! Assembling the conversation detail page context.

use super::figures::{charts, kpis};
use super::rows::{decision_view, judge_view, safety_view, skill_view, tool_view, turn_view};
use super::views::{ConversationDetailContext, LedgerCallView};
use crate::handlers::ssr::analysis_urls::{ANALYSIS_CONVERSATIONS_URL, analysis_conversation_url};
use crate::handlers::ssr::format::{client_chip, format_duration_ms, local_time};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analysis::conversations::ConversationFactRow;
use crate::repositories::analysis::conversations::planes::{
    ConversationDecisionRow, ConversationSafetyRow, ConversationSkillRow, ConversationToolCallRow,
    ConversationTurnRow,
};

// Why: everything read beside the fact row, handed to the builder as one.
pub(super) struct Planes<'a> {
    pub(super) turns: &'a [ConversationTurnRow],
    pub(super) tools: &'a [ConversationToolCallRow],
    pub(super) decisions: &'a [ConversationDecisionRow],
    pub(super) skills: &'a [ConversationSkillRow],
    pub(super) safety: &'a [ConversationSafetyRow],
}

fn artifact_views(tools: &[ConversationToolCallRow]) -> Vec<LedgerCallView> {
    tools
        .iter()
        .filter(|t| t.artifact_kind.is_some())
        .map(tool_view)
        .collect()
}

fn display_name(f: &ConversationFactRow) -> String {
    f.display_name
        .clone()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| f.user_id.as_str().to_owned())
}

pub(super) fn build_context(
    f: &ConversationFactRow,
    planes: &Planes<'_>,
    can_judge: bool,
) -> ConversationDetailContext {
    let Planes {
        turns,
        tools,
        decisions,
        skills,
        safety,
    } = *planes;
    let heading = f
        .judge_title
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| f.title.clone());
    let short = f.context_id.as_str()[..f.context_id.as_str().len().min(8)].to_owned();
    ConversationDetailContext {
        page: "analysis-conversation",
        title: format!("{heading} · Conversation"),
        crumb: heading.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Analysis", ANALYSIS_CONVERSATIONS_URL),
            BreadcrumbView::link("Conversations", ANALYSIS_CONVERSATIONS_URL),
            BreadcrumbView::current(short),
        ],
        context_key: f.context_id.as_str().to_owned(),
        heading,
        user_href: format!("/admin/users/{}", urlencoding::encode(f.user_id.as_str())),
        display_name: display_name(f),
        group_name: f.group_name.clone(),
        project_name: f.project_name.clone(),
        client: client_chip(&f.client_kind, &f.client_attestation),
        wire_protocol: f.wire_protocol.clone(),
        models: f.models.clone(),
        providers: f.providers.clone(),
        first_display: local_time(f.first_at),
        last_display: local_time(f.last_at),
        duration_display: format_duration_ms(f.duration_seconds * 1000),
        hook_status: f
            .hook_status
            .clone()
            .unwrap_or_else(|| "no hooks".to_owned()),
        session_href: f
            .client_session_id
            .as_deref()
            .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        context_href: format!(
            "/admin/contexts/{}",
            urlencoding::encode(f.context_id.as_str())
        ),
        back_url: ANALYSIS_CONVERSATIONS_URL,
        help: crate::handlers::ssr::analysis::help::conversation_detail_help(),
        export: crate::export::ExportView::conversation(&f.context_id, true),
        judge: judge_view(
            f,
            format!("{}/judge", analysis_conversation_url(&f.context_id)),
        ),
        can_judge,
        kpis: kpis(f),
        charts: charts(turns),
        turns: turns.iter().enumerate().map(turn_view).collect(),
        turn_count: turns.len(),
        tool_count: tools.len(),
        artifact_count: tools.iter().filter(|t| t.artifact_kind.is_some()).count(),
        artifacts: artifact_views(tools),
        tool_calls: tools.iter().map(tool_view).collect(),
        decisions: decisions.iter().map(decision_view).collect(),
        decision_count: decisions.len(),
        skills: skills.iter().map(skill_view).collect(),
        skill_count: skills.len(),
        safety: safety.iter().map(safety_view).collect(),
        safety_count: safety.len(),
    }
}
