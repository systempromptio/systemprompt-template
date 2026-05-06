use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use chrono::{DateTime, Utc};

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::departments::DepartmentSummary;
use crate::types::{MarketplaceContext, UserContext};

use super::ssr_helpers::render_typed_page;
use super::ACCESS_DENIED_HTML;

fn forbidden() -> Response {
    (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response()
}

fn url_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Debug, Serialize)]
struct DepartmentsPageData {
    page: &'static str,
    title: &'static str,
    departments: Vec<DepartmentSummary>,
}

pub async fn management_departments_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let departments = repositories::list_departments(&pool)
        .await
        .unwrap_or_default();

    let data = DepartmentsPageData {
        page: "management-departments",
        title: "Departments",
        departments,
    };

    render_typed_page(
        &engine,
        "management-departments",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}

#[derive(Debug, Deserialize)]
pub struct DepartmentDetailQuery {
    #[serde(default)]
    tab: Option<String>,
}

#[derive(Debug, Serialize)]
struct DepartmentDetailPageData {
    page: &'static str,
    title: String,
    department: crate::types::departments::Department,
    members: Vec<crate::types::departments::DepartmentMember>,
    member_count: i64,
    assignments_url: String,
    tab: String,
    not_found: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct DeviceRowDb {
    id: String,
    name: String,
    key_prefix: String,
    user_id: String,
    user_email: Option<String>,
    department: Option<String>,
    platform: Option<String>,
    app_version: Option<String>,
    hostname: Option<String>,
    last_seen_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
struct DeviceRow {
    id: String,
    name: String,
    key_prefix: String,
    user_id: String,
    user_email: Option<String>,
    department: Option<String>,
    platform: Option<String>,
    app_version: Option<String>,
    hostname: Option<String>,
    last_seen_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked: bool,
}

#[derive(Debug, Serialize)]
struct ManagementDevicesPageData {
    page: &'static str,
    title: &'static str,
    devices: Vec<DeviceRow>,
    total: usize,
    online: usize,
}

pub async fn management_devices_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let rows: Vec<DeviceRowDb> = sqlx::query_as::<_, DeviceRowDb>(
        r"
        SELECT
            ak.id,
            ak.name,
            ak.key_prefix,
            ak.user_id,
            u.email::TEXT AS user_email,
            NULLIF(u.department, '') AS department,
            dal.app_platform AS platform,
            NULLIF(dal.app_version, '') AS app_version,
            NULLIF(dal.hostname, '') AS hostname,
            COALESCE(dal.last_seen_at, ak.last_used_at) AS last_seen_at,
            ak.created_at,
            ak.revoked_at
        FROM user_api_keys ak
        LEFT JOIN users u ON u.id = ak.user_id
        LEFT JOIN device_app_links dal ON dal.device_id = ak.id
        ORDER BY ak.revoked_at IS NOT NULL, ak.last_used_at DESC NULLS LAST, ak.created_at DESC
        ",
    )
    .fetch_all(&*pool)
    .await
    .unwrap_or_default();

    let now = Utc::now();
    let mut devices = Vec::with_capacity(rows.len());
    let mut online = 0usize;
    for r in rows {
        let revoked = r.revoked_at.is_some();
        if !revoked {
            if let Some(ts) = r.last_seen_at {
                if (now - ts).num_minutes() < 5 {
                    online += 1;
                }
            }
        }
        devices.push(DeviceRow {
            id: r.id,
            name: r.name,
            key_prefix: r.key_prefix,
            user_id: r.user_id,
            user_email: r.user_email,
            department: r.department,
            platform: r.platform,
            app_version: r.app_version,
            hostname: r.hostname,
            last_seen_at: r.last_seen_at,
            created_at: r.created_at,
            revoked,
        });
    }
    let total = devices.len();

    let data = ManagementDevicesPageData {
        page: "management-devices",
        title: "Devices",
        devices,
        total,
        online,
    };
    render_typed_page(&engine, "management-devices", &data, &user_ctx, &mkt_ctx)
}

#[derive(Debug, Serialize)]
struct AssignedItem {
    id: String,
    name: String,
    description: Option<String>,
    departments: i64,
    users: i64,
    coverage_label: String,
}

#[derive(Debug, Serialize)]
struct ManagementListPageData {
    page: &'static str,
    title: &'static str,
    items: Vec<AssignedItem>,
    total: usize,
}

async fn fetch_assigned_skills(pool: &PgPool) -> Vec<AssignedItem> {
    let query = r"
        SELECT
            base.id,
            base.name,
            base.description,
            COALESCE(d.cnt, 0)::BIGINT,
            COALESCE(u.cnt, 0)::BIGINT
        FROM (
            SELECT DISTINCT skill_id AS id, name, description
            FROM user_skills
        ) base
        LEFT JOIN (
            SELECT entity_id, COUNT(*)::BIGINT AS cnt
            FROM access_control_rules
            WHERE entity_type = 'skill' AND rule_type = 'department' AND access = 'allow'
            GROUP BY entity_id
        ) d ON d.entity_id = base.id
        LEFT JOIN (
            SELECT entity_id, COUNT(*)::BIGINT AS cnt
            FROM access_control_rules
            WHERE entity_type = 'skill' AND rule_type = 'user' AND access = 'allow'
            GROUP BY entity_id
        ) u ON u.entity_id = base.id
        ORDER BY base.name
        ";

    let rows: Vec<(String, String, Option<String>, i64, i64)> = sqlx::query_as(query)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    rows.into_iter()
        .map(|(id, name, description, departments, users)| AssignedItem {
            coverage_label: coverage_label("skill", departments, users),
            id,
            name,
            description,
            departments,
            users,
        })
        .collect()
}

fn coverage_label(entity_type: &str, departments: i64, users: i64) -> String {
    if departments == 0 && users == 0 {
        format!("Unassigned · open via access matrix to assign this {entity_type}")
    } else {
        format!(
            "{} dept{} · {} user{}",
            departments,
            if departments == 1 { "" } else { "s" },
            users,
            if users == 1 { "" } else { "s" }
        )
    }
}

fn fetch_marketplace_items() -> Vec<AssignedItem> {
    crate::services::marketplaces::load_marketplaces()
        .into_iter()
        .map(|mp| {
            let members = (mp.plugins.include.len()
                + mp.skills.include.len()
                + mp.agents.include.len()
                + mp.mcp_servers.len()) as i64;
            AssignedItem {
                id: mp.id.as_str().to_string(),
                name: mp.name,
                description: Some(mp.description),
                departments: members,
                users: 0,
                coverage_label: format!(
                    "{} member{}",
                    members,
                    if members == 1 { "" } else { "s" }
                ),
            }
        })
        .collect()
}

pub async fn management_skills_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    let items = fetch_assigned_skills(&pool).await;
    let total = items.len();
    let data = ManagementListPageData {
        page: "management-skills",
        title: "Skills",
        items,
        total,
    };
    render_typed_page(&engine, "management-skills", &data, &user_ctx, &mkt_ctx)
}

pub async fn management_marketplaces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }
    let items = fetch_marketplace_items();
    let total = items.len();
    let data = ManagementListPageData {
        page: "management-marketplaces",
        title: "Marketplaces",
        items,
        total,
    };
    render_typed_page(&engine, "management-marketplaces", &data, &user_ctx, &mkt_ctx)
}

pub async fn management_department_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Query(q): Query<DepartmentDetailQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let Ok(Some(department)) = repositories::get_department(&pool, &id).await else {
        return (
            StatusCode::NOT_FOUND,
            Html("<h1>Department not found</h1>"),
        )
            .into_response();
    };

    let members = repositories::list_department_members(&pool, &department.name)
        .await
        .unwrap_or_default();
    let member_count = members.len() as i64;

    let assignments_url = format!(
        "/admin/access-control?department={}",
        url_escape(&department.name)
    );

    let title = format!("Department · {}", department.name);
    let data = DepartmentDetailPageData {
        page: "management-department-detail",
        title,
        department,
        members,
        member_count,
        assignments_url,
        tab: q.tab.unwrap_or_else(|| "members".to_string()),
        not_found: false,
    };

    render_typed_page(
        &engine,
        "management-department-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}
