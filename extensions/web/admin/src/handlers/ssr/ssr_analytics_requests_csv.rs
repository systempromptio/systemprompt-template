//! `GET /admin/api/analytics/requests.csv` — CSV export of the Inference
//! Requests page. Mirrors filter / time-range parsing.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::repositories::analytics_grp::requests::{
    fetch_requests_paged, RequestFilter, RequestRow, RequestSortColumn, RequestSortSpec, SortDir,
};
use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRangeQuery};
use crate::types::UserContext;

use super::ssr_analytics_requests::RequestsQuery;

const CSV_LIMIT: i64 = 10_000;

pub async fn analytics_requests_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RequestsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let filter = RequestFilter {
        user_id: empty_to_none(query.user_id.as_ref()),
        agent_id: empty_to_none(query.agent_id.as_ref()),
        model: empty_to_none(query.model.as_ref()),
        provider: empty_to_none(query.provider.as_ref()),
        status: empty_to_none(query.status.as_ref()),
        search: empty_to_none(query.q.as_ref()),
    };
    let sort = RequestSortSpec {
        column: RequestSortColumn::CreatedAt,
        dir: SortDir::Desc,
    };

    let (rows, _total) = match fetch_requests_paged(&pool, &filter, range, sort, CSV_LIMIT, 0).await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "analytics requests CSV export failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let body = build_csv(&rows);
    let filename = format!(
        "ai-requests-{}-to-{}.csv",
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

fn build_csv(rows: &[RequestRow]) -> String {
    let mut out = String::with_capacity(rows.len() * 240);
    out.push_str(
        "created_at,id,request_id,trace_id,session_id,user_id,\
         provider,model,status,input_tokens,output_tokens,\
         cost_microdollars,latency_ms,decision_count,deny_count,\
         tool_call_count,error_message\n",
    );
    for r in rows {
        out.push_str(&csv_field(&r.created_at.to_rfc3339()));
        out.push(',');
        out.push_str(&csv_field(&r.id));
        out.push(',');
        out.push_str(&csv_field(&r.request_id));
        out.push(',');
        out.push_str(&csv_field(r.trace_id.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(r.session_id.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_field(&r.user_id));
        out.push(',');
        out.push_str(&csv_field(&r.provider));
        out.push(',');
        out.push_str(&csv_field(&r.model));
        out.push(',');
        out.push_str(&csv_field(&r.status));
        out.push(',');
        if let Some(t) = r.input_tokens {
            out.push_str(&t.to_string());
        }
        out.push(',');
        if let Some(t) = r.output_tokens {
            out.push_str(&t.to_string());
        }
        out.push(',');
        out.push_str(&r.cost_microdollars.to_string());
        out.push(',');
        if let Some(l) = r.latency_ms {
            out.push_str(&l.to_string());
        }
        out.push(',');
        out.push_str(&r.decision_count.to_string());
        out.push(',');
        out.push_str(&r.deny_count.to_string());
        out.push(',');
        out.push_str(&r.tool_call_count.to_string());
        out.push(',');
        out.push_str(&csv_field(r.error_message.as_deref().unwrap_or("")));
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
