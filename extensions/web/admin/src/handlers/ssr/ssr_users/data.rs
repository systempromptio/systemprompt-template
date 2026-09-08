//! Data collection for the user roster and detail pages.
//!
//! Fans out the repository queries each page needs and shapes the raw rows
//! into view types: the roster's rows from the account list plus its
//! department, token and runtime aggregates, and the per-user assignment,
//! token and effective-permission extras the detail page's tabs read.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::departments::UserMarketplaceOverride;
use crate::services::marketplaces::load_marketplaces;
use crate::types::departments::DEFAULT_DEPARTMENT;

use super::super::format::format_token_total;
use super::super::types::{UserAssignmentSummary, UserMarketplaceRef, UserTokenView};
use super::context::RosterRowView;
use super::roster_view::{initials, relative};

// Why: an empty list is not a safe default for the overrides:
// `resolve_marketplaces` seeds from every YAML marketplace and *subtracts*
// the deny rows, so losing the overrides does not lose grants — it loses the
// denials, and every explicitly denied marketplace renders as granted. That is
// the one failure direction on these pages that over-reports access, so it
// degrades to showing nothing rather than to showing everything.
async fn load_overrides(pool: &PgPool) -> (Vec<UserMarketplaceOverride>, Vec<(String, String)>) {
    match repositories::departments::list_user_marketplace_overrides(pool).await {
        Ok(rows) => {
            let yaml = load_marketplaces()
                .into_iter()
                .map(|m| (m.id.to_string(), m.name))
                .collect();
            (rows, yaml)
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to fetch marketplace overrides");
            (Vec::new(), Vec::new())
        },
    }
}

fn token_tone(ts: Option<chrono::DateTime<chrono::Utc>>) -> &'static str {
    ts.map_or("muted", |t| {
        let age = chrono::Utc::now() - t;
        if age < chrono::Duration::minutes(5) {
            "ok"
        } else if age < chrono::Duration::hours(1) {
            "info"
        } else {
            "warn"
        }
    })
}

pub(super) fn resolve_marketplaces(
    yaml_defaults: &[(String, String)],
    overrides: &[&UserMarketplaceOverride],
) -> Vec<UserMarketplaceRef> {
    let mut entries: Vec<UserMarketplaceRef> = yaml_defaults
        .iter()
        .map(|(id, name)| UserMarketplaceRef {
            id: id.clone(),
            name: name.clone(),
            source: "default",
        })
        .collect();
    for ovr in overrides {
        match ovr.access.as_str() {
            "allow" if !entries.iter().any(|e| e.id == ovr.entity_id) => {
                let name = yaml_defaults
                    .iter()
                    .find(|(id, _)| id == &ovr.entity_id)
                    .map_or_else(|| ovr.entity_id.clone(), |(_, n)| n.clone());
                entries.push(UserMarketplaceRef {
                    id: ovr.entity_id.clone(),
                    name,
                    source: "override",
                });
            },
            "deny" => entries.retain(|e| e.id != ovr.entity_id),
            _ => {},
        }
    }
    entries
}

