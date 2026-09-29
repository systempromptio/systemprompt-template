//! `/admin/analysis/conversations` — every gateway conversation as the
//! record shows it: who, which client and model, turns, tool calls, errors
//! and denials, tokens and cache share, cost, latency and duration, the
//! skills invoked — with the judge's one label (title, intent, outcome,
//! completion) joined on top when it exists.
//!
//! KPI tiles carry a sparkline of the window in a reserved trend slot; three
//! charts share one x axis padded to the whole window; one breakdown at a
//! time (intent, model, client, group, project, person, skill, outcome) over
//! the same filtered set the table pages through, so the buckets add up to
//! the tiles, each bucket linking into the list and downloadable on its own.
//! Filters live in the ribbon: window, intent, outcome, judged band, model,
//! client, a record flag, skill, person, free text and the shared
//! `?group=&project=` scope. Rows can be ticked for export or a bulk judge
//! request; every unjudged row offers a Judge button; the latest global AI
//! report sits in a banner under the header.

pub(crate) mod charts;
mod context;
mod continuation;
pub(crate) mod export;
mod exports;
mod filters;
pub(crate) mod judge_all;
pub(crate) mod kpis;
mod links;
mod page_context;
pub(crate) mod query;
mod row_facts;
pub(crate) mod summary;
pub(crate) mod view;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt_web_shared::{GroupId, ProjectId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::scope_filter_view;
use crate::handlers::ssr::page::Page;
use crate::repositories;
use crate::repositories::analysis::conversations::{
    ConversationAnalysisFilter, ConversationAnalysisPage, load_conversation_analysis_page,
};
use crate::repositories::scope::ScopeRequest;

use crate::handlers::ssr::list_view::DEFAULT_PAGE_SIZE;
use context::{ConversationsPageInputs, build_context};
pub(crate) use judge_all::judge_all;
use query::{BASE_URL, ConversationAnalysisQuery};

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ConversationAnalysisQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let request = ScopeRequest::from_query(
        &shell.user,
        params.group.as_deref(),
        params.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let filter = ConversationAnalysisFilter {
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
        user_id: params.user_id(),
        category: params.category(),
        outcome: params.outcome(),
        skill: params.skill(),
        free_text: params.free_text(),
        since: params.since_datetime(),
        until: None,
        judged: params.judged(),
        model: params.model(),
        client_kind: params.client(),
        group_id: request.group.as_deref().map(GroupId::new),
        project_id: request.project.as_deref().map(ProjectId::new),
        flag: params.flag(),
        context_ids: params.context_ids(),
        include_without_turns: params.show_all(),
    };
    let data = load_conversation_analysis_page(
        &pool,
        &filter,
        ConversationAnalysisPage {
            sort: params.sort(),
            descending: params.descending(),
            limit: DEFAULT_PAGE_SIZE,
            offset: params.page() * DEFAULT_PAGE_SIZE,
            breakdown: params.breakdown(),
        },
    )
    .await?;
    let scope_filter = scope_filter_view(
        &pool,
        &shell.user,
        &request,
        BASE_URL,
        params.preserved(&["group", "project", "page"]),
    )
    .await;
    let report_banner = crate::handlers::ssr::analysis::reports::banner::report_banner(
        &pool,
        "global",
        None,
        &params.query_string(),
        "Conversations in view",
    )
    .await;
    let context = build_context(
        &params,
        &request,
        &data,
        ConversationsPageInputs {
            scope_filter,
            report_banner,
            can_judge: shell.user.is_console,
            manual_judge: !crate::handlers::ssr::analysis::judge_mode::automatic(),
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-conversations",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
