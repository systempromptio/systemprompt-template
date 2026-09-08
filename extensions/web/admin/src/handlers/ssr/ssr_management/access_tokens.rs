//! Access-token view model for `/admin/access-tokens`.
//!
//! Loads every issued personal access token joined to its owner, reshapes the
//! rows for the template, counts the active, expiring and revoked ones, and
//! narrows the listing by the status, department and search facets in the
//! URL. One flat row per token: the owner is a cell, not a rowspan group, so
//! sorting and filtering never have to keep a group together.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::SelectOptionView;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::users::access_tokens::{self, AccessTokenRowDb};

const EXPIRING_SOON_DAYS: i64 = 30;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct AccessTokensQuery {
    pub status: Option<String>,
    pub department: Option<String>,
    pub q: Option<String>,
}

impl AccessTokensQuery {
    fn field(&self, name: &str) -> Option<&str> {
        let value = match name {
            "status" => self.status.as_deref(),
            "department" => self.department.as_deref(),
            "q" => self.q.as_deref(),
            _ => None,
        };
        value.map(str::trim).filter(|v| !v.is_empty())
    }

    pub(super) fn search(&self) -> Option<&str> {
        self.field("q")
    }

    pub(super) fn any_applied(&self) -> bool {
        ["status", "department", "q"]
            .iter()
            .any(|f| self.field(f).is_some())
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct AccessTokenRow {
    id: String,
    name: String,
    key_prefix: String,
    user_id: UserId,
    owner: String,
    owner_url: String,
    department: String,
    department_url: String,
    last_used_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked: bool,
    expiring_soon: bool,
    status_label: &'static str,
    status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct TokenUserOption {
    user_id: UserId,
    label: String,
}

// Why: Headline counts for the KPI tiles: total, active, expiring and revoked.
#[derive(Debug, Default)]
pub(super) struct TokenCounts {
    pub total: usize,
    pub active: usize,
    pub expiring_soon: usize,
    pub revoked: usize,
}

pub(super) async fn load_access_tokens(pool: &PgPool) -> Vec<AccessTokenRowDb> {
    access_tokens::list_access_tokens(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_management: load access tokens failed"))
        .unwrap_or_default()
}

pub(super) fn build_token_rows(rows: Vec<AccessTokenRowDb>) -> Vec<AccessTokenRow> {
    let soon = Utc::now() + Duration::days(EXPIRING_SOON_DAYS);
    rows.into_iter()
        .map(|r| {
            let revoked = r.revoked_at.is_some();
            let expired = r.expires_at.is_some_and(|ts| ts <= Utc::now());
            let expiring_soon = !revoked && !expired && r.expires_at.is_some_and(|ts| ts <= soon);
            let department = r.department.unwrap_or_default();
            let encoded_user = urlencoding::encode(r.user_id.as_str()).into_owned();
            AccessTokenRow {
                owner: r
                    .user_email
                    .clone()
                    .unwrap_or_else(|| r.user_id.as_str().to_owned()),
                owner_url: format!("/admin/user?id={encoded_user}"),
                department_url: format!(
                    "/admin/access-control?department={}",
                    urlencoding::encode(&department)
                ),
                department,
                status_label: if revoked {
                    "Revoked"
                } else if expired {
                    "Expired"
                } else if expiring_soon {
                    "Expiring"
                } else {
                    "Active"
                },
                status_tone: if revoked {
                    "muted"
                } else if expired {
                    "err"
                } else if expiring_soon {
                    "warn"
                } else {
                    "ok"
                },
                id: r.id,
                name: r.name,
                key_prefix: r.key_prefix,
                user_id: r.user_id,
                last_used_at: r.last_used_at,
                expires_at: r.expires_at,
                created_at: r.created_at,
                revoked,
                expiring_soon,
            }
        })
        .collect()
}

pub(super) fn counts(rows: &[AccessTokenRow]) -> TokenCounts {
    TokenCounts {
        total: rows.len(),
        active: rows.iter().filter(|t| !t.revoked).count(),
        expiring_soon: rows.iter().filter(|t| t.expiring_soon).count(),
        revoked: rows.iter().filter(|t| t.revoked).count(),
    }
}

pub(super) fn department_names(rows: &[AccessTokenRow]) -> Vec<String> {
    let mut names: Vec<String> = rows
        .iter()
        .map(|t| t.department.clone())
        .filter(|d| !d.is_empty())
        .collect();
    names.sort();
    names.dedup();
    names
}

pub(super) fn filtered(rows: &[AccessTokenRow], query: &AccessTokensQuery) -> Vec<AccessTokenRow> {
    let needle = query.search().map(str::to_lowercase);
    rows.iter()
        .filter(|t| match query.field("status") {
            Some("active") => !t.revoked,
            Some("expiring") => t.expiring_soon,
            Some("revoked") => t.revoked,
            _ => true,
        })
        .filter(|t| query.field("department").is_none_or(|d| t.department == d))
        .filter(|t| {
            needle.as_ref().is_none_or(|n| {
                format!(
                    "{} {} {} {}",
                    t.name,
                    t.key_prefix,
                    t.owner,
                    t.user_id.as_str()
                )
                .to_lowercase()
                .contains(n)
            })
        })
        .cloned()
        .collect()
}

fn options(
    entries: &[(&str, &str)],
    all_label: &str,
    selected: Option<&str>,
) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(entries.iter().map(|(value, label)| SelectOptionView {
        value: (*value).to_owned(),
        label: (*label).to_owned(),
        selected: selected == Some(*value),
    }));
    out
}

pub(super) fn status_options(query: &AccessTokensQuery) -> Vec<SelectOptionView> {
    options(
        &[
            ("active", "Active"),
            ("expiring", "Expiring within 30 days"),
            ("revoked", "Revoked"),
        ],
        "Any status",
        query.field("status"),
    )
}

pub(super) fn department_options(
    departments: &[String],
    query: &AccessTokensQuery,
) -> Vec<SelectOptionView> {
    let entries: Vec<(&str, &str)> = departments
        .iter()
        .map(|d| (d.as_str(), d.as_str()))
        .collect();
    options(&entries, "All departments", query.field("department"))
}

pub(super) async fn load_token_user_options(pool: &PgPool) -> Vec<TokenUserOption> {
    access_tokens::list_token_user_options(pool)
        .await
        .inspect_err(
            |e| tracing::warn!(error = %e, "ssr_management: load token user options failed"),
        )
        .unwrap_or_default()
        .into_iter()
        .map(|r| {
            let label = match (r.display.as_deref(), r.email.as_deref()) {
                (Some(d), Some(e)) => format!("{d} ({e})"),
                (Some(d), None) => d.to_owned(),
                (None, Some(e)) => e.to_owned(),
                (None, None) => r.uid.clone(),
            };
            TokenUserOption {
                user_id: UserId::new(r.uid),
                label,
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub(super) struct ManagementAccessTokensPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub total: usize,
    pub active: usize,
    pub expiring_soon: usize,
    pub revoked: usize,
    pub status_options: Vec<SelectOptionView>,
    pub department_options: Vec<SelectOptionView>,
    pub search: String,
    pub filters_applied: bool,
    pub clear_url: &'static str,
    pub shown: usize,
    pub has_rows: bool,
    pub tokens: Vec<AccessTokenRow>,
    pub user_options: Vec<TokenUserOption>,
    pub can_write: bool,
}
