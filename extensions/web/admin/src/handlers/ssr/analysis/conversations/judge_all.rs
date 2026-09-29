//! `POST /admin/analysis/conversations/judge` — queue many conversations for
//! the judge at once: the ticked rows (`ids`) from the bulk bar, or every
//! unjudged conversation the page's filters select (`query`), capped so one
//! click never enqueues the whole history. Every context is re-read through
//! the page repository under the caller's scope, so a participant can only
//! queue what they can see. The next `conversation_judge` tick claims manual
//! rows whatever the profile's automatic switch says.

use std::sync::Arc;

use axum::extract::{Form, Query, State};
use axum::http::{HeaderMap, Uri};
use axum::response::Redirect;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt_web_shared::{GroupId, ProjectId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::page::Page;
use crate::repositories;
use crate::repositories::analysis::conversations::detail::insert_manual_judgements;
use crate::repositories::analysis::conversations::{
    ConversationAnalysisFilter, ConversationAnalysisPage, JudgedFilter,
    load_conversation_analysis_page,
};
use crate::repositories::scope::ScopeRequest;

use super::query::{BASE_URL, ConversationAnalysisQuery};

pub(crate) const JUDGE_ALL_URL: &str = "/admin/analysis/conversations/judge";

// Why: one click may queue at most this many; the judge's daily cost cap is
// the other guard, and a backlog past this is a scheduling decision.
const MAX_PER_CLICK: i64 = 200;

#[derive(Debug, Deserialize)]
pub(crate) struct JudgeAllForm {
    pub ids: Option<String>,
    pub query: Option<String>,
    pub back: Option<String>,
}

fn back_url(form: &JudgeAllForm) -> String {
    form.back
        .as_deref()
        .filter(|b| b.starts_with(BASE_URL))
        .unwrap_or(BASE_URL)
        .to_owned()
}

// Why: the page's own filters, re-read from the query string the form
// carried, narrowed to unjudged rows.
fn params_from(form: &JudgeAllForm) -> Result<ConversationAnalysisQuery, AdminError> {
    let query = form
        .query
        .as_deref()
        .unwrap_or_default()
        .trim_start_matches('?');
    let uri: Uri = format!("{BASE_URL}?{query}")
        .parse()
        .map_err(|_bad| AdminError::BadRequest("That filter query is not valid.".to_owned()))?;
    let mut params = Query::<ConversationAnalysisQuery>::try_from_uri(&uri)
        .map(|q| q.0)
        .map_err(|e| AdminError::BadRequest(e.body_text()))?;
    params.judged = Some(JudgedFilter::Unjudged.as_str().to_owned());
    params.page = None;
    params.ids.clone_from(&form.ids);
    Ok(params)
}

pub(crate) async fn judge_all(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<JudgeAllForm>,
) -> AdminHtmlResult<Redirect> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    crate::handlers::shared::require_write_origin(&headers)?;
    let params = params_from(&form)?;
    let request = ScopeRequest::from_query(
        &shell.user,
        params.group.as_deref(),
        params.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let selected = params.context_ids();
    let filter = ConversationAnalysisFilter {
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
        user_id: params.user_id(),
        category: params.category(),
        outcome: params.outcome(),
        skill: params.skill(),
        free_text: params.free_text(),
        since: params.since_datetime(),
        until: None,
        // Why: a ticked row is re-judged even when it already has a verdict;
        // the filtered set only ever adds what has none.
        judged: if selected.is_some() {
            None
        } else {
            Some(JudgedFilter::Unjudged)
        },
        model: params.model(),
        client_kind: params.client(),
        group_id: request.group.as_deref().map(GroupId::new),
        project_id: request.project.as_deref().map(ProjectId::new),
        flag: params.flag(),
        context_ids: selected,
        include_without_turns: false,
    };
    let data = load_conversation_analysis_page(
        &pool,
        &filter,
        ConversationAnalysisPage {
            sort: params.sort(),
            descending: params.descending(),
            limit: MAX_PER_CLICK,
            offset: 0,
            breakdown: params.breakdown(),
        },
    )
    .await?;
    let ids: Vec<String> = data
        .rows
        .iter()
        .map(|r| r.context_id.as_str().to_owned())
        .collect();
    let queued = insert_manual_judgements(&pool, &ids, &shell.user.user_id).await?;
    tracing::info!(
        queued,
        requested_by = %shell.user.user_id,
        selected = form.ids.is_some(),
        "conversations queued for the judge from the console"
    );
    Ok(Redirect::to(&back_url(&form)))
}
