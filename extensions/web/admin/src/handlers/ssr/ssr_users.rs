use std::sync::Arc;

use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use super::types::{EnrichedUserView, UserDetailPageData, UserRuntimeView, UsersPageData};

fn freshness_for(ts: Option<chrono::DateTime<chrono::Utc>>) -> &'static str {
    ts.map_or("never", |t| {
        let age = chrono::Utc::now() - t;
        if age < chrono::Duration::minutes(5) {
            "fresh"
        } else if age < chrono::Duration::hours(1) {
            "idle"
        } else {
            "stale"
        }
    })
}

fn enrich_users(
    users: &[crate::types::UserSummary],
    aggregates: &[repositories::UserManagementAggregate],
    runtime: &[repositories::UserRuntimeAggregate],
) -> Vec<EnrichedUserView> {
    let agg_map: std::collections::HashMap<&str, &repositories::UserManagementAggregate> =
        aggregates.iter().map(|a| (a.user_id.as_str(), a)).collect();
    let rt_map: std::collections::HashMap<&str, &repositories::UserRuntimeAggregate> =
        runtime.iter().map(|r| (r.user_id.as_str(), r)).collect();

    users
        .iter()
        .map(|u| {
            let agg = agg_map.get(u.user_id.as_str());
            let rt = rt_map.get(u.user_id.as_str());
            let device_freshness =
                freshness_for(rt.and_then(|r| r.newest_device_seen_at)).to_string();
            EnrichedUserView {
                user_id: u.user_id.to_string(),
                display_name: u.display_name.clone(),
                email: u.email.as_ref().map(ToString::to_string),
                roles: u.roles.clone(),
                is_active: u.is_active,
                last_active: u.last_active.to_rfc3339(),
                total_events: u.total_events,
                last_tool: u.last_tool.clone(),
                custom_skills_count: u.custom_skills_count,
                preferred_client: u.preferred_client.clone(),
                prompts: u.prompts,
                sessions: u.sessions,
                bytes: u.bytes,
                logins: u.logins,
                department: agg.map(|a| a.department.clone()).unwrap_or_default(),
                assigned_skills_count: agg.map_or(0, |a| a.assigned_skills_count),
                assigned_marketplaces_count: agg.map_or(0, |a| a.assigned_marketplaces_count),
                devices_count: agg.map_or(0, |a| a.devices_count),
                connected_agents: rt.map_or(0, |r| r.connected_agents),
                total_agents: rt.map_or(0, |r| r.total_agents),
                lifetime_tokens: rt.map_or(0, |r| r.lifetime_tokens),
                device_freshness,
            }
        })
        .collect()
}

pub async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let users = repositories::list_users(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list users");
        vec![]
    });

    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let total_events: i64 = users.iter().map(|u| u.total_events).sum();

    let aggregates = repositories::list_user_management_aggregates(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user management aggregates");
            Vec::new()
        });

    let runtime = repositories::list_user_runtime_aggregates(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user runtime aggregates");
            Vec::new()
        });

    let enriched_users = enrich_users(&users, &aggregates, &runtime);

    let data = UsersPageData {
        page: "users",
        title: "Users",
        users: enriched_users,
        total_users,
        active_users,
        total_events,
    };

    let mut value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "page_stats".to_string(),
            serde_json::json!([
                {"value": total_users, "label": "Users"},
                {"value": active_users, "label": "Active"},
                {"value": total_events, "label": "Events"},
            ]),
        );
    }
    super::render_page(&engine, "users", &value, &user_ctx, &mkt_ctx)
}

type UserDetailExtras = (
    String,
    super::types::UserAssignmentSummary,
    Vec<super::types::UserDeviceView>,
    i64,
    Option<repositories::governance_grp::effective::EffectivePermissions>,
);

