//! `GET /admin/api/governance/decisions.csv` — CSV export of the audit page.
//!
//! Mirrors `governance_audit_page`'s filter/time-range parsing so the bookmark
//! URL on the page can be turned into a CSV by swapping the path.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::repositories::governance_grp::paged::{
    fetch_decisions_paged, DecisionFilter, DecisionRow, SortColumn, SortDir, SortSpec,
};
use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRangeQuery};
use crate::types::UserContext;

use super::ssr_governance_audit::AuditQuery;

const CSV_LIMIT: i64 = 10_000;

pub async fn governance_audit_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AuditQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let filter = DecisionFilter {
        user_id: empty_to_none(query.user_id.as_ref()),
        agent_id: empty_to_none(query.agent_id.as_ref()),
        agent_scope: empty_to_none(query.agent_scope.as_ref()),
        policy: empty_to_none(query.policy.as_ref()),
        decision: empty_to_none(query.decision.as_ref()),
        tool_name: empty_to_none(query.tool_name.as_ref()),
        search: empty_to_none(query.q.as_ref()),
    };
    let sort = SortSpec {
        column: SortColumn::CreatedAt,
        dir: SortDir::Desc,
    };

    let (rows, _total) = match fetch_decisions_paged(&pool, &filter, range, sort, CSV_LIMIT, 0).await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "audit csv export failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let body = build_csv(&rows);
    let filename = format!(
        "governance-decisions-{}-to-{}.csv",
        range.from.format("%Y%m%dT%H%M%SZ"),
        range.to.format("%Y%m%dT%H%M%SZ"),
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    if let Ok(disp) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, disp);
    }
    (StatusCode::OK, headers, body).into_response()
}

fn empty_to_none(v: Option<&String>) -> Option<String> {
    v.map(String::as_str)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn build_csv(rows: &[DecisionRow]) -> String {
    let mut out = String::with_capacity(rows.len() * 200);
    out.push_str(
        "created_at,decision_id,trace_id,session_id,user_id,\
         agent_id,agent_scope,tool_name,policy,decision,reason,\
         cost_microdollars,latency_ms\n",
    );
    for r in rows {
        out.push_str(&csv_field(&r.created_at.to_rfc3339()));
        out.push(',');
        out.push_str(&csv_field(&r.id));
        out.push(',');
        out.push_str(&csv_field(r.trace_id.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(&r.session_id));
        out.push(',');
        out.push_str(&csv_field(&r.user_id));
        out.push(',');
        out.push_str(&csv_field(r.agent_id.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(r.agent_scope.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(&r.tool_name));
        out.push(',');
        out.push_str(&csv_field(&r.policy));
        out.push(',');
        out.push_str(&csv_field(&r.decision));
        out.push(',');
        out.push_str(&csv_field(&r.reason));
        out.push(',');
        if let Some(c) = r.cost_microdollars {
            out.push_str(&c.to_string());
        }
        out.push(',');
        if let Some(l) = r.latency_ms {
            out.push_str(&l.to_string());
        }
        out.push('\n');
    }
    out
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        let escaped = value.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        value.to_string()
    }
}
