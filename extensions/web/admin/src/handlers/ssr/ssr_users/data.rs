//! Data collection for the user roster and detail pages.
//!
//! Fans out the repository / runtime queries each page needs and shapes the raw
//! rows just enough for the view layer: roster aggregates, per-user assignment
//! and device extras, and the department list.

use sqlx::PgPool;

use crate::repositories;
use crate::services::marketplaces::load_marketplaces;

use super::super::types::{DepartmentGroup, UserAssignmentSummary, UserDeviceView};
use super::view;

type UserDetailExtras = (
    String,
    UserAssignmentSummary,
    Vec<UserDeviceView>,
    i64,
    Option<repositories::governance::effective::EffectivePermissions>,
);

pub(super) async fn load_user_groups(
    pool: &PgPool,
    users: &[crate::types::UserSummary],
) -> Vec<DepartmentGroup> {
    let aggregates = repositories::departments::list_user_management_aggregates(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user management aggregates");
            Vec::new()
        });

    let runtime = repositories::users::queries::list_user_runtime_aggregates(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user runtime aggregates");
            Vec::new()
        });

    // Why an empty list is not a safe default here: `resolve_marketplaces`
    // seeds from every YAML marketplace and *subtracts* the deny rows, so
    // losing the overrides does not lose grants — it loses the denials, and
    // every explicitly denied marketplace renders as granted. That is the one
    // failure direction on this page that over-reports access, so it degrades
    // to showing nothing rather than to showing everything.
    let (overrides, overrides_failed) =
        match repositories::departments::list_user_marketplace_overrides(pool).await {
            Ok(rows) => (rows, false),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to fetch marketplace overrides");
                (Vec::new(), true)
            },
        };

    let yaml_marketplaces: Vec<(String, String)> = if overrides_failed {
        Vec::new()
    } else {
        load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name))
            .collect()
    };

    let enriched_users =
        view::enrich_users(users, &aggregates, &runtime, &overrides, &yaml_marketplaces);
    view::group_by_department(enriched_users)
}

pub(super) async fn collect_user_detail_extras(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> crate::error::AdminHtmlResult<UserDetailExtras> {
    // Neither half of this tolerates a default. `department` is bound to a
    // <select> that the save handler reads straight back, so an empty value
    // does not merely display wrongly — it reassigns the user to Unassigned on
    // the next save, dropping every access rule their real department carried.
    // `roles` is fed to `compute_effective_permissions`, so an empty value
    // renders ALLOW/DENY rows under a caption promising they were computed
    // against this user's actual roles.
    let (roles, department) =
        repositories::users::queries::find_user_roles_department(pool, &d.user_id)
            .await?
            .unwrap_or_else(|| (Vec::new(), String::new()));

    let mut assignments = UserAssignmentSummary::default();
    let devices_count = if let Ok(rows) =
        repositories::departments::list_user_management_aggregates(pool).await
        && let Some(row) = rows.into_iter().find(|r| r.user_id == d.user_id.as_str())
    {
        assignments.skills_count = row.assigned_skills_count;
        row.devices_count
    } else {
        0i64
    };

    // Same fail-closed reasoning as the roster: losing the overrides loses the
    // *denials*, so an explicitly denied marketplace would render as granted.
    // Show none rather than all.
    let (user_overrides, overrides_failed): (
        Vec<repositories::departments::UserMarketplaceOverride>,
        bool,
    ) = match repositories::departments::list_user_marketplace_overrides(pool).await {
        Ok(rows) => (
            rows.into_iter()
                .filter(|o| o.user_id == d.user_id.as_str())
                .collect(),
            false,
        ),
        Err(e) => {
            tracing::warn!(error = %e, user_id = %d.user_id, "Failed to fetch marketplace overrides");
            (Vec::new(), true)
        },
    };
    let yaml_marketplaces: Vec<(String, String)> = if overrides_failed {
        Vec::new()
    } else {
        load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name))
            .collect()
    };
    let override_refs: Vec<&repositories::departments::UserMarketplaceOverride> =
        user_overrides.iter().collect();
    assignments.marketplaces = view::resolve_marketplaces(&yaml_marketplaces, &override_refs);
    assignments.marketplaces_count = assignments.marketplaces.len() as i64;

    let devices = collect_user_devices(pool, d).await;

    let effective = Some(
        repositories::governance::effective::compute_effective_permissions(
            pool, &d.user_id, &roles,
        )
        .await,
    );

    Ok((department, assignments, devices, devices_count, effective))
}

async fn collect_user_devices(pool: &PgPool, d: &crate::types::UserDetail) -> Vec<UserDeviceView> {
    let Ok(pats) = repositories::bridge::list_api_keys_for_user(pool, &d.user_id).await else {
        return Vec::new();
    };
    let app_links: std::collections::HashMap<
        String,
        (String, String, Option<chrono::DateTime<chrono::Utc>>),
    > = repositories::users::devices::list_device_app_links(pool, &d.user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: load device app_links failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|r| (r.device_id, (r.app_platform, r.app_version, r.last_seen_at)))
        .collect();

    pats.into_iter()
        .map(|row| {
            let link = app_links.get(&row.id);
            UserDeviceView {
                id: row.id,
                name: row.name,
                key_prefix: row.key_prefix,
                platform: link.map(|(p, _, _)| p.clone()),
                app_version: link.map(|(_, v, _)| v.clone()).filter(|v| !v.is_empty()),
                last_seen_at: link.and_then(|(_, _, ts)| *ts).or(row.last_used_at),
                revoked: row.revoked_at.is_some(),
            }
        })
        .collect()
}

/// The user's own department is always present in the returned list.
///
/// The list populates a bound `<select>`: an option that is missing cannot be
/// selected, so the browser silently falls back to the first one (Unassigned)
/// and the next save moves the user out of a department they were never
/// deliberately removed from. Degrading to a short list is survivable;
/// degrading to one that cannot represent the current value is not.
pub(super) async fn fetch_departments(pool: &PgPool, current: &str) -> Vec<String> {
    let mut names = repositories::departments::list_department_names(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: load departments failed"))
        .unwrap_or_default();
    if !current.is_empty() && !names.iter().any(|n| n == current) {
        names.push(current.to_owned());
    }
    names
}