async fn collect_user_detail_extras(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> UserDetailExtras {
    let (roles, department) = repositories::get_user_roles_department(pool, d.user_id.as_str())
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| (Vec::new(), String::new()));

    let mut assignments = super::types::UserAssignmentSummary::default();
    let mut devices_count = 0i64;
    if let Ok(rows) = repositories::list_user_management_aggregates(pool).await {
        if let Some(row) = rows.into_iter().find(|r| r.user_id == d.user_id.as_str()) {
            assignments.skills_count = row.assigned_skills_count;
            assignments.marketplaces_count = row.assigned_marketplaces_count;
            devices_count = row.devices_count;
        }
    }

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

async fn collect_user_devices(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> Vec<super::types::UserDeviceView> {
    let Ok(pats) = repositories::cowork_grp::list_api_keys_for_user(pool, &d.user_id).await else {
        return Vec::new();
    };
    let app_links: std::collections::HashMap<
        String,
        (String, String, Option<chrono::DateTime<chrono::Utc>>),
    > = sqlx::query_as::<_, (String, String, String, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT device_id, app_platform, app_version, last_seen_at FROM device_app_links WHERE user_id = $1",
    )
    .bind(d.user_id.as_str())
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, p, v, ts)| (id, (p, v, ts)))
    .collect();

    pats.into_iter()
        .map(|row| {
            let link = app_links.get(&row.id);
            super::types::UserDeviceView {
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

async fn fetch_departments(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar::<_, String>("SELECT name FROM departments ORDER BY name")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

pub async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> Response {
    if !user_ctx.is_admin && Some(user_ctx.user_id.as_str()) != params.id() {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(
                "<h1>Access Denied</h1><p>You can only view your own profile.</p>",
            ),
        )
            .into_response();
    }

    let Some(id) = params.id() else {
        let data = UserDetailPageData {
            page: "user-detail",
            title: "User Detail",
            user: None,
            gamification: None,
            not_found: true,
            user_department: String::new(),
            user_assignments: super::types::UserAssignmentSummary::default(),
            user_devices: Vec::new(),
            user_devices_count: 0,
            departments: Vec::new(),
            runtime: None,
        };
        let value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
        return super::render_page(&engine, "user-detail", &value, &user_ctx, &mkt_ctx);
    };
    let user_id = UserId::new(id);

    let detail = repositories::find_user_detail(&pool, &user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user detail");
        })
        .ok()
        .flatten();
    let gamification: Option<crate::types::UserGamificationProfile> = None;

    let not_found = detail.is_none();

    let (user_department, user_assignments, user_devices, user_devices_count, effective) =
        match detail.as_ref() {
            Some(d) => collect_user_detail_extras(&pool, d).await,
            None => (
                String::new(),
                super::types::UserAssignmentSummary::default(),
                Vec::new(),
                0,
                None,
            ),
        };

    let runtime = match detail.as_ref() {
        Some(d) => repositories::get_user_runtime_detail(&pool, d.user_id.as_str())
            .await
            .ok()
            .map(|r| UserRuntimeView {
                connected_agents: r.connected_agents,
                total_agents: r.total_agents,
                tokens_in: r.tokens_in,
                tokens_out: r.tokens_out,
                last_bridge_version: r.last_bridge_version,
                last_os: r.last_os,
                last_hostname: r.last_hostname,
                last_heartbeat_at: r.last_heartbeat_at.map(|t| t.to_rfc3339()),
            }),
        None => None,
    };

    let departments = fetch_departments(&pool).await;

    let data = UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: detail,
        gamification,
        not_found,
        user_department,
        user_assignments,
        user_devices,
        user_devices_count,
        departments,
        runtime,
    };
    let mut value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
    if let (serde_json::Value::Object(ref mut map), Some(eff)) = (&mut value, effective) {
        map.insert(
            "effective_permissions".to_string(),
            serde_json::to_value(&eff).unwrap_or(serde_json::Value::Null),
        );
        map.insert(
            "has_effective_permissions".to_string(),
            serde_json::Value::Bool(
                !eff.gateway_routes.is_empty() || !eff.mcp_servers.is_empty(),
            ),
        );
    }
    super::render_page(&engine, "user-detail", &value, &user_ctx, &mkt_ctx)
}
