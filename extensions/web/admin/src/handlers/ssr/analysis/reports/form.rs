//! Turning a report request — the Generate form, a "Report on this view"
//! action or a Regenerate — into the scope the digest is computed over and
//! the row that is queued.

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::analysis::reports::{
    AnalysisReportRow, DigestScope, NewReport, ReportDigestInputs, get_report_digest,
};
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ReportRequestForm {
    pub scope_kind: Option<String>,
    pub scope_id: Option<String>,
    pub label: Option<String>,
    pub days: Option<i64>,
    pub query: Option<String>,
}

fn trimmed(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

// Why: the page query a "Report on this view" action carries is the
// Conversations / Skills query string as-is. The keys read from it are
// `since` (24h|7d|30d|90d|all) or `days` (N), `group`, `project`, `user_id`,
// `model`, `client`, `skill`, `category` and `outcome`; anything else
// (sort, page, breakdown, free text) does not narrow a digest and is ignored.
fn query_pairs(query: &str) -> Vec<(String, String)> {
    query
        .trim_start_matches('?')
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            let v = urlencoding::decode(v).ok()?.into_owned();
            (!v.is_empty()).then(|| (k.to_owned(), v))
        })
        .collect()
}

fn lookup<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn window_days(form: &ReportRequestForm, pairs: &[(String, String)]) -> i64 {
    if let Some(days) = form.days {
        return days.clamp(1, 365);
    }
    match lookup(pairs, "since") {
        Some("24h" | "1d") => 1,
        Some("7d") => 7,
        Some("90d") => 90,
        Some("all") => 365,
        _ => lookup(pairs, "days")
            .and_then(|d| d.parse::<i64>().ok())
            .map_or(30, |d| d.clamp(1, 365)),
    }
}

// Why: what a request resolved to before it is written.
pub(super) struct ResolvedRequest {
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub scope_label: Option<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub digest_scope: DigestScope,
    pub filter_query: Option<String>,
}

pub(super) async fn resolve(
    pool: &PgPool,
    user: &UserContext,
    form: &ReportRequestForm,
) -> AdminResult<ResolvedRequest> {
    let scope_kind = trimmed(form.scope_kind.as_deref()).unwrap_or_else(|| "global".to_owned());
    if !matches!(
        scope_kind.as_str(),
        "global" | "marketplace" | "skill" | "filter"
    ) {
        return Err(AdminError::BadRequest("Unknown report scope".to_owned()));
    }
    let scope_id = trimmed(form.scope_id.as_deref());
    if matches!(scope_kind.as_str(), "marketplace" | "skill") && scope_id.is_none() {
        return Err(AdminError::BadRequest(
            "A marketplace or skill report needs an id".to_owned(),
        ));
    }
    let filter_query = (scope_kind == "filter")
        .then(|| trimmed(form.query.as_deref()))
        .flatten();
    let pairs = filter_query.as_deref().map(query_pairs).unwrap_or_default();
    let window_end = Utc::now();
    let window_start = window_end - Duration::days(window_days(form, &pairs));

    let request =
        ScopeRequest::from_query(user, lookup(&pairs, "group"), lookup(&pairs, "project"));
    let subjects =
        crate::repositories::scope::membership::get_subject_scope(pool, &request).await?;
    let digest_scope = DigestScope {
        window_start,
        window_end,
        subject_ids: subjects.as_sql().map(<[String]>::to_vec),
        marketplace_key: (scope_kind == "marketplace")
            .then(|| scope_id.clone())
            .flatten(),
        skill: if scope_kind == "skill" {
            scope_id.clone()
        } else {
            lookup(&pairs, "skill").map(str::to_owned)
        },
        user_key: lookup(&pairs, "user_id").map(str::to_owned),
        model: lookup(&pairs, "model").map(str::to_owned),
        client_kind: lookup(&pairs, "client").map(str::to_owned),
        category: lookup(&pairs, "category").map(str::to_owned),
        outcome: lookup(&pairs, "outcome").map(str::to_owned),
    };
    Ok(ResolvedRequest {
        scope_label: trimmed(form.label.as_deref()),
        scope_kind,
        scope_id,
        window_start,
        window_end,
        digest_scope,
        filter_query,
    })
}

// Why: the row's id and the lease the generation task must present.
pub(super) struct Queued {
    pub(super) id: String,
    pub(super) lease_token: String,
}

pub(super) async fn queue(
    pool: &PgPool,
    user: &UserContext,
    resolved: ResolvedRequest,
) -> AdminResult<Queued> {
    let digest = get_report_digest(pool, &resolved.digest_scope).await?;
    let lease_token = uuid::Uuid::new_v4().to_string();
    let id = crate::repositories::analysis::reports::insert_report_request(
        pool,
        NewReport {
            lease_token: lease_token.clone(),
            scope_kind: resolved.scope_kind,
            scope_id: resolved.scope_id,
            scope_label: resolved.scope_label.clone(),
            window_start: resolved.window_start,
            window_end: resolved.window_end,
            requested_by: user.user_id.as_str().to_owned(),
            inputs: ReportDigestInputs {
                digest,
                filter_query: resolved.filter_query,
                filter_label: resolved.scope_label,
            },
        },
    )
    .await?;
    Ok(Queued { id, lease_token })
}

// Why: a regenerate is a fresh request over the same scope; the window is
// re-anchored to now with the same length so the report reads the present.
pub(super) fn regenerate_form(row: &AnalysisReportRow) -> ReportRequestForm {
    ReportRequestForm {
        scope_kind: Some(row.scope_kind.clone()),
        scope_id: row.scope_id.clone(),
        label: row.scope_label.clone(),
        days: Some((row.window_end - row.window_start).num_days().max(1)),
        query: row.inputs.filter_query.clone(),
    }
}
