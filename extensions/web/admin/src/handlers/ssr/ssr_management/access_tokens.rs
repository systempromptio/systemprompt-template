//! Access-token view model for `/admin/access/tokens`.
//!
//! Loads every issued personal access token joined to its owner, reshapes the
//! rows for the template, computes the per-owner rowspans that group a user's
//! tokens in the table, and counts the active and soon-to-expire ones.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::users::access_tokens::{self, AccessTokenRowDb};

const EXPIRING_SOON_DAYS: i64 = 30;

#[derive(Debug, Serialize)]
pub(super) struct AccessTokenRow {
    id: String,
    name: String,
    key_prefix: String,
    user_id: UserId,
    user_email: Option<String>,
    department: Option<String>,
    last_used_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked: bool,
    owner_rowspan: u32,
    group_start: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct TokenUserOption {
    user_id: UserId,
    label: String,
}

// Why: Headline counts for the stat cards: total, active, and expiring soon.
#[derive(Debug, Default)]
pub(super) struct TokenCounts {
    pub total: usize,
    pub active: usize,
    pub expiring_soon: usize,
}

pub(super) async fn load_access_tokens(pool: &PgPool) -> Vec<AccessTokenRowDb> {
    access_tokens::list_access_tokens(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_management: load access tokens failed"))
        .unwrap_or_default()
}

pub(super) fn build_token_rows(rows: Vec<AccessTokenRowDb>) -> (Vec<AccessTokenRow>, TokenCounts) {
    let soon = Utc::now() + Duration::days(EXPIRING_SOON_DAYS);
    let mut tokens = Vec::with_capacity(rows.len());
    let mut counts = TokenCounts::default();
    for r in rows {
        let revoked = r.revoked_at.is_some();
        if !revoked {
            counts.active += 1;
            if let Some(ts) = r.expires_at
                && ts <= soon
            {
                counts.expiring_soon += 1;
            }
        }
        tokens.push(AccessTokenRow {
            id: r.id,
            name: r.name,
            key_prefix: r.key_prefix,
            user_id: r.user_id,
            user_email: r.user_email,
            department: r.department,
            last_used_at: r.last_used_at,
            expires_at: r.expires_at,
            created_at: r.created_at,
            revoked,
            owner_rowspan: 0,
            group_start: false,
        });
    }
    counts.total = tokens.len();
    (tokens, counts)
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

fn owner_key(t: &AccessTokenRow) -> &str {
    t.user_email.as_deref().unwrap_or(t.user_id.as_str())
}

pub(super) fn compute_owner_rowspans(tokens: &mut [AccessTokenRow]) {
    let mut i = 0;
    while i < tokens.len() {
        let key = owner_key(&tokens[i]).to_owned();
        let mut j = i + 1;
        while j < tokens.len() && owner_key(&tokens[j]) == key {
            j += 1;
        }
        let span = u32::try_from(j - i).unwrap_or(1);
        tokens[i].owner_rowspan = span;
        tokens[i].group_start = true;
        i = j;
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ManagementAccessTokensPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub tokens: Vec<AccessTokenRow>,
    pub total: usize,
    pub active: usize,
    pub expiring_soon: usize,
    pub user_options: Vec<TokenUserOption>,
    pub department_options: Vec<String>,
}
