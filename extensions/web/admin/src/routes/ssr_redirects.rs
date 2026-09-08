//! The legacy-path redirects the SSR router points at.
//!
//! They live beside the router rather than in it because the router file is a
//! table of route literals — the admin contract suite reads it as one — and a
//! handler body in the middle of that table is the thing that makes it stop
//! reading like a table.
//!
//! Every route in [`legacy_routes`] is a path the dashboard used to serve and
//! no longer does. They answer `308`, never HTML: one home per page, so a
//! bookmark or a stale link lands on the new page rather than on a second copy
//! of it that could drift. They are a one-release courtesy and are expected to
//! be deleted, which is why they are one table rather than scattered.

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, RawQuery};
use axum::response::Redirect;
use axum::routing::get;
use sqlx::PgPool;

// Why: `Redirect::permanent` is a 308, which preserves the method and the
// body. A 301 would let an intermediary rewrite a POST into a GET.
fn moved(target: &str) -> Redirect {
    Redirect::permanent(target)
}

fn with_query(base: &str, query: Option<String>) -> Redirect {
    match query {
        Some(q) if !q.is_empty() => moved(&format!("{base}?{q}")),
        _ => moved(base),
    }
}

// Why: the old paths, each 308ing to its new home.
pub(super) fn legacy_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/access/users", get(access_users))
        .route("/access/user", get(access_user))
        .route("/access/departments", get(access_departments))
        .route("/access/departments/{id}", get(access_department_detail))
        .route("/access/tokens", get(access_tokens))
        .route("/access/devices", get(access_tokens))
        .route("/access/matrix", get(access_control))
        .route("/governance/policies", get(governance_policies))
        .route("/entities/requests", get(entities_requests))
        .route("/entities/requests/{request_id}", get(entities_request))
        .route("/entities/sessions", get(entities_sessions))
        .route("/entities/sessions/{session_id}", get(entities_session))
        .route("/entities/traces", get(entities_traces))
        .route("/entities/traces/{trace_id}", get(entities_trace))
        .route("/entities/contexts", get(entities_contexts))
        .route("/entities/contexts/{context_id}", get(entities_context))
}

async fn access_users(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/users", q)
}

async fn access_user(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/user", q)
}

async fn access_departments(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/departments", q)
}

async fn access_department_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/departments/{id}"), q)
}

async fn access_tokens(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/access-tokens", q)
}

async fn access_control(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/access-control", q)
}

async fn governance_policies(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/governance", q)
}

async fn entities_requests(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/requests", q)
}

async fn entities_request(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/requests/{id}"), q)
}

async fn entities_sessions(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/sessions", q)
}

async fn entities_session(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/sessions/{id}"), q)
}

async fn entities_traces(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/traces", q)
}

async fn entities_trace(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/traces/{id}"), q)
}

async fn entities_contexts(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/contexts", q)
}

async fn entities_context(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/contexts/{id}"), q)
}
