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
    Option<repositories::governance_grp::effective::EffectivePermissions>,
);

pub(super) async fn load_user_groups(
    pool: &PgPool,
    users: &[crate::types::UserSummary],
) -> Vec<DepartmentGroup> {
    let aggregates = repositories::list_user_management_aggregates(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user management aggregates");
            Vec::new()
        });

    let runtime = repositories::list_user_runtime_aggregates(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user runtime aggregates");
            Vec::new()
        });

    let overrides = repositories::list_user_marketplace_overrides(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch marketplace overrides");
            Vec::new()
        });

    let yaml_marketplaces: Vec<(String, String)> = load_marketplaces()
        .into_iter()
        .map(|m| (m.id.to_string(), m.name))
        .collect();

    let enriched_users =
        view::enrich_users(users, &aggregates, &runtime, &overrides, &yaml_marketplaces);
    view::group_by_department(enriched_users)
}

pub(super) async fn collect_user_detail_extras(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> UserDetailExtras {
    let (roles, department) = repositories::get_user_roles_department(pool, d.user_id.as_str())
        .await
        .inspect_err(|e| tracing::warn!(error = %e, user_id = %d.user_id, "ssr_users: get_user_roles_department failed"))
        .ok()
        .flatten()
        .unwrap_or_else(|| (Vec::new(), String::new()));

    let mut assignments = UserAssignmentSummary::default();
    let mut devices_count = 0i64;
    if let Ok(rows) = repositories::list_user_management_aggregates(pool).await {
        if let Some(row) = rows.into_iter().find(|r| r.user_id == d.user_id.as_str()) {
            assignments.skills_count = row.assigned_skills_count;
            devices_count = row.devices_count;
        }
    }

    let yaml_marketplaces: Vec<(String, String)> = load_marketplaces()
        .into_iter()
        .map(|m| (m.id.to_string(), m.name))
        .collect();
    let user_overrides: Vec<repositories::UserMarketplaceOverride> =
        repositories::list_user_marketplace_overrides(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|o| o.user_id == d.user_id.as_str())
            .collect();
    let override_refs: Vec<&repositories::UserMarketplaceOverride> =
        user_overrides.iter().collect();
    assignments.marketplaces = view::resolve_marketplaces(&yaml_marketplaces, &override_refs);
    assignments.marketplaces_count = assignments.marketplaces.len() as i64;

    let devices = collect_user_devices(pool, d).await;

    let effective = Some(
        repositories::governance_grp::effective::compute_effective_permissions(
            pool,
            d.user_id.as_str(),
            &roles,
            &department,
        )
        .await,
    );

    (department, assignments, devices, devices_count, effective)
}

async fn collect_user_devices(pool: &PgPool, d: &crate::types::UserDetail) -> Vec<UserDeviceView> {
    let Ok(pats) = repositories::bridge_grp::list_api_keys_for_user(pool, &d.user_id).await else {
        return Vec::new();
    };
    let app_links: std::collections::HashMap<
        String,
        (String, String, Option<chrono::DateTime<chrono::Utc>>),
    > = sqlx::query!(
        r#"SELECT device_id AS "device_id!",
                  app_platform AS "app_platform!",
                  app_version AS "app_version!",
                  last_seen_at
             FROM device_app_links
             WHERE user_id = $1"#,
        d.user_id.as_str(),
    )
    .fetch_all(pool)
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

pub(super) async fn fetch_departments(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar!("SELECT name FROM departments ORDER BY name")
        .fetch_all(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: load departments failed"))
        .unwrap_or_default()
}