pub(super) async fn load_roster_rows(
    pool: &PgPool,
    users: &[crate::types::UserSummary],
) -> Vec<RosterRowView> {
    let aggregates = repositories::departments::list_user_management_aggregates(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "Failed to fetch user management aggregates"))
        .unwrap_or_default();
    let runtime = repositories::users::queries::list_user_runtime_aggregates(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "Failed to fetch user runtime aggregates"))
        .unwrap_or_default();
    let (overrides, yaml) = load_overrides(pool).await;

    let agg_map: HashMap<&str, _> = aggregates.iter().map(|a| (a.user_id.as_str(), a)).collect();
    let rt_map: HashMap<&str, _> = runtime.iter().map(|r| (r.user_id.as_str(), r)).collect();
    let mut ovr_map: HashMap<&str, Vec<&UserMarketplaceOverride>> = HashMap::new();
    for o in &overrides {
        ovr_map.entry(o.user_id.as_str()).or_default().push(o);
    }

    users
        .iter()
        .map(|u| {
            let id = u.user_id.as_str();
            let agg = agg_map.get(id);
            let rt = rt_map.get(id);
            let name = u.display_name.clone().unwrap_or_else(|| id.to_owned());
            let department = agg
                .map(|a| a.department.clone())
                .filter(|d| !d.is_empty())
                .unwrap_or_else(|| DEFAULT_DEPARTMENT.to_owned());
            let marketplaces = resolve_marketplaces(
                &yaml,
                ovr_map.get(id).map(Vec::as_slice).unwrap_or_default(),
            );
            let model_tokens = rt.map_or(0, |r| r.lifetime_tokens);
            RosterRowView {
                initials: initials(&name),
                email: u
                    .email
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                detail_url: super::detail_url(id),
                has_roles: !u.roles.is_empty(),
                roles: u.roles.clone(),
                department_url: format!(
                    "{}?department={}",
                    super::BASE_URL,
                    urlencoding::encode(&department)
                ),
                department,
                marketplaces_count: marketplaces.len(),
                tokens_count: agg.map_or(0, |a| a.tokens_count),
                token_tone: token_tone(rt.and_then(|r| r.newest_token_used_at)),
                model_tokens_display: format_token_total(model_tokens),
                model_tokens_raw: model_tokens,
                last_active_epoch: u.last_active.timestamp(),
                sessions: u.sessions,
                last_active: relative(u.last_active.timestamp()),
                last_active_title: u.last_active.to_rfc3339(),
                is_active: u.is_active,
                status_label: if u.is_active { "Active" } else { "Inactive" },
                status_tone: if u.is_active { "ok" } else { "muted" },
                name,
                user_id: u.user_id.clone(),
            }
        })
        .collect()
}

pub(super) struct DetailExtras {
    pub department: String,
    pub roles: Vec<String>,
    pub assignments: UserAssignmentSummary,
    pub tokens_count: i64,
}

pub(super) async fn load_detail_extras(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> crate::error::AdminHtmlResult<DetailExtras> {
    // Why: neither half of this tolerates a default. `department` is bound to
    // a <select> that the save handler reads straight back, so an empty value
    // does not merely display wrongly — it reassigns the user to the Default
    // department on the next save, dropping every access rule their real
    // department carried. `roles` feeds `compute_effective_permissions`, so an
    // empty value renders ALLOW/DENY rows under a caption promising they were
    // computed against this user's actual roles.
    let (roles, department) =
        repositories::users::queries::find_user_roles_department(pool, &d.user_id)
            .await?
            .unwrap_or_else(|| (Vec::new(), String::new()));
    let department = if department.is_empty() {
        DEFAULT_DEPARTMENT.to_owned()
    } else {
        department
    };

    let mut assignments = UserAssignmentSummary::default();
    let tokens_count = if let Ok(rows) =
        repositories::departments::list_user_management_aggregates(pool).await
        && let Some(row) = rows.into_iter().find(|r| r.user_id == d.user_id.as_str())
    {
        assignments.skills_count = row.assigned_skills_count;
        row.tokens_count
    } else {
        0i64
    };

    let (overrides, yaml) = load_overrides(pool).await;
    let mine: Vec<&UserMarketplaceOverride> = overrides
        .iter()
        .filter(|o| o.user_id == d.user_id.as_str())
        .collect();
    assignments.marketplaces = resolve_marketplaces(&yaml, &mine);
    assignments.marketplaces_count = i64::try_from(assignments.marketplaces.len()).unwrap_or(0);

    Ok(DetailExtras {
        department,
        roles,
        assignments,
        tokens_count,
    })
}

pub(super) async fn load_user_tokens(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> Vec<UserTokenView> {
    repositories::access_tokens::list_api_keys_for_user(pool, &d.user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: token listing failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|row| UserTokenView {
            id: row.id,
            name: row.name,
            key_prefix: row.key_prefix,
            last_used_at: row.last_used_at,
            revoked: row.revoked_at.is_some(),
        })
        .collect()
}

// Why: the user's own department is always present in the returned list. The
// list populates a bound `<select>`: an option that is missing cannot be
// selected, so the browser silently falls back to whichever option happens to
// be first and the next save moves the user out of a department they were
// never deliberately removed from.
pub(super) async fn load_departments(pool: &PgPool, current: &str) -> Vec<String> {
    let mut names = repositories::departments::list_department_names(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: load departments failed"))
        .unwrap_or_default();
    if !names.iter().any(|n| n == current) {
        names.push(current.to_owned());
    }
    names
}
