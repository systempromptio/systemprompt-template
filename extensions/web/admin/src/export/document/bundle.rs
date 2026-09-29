//! The document a conversation leaves the console as, and the reads that
//! assemble it from the planes the detail pages already show separately.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::error::AdminResult;
use crate::handlers::ssr::transcript_view::{
    ConversationView, TranscriptOptions, build_conversation, display_body, transcript_request_ids,
};
use crate::repositories::analysis::conversations::ConversationFactRow;
use crate::repositories::analysis::conversations::detail::find_conversation_facts;
use crate::repositories::analysis::conversations::hook_events::{
    ConversationHookEventRow, list_conversation_hook_events,
};
use crate::repositories::analysis::conversations::planes::{
    ConversationTurnRow, list_conversation_decisions, list_conversation_safety_findings,
    list_conversation_skills, list_conversation_tool_calls, list_conversation_turns,
};
use crate::repositories::analytics::context_detail::{
    ContextHeader, ContextRequestRow, find_context_header, list_context_requests,
    list_messages_for_requests,
};
use crate::repositories::analytics::context_tool_calls::list_tool_calls_for_requests;

use super::view::{
    DecisionDoc, HookEventDoc, MessageDoc, RequestDoc, SafetyDoc, SkillDoc, ToolCallDoc,
    ToolLedgerDoc, decision_doc, hook_event_doc, message_doc, request_doc, safety_doc, skill_doc,
    tool_call_doc, tool_ledger_doc,
};

pub(crate) const SCHEMA_VERSION: u32 = 1;

// Why: the whole record of one conversation. Field order is the order a
// reader wants it in: who and what, then the judge, then the ledger, then the
// bodies, then the planes around them.
#[derive(Debug, Serialize)]
pub(crate) struct ConversationBundle {
    pub schema_version: u32,
    pub exported_at: DateTime<Utc>,
    pub redacted: bool,
    pub context_id: ContextId,
    pub title: Option<String>,
    pub header: HeaderDoc,
    pub facts: Option<ConversationFactRow>,
    pub requests: Vec<RequestDoc>,
    pub transcript: ConversationView,
    pub messages: Vec<MessageDoc>,
    pub tool_calls: Vec<ToolCallDoc>,
    pub tool_ledger: Vec<ToolLedgerDoc>,
    pub decisions: Vec<DecisionDoc>,
    pub skills: Vec<SkillDoc>,
    pub safety_findings: Vec<SafetyDoc>,
    pub hook_events: Vec<HookEventDoc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HeaderDoc {
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub client_session_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub hook_status: Option<String>,
}

fn header_doc(h: &ContextHeader) -> HeaderDoc {
    HeaderDoc {
        user_id: h.user_id.clone(),
        display_name: h.display_name.clone(),
        session_id: h.session_id.clone(),
        client_session_id: h.client_session_id.clone(),
        created_at: h.created_at,
        updated_at: h.updated_at,
        hook_status: h.hook_status.clone(),
    }
}

// Why: the reads the two detail pages perform, gathered once. The caller has
// already decided the viewer may see this owner's conversations; this only
// answers "is there a record" (`None`) and assembles it.
pub(crate) async fn load_conversation_bundle(
    pool: &PgPool,
    context_id: &ContextId,
    opts: TranscriptOptions,
) -> AdminResult<Option<ConversationBundle>> {
    let Some(header) = find_context_header(pool, context_id).await? else {
        return Ok(None);
    };
    let client_session = header.client_session_id.as_deref();
    let (facts, requests, turns, ledger, skills, safety) = tokio::try_join!(
        find_conversation_facts(pool, context_id),
        list_context_requests(pool, context_id),
        list_conversation_turns(pool, context_id),
        list_conversation_tool_calls(pool, context_id, client_session),
        list_conversation_skills(pool, context_id),
        list_conversation_safety_findings(pool, context_id),
    )?;
    let ids = transcript_request_ids(&requests);
    let (messages, tool_calls) = tokio::try_join!(
        list_messages_for_requests(pool, &ids.messages),
        list_tool_calls_for_requests(pool, &ids.tool_calls),
    )?;
    let (decisions, hook_events) = tokio::try_join!(
        list_conversation_decisions(pool, context_id, client_session),
        hook_events(pool, client_session),
    )?;
    let transcript = build_conversation(&messages, &tool_calls, &requests, opts);
    Ok(Some(ConversationBundle {
        schema_version: SCHEMA_VERSION,
        exported_at: Utc::now(),
        redacted: opts.redact,
        context_id: context_id.clone(),
        title: facts
            .as_ref()
            .map(|f| f.title.clone())
            .or_else(|| header.ai_title.clone())
            .or_else(|| header.name.clone()),
        header: header_doc(&header),
        facts,
        requests: request_docs(&requests, &turns),
        transcript,
        messages: messages
            .iter()
            .map(|m| message_doc(m, |body| display_body(body, opts).0))
            .collect(),
        tool_calls: tool_calls.iter().map(tool_call_doc).collect(),
        tool_ledger: ledger.iter().map(tool_ledger_doc).collect(),
        decisions: decisions.iter().map(decision_doc).collect(),
        skills: skills.iter().map(skill_doc).collect(),
        safety_findings: safety.iter().map(safety_doc).collect(),
        hook_events: hook_events.iter().map(hook_event_doc).collect(),
    }))
}

async fn hook_events(
    pool: &PgPool,
    client_session: Option<&str>,
) -> Result<Vec<ConversationHookEventRow>, sqlx::Error> {
    match client_session {
        Some(session) => list_conversation_hook_events(pool, session).await,
        None => Ok(Vec::new()),
    }
}

// Why: the reader's request list carries the transcript bookkeeping and the
// analysis ledger carries routing, finish reasons and safety counts for the
// same rows; one document row joins them by request id.
fn request_docs(requests: &[ContextRequestRow], turns: &[ConversationTurnRow]) -> Vec<RequestDoc> {
    requests
        .iter()
        .map(|r| {
            let turn = turns.iter().find(|t| t.request_id == r.id.as_str());
            request_doc(r, turn)
        })
        .collect()
}
