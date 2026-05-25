use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;
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

#[derive(Debug, Serialize)]
struct DepartmentDetailPageData {
    page: &'static str,
    title: String,
    department: crate::types::departments::Department,
    members: Vec<crate::types::departments::DepartmentMember>,
    member_count: i64,
    assignments_url: String,
    top_tools: Vec<crate::types::departments::DepartmentTopTool>,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_requests: i64,
    total_cost_microdollars: i64,
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
    enrolled_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
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
    enrolled_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked: bool,
    owner_rowspan: u32,
    group_start: bool,
}

fn build_device_rows(rows: Vec<DeviceRowDb>) -> (Vec<DeviceRow>, usize) {
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
            enrolled_at: r.enrolled_at,
            expires_at: r.expires_at,
            created_at: r.created_at,
            revoked,
            owner_rowspan: 0,
            group_start: false,
        });
    }
    (devices, online)
}

async fn load_device_user_options(pool: &PgPool) -> Vec<DeviceUserOption> {
    sqlx::query!(
        r#"
        SELECT u.id::TEXT AS "uid!",
               u.email::TEXT AS "email?",
               COALESCE(NULLIF(u.display_name, ''), NULLIF(u.full_name, ''), NULLIF(u.name, '')) AS "display?"
        FROM users u
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        ORDER BY COALESCE(NULLIF(u.display_name, ''), u.email::TEXT, u.id::TEXT)
        "#,
    )
    .fetch_all(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "ssr_management: load device user options failed"))
    .unwrap_or_default()
    .into_iter()
    .map(|r| {
        let label = match (r.display.as_deref(), r.email.as_deref()) {
            (Some(d), Some(e)) => format!("{d} ({e})"),
            (Some(d), None) => d.to_string(),
            (None, Some(e)) => e.to_string(),
            (None, None) => r.uid.clone(),
        };
        DeviceUserOption {
            user_id: r.uid,
            label,
        }
    })
    .collect()
}

fn owner_key(d: &DeviceRow) -> &str {
    d.user_email.as_deref().unwrap_or(d.user_id.as_str())
}

fn compute_owner_rowspans(devices: &mut [DeviceRow]) {
    let mut i = 0;
    while i < devices.len() {
        let key = owner_key(&devices[i]).to_owned();
        let mut j = i + 1;
        while j < devices.len() && owner_key(&devices[j]) == key {
            j += 1;
        }
        let span = u32::try_from(j - i).unwrap_or(1);
        devices[i].owner_rowspan = span;
        devices[i].group_start = true;
        i = j;
    }
}

#[derive(Debug, Serialize)]
struct DeviceUserOption {
    user_id: String,
    label: String,
}

#[derive(Debug, Serialize)]
struct ManagementDevicesPageData {
    page: &'static str,
    title: &'static str,
    devices: Vec<DeviceRow>,
    total: usize,
    online: usize,
    user_options: Vec<DeviceUserOption>,
    department_options: Vec<String>,
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

    let rows: Vec<DeviceRowDb> = sqlx::query_as!(
        DeviceRowDb,
        r#"
        SELECT
            ak.id AS "id!",
            ak.name AS "name!",
            ak.key_prefix AS "key_prefix!",
            ak.user_id AS "user_id!",
            u.email::TEXT AS "user_email?",
            NULLIF(upe.department, '') AS "department?",
            dal.app_platform AS "platform?",
            NULLIF(dal.app_version, '') AS "app_version?",
            NULLIF(dal.hostname, '') AS "hostname?",
            COALESCE(dal.last_seen_at, ak.last_used_at) AS "last_seen_at?",
            dal.enrolled_at AS "enrolled_at?",
            ak.expires_at AS "expires_at?",
            ak.created_at AS "created_at?",
            ak.revoked_at AS "revoked_at?"
        FROM user_api_keys ak
        LEFT JOIN users u ON u.id = ak.user_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        LEFT JOIN device_app_links dal ON dal.device_id = ak.id
        ORDER BY ak.revoked_at IS NOT NULL,
                 COALESCE(u.email::TEXT, ak.user_id::TEXT),
                 ak.created_at DESC
        "#,
    )
    .fetch_all(&*pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "ssr_management: load devices failed"))
    .unwrap_or_default();

    let (mut devices, online) = build_device_rows(rows);
    let total = devices.len();
    compute_owner_rowspans(&mut devices);

    let user_options = load_device_user_options(&pool).await;

    let department_options: Vec<String> = repositories::list_departments(&pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.name)
        .collect();

    let data = ManagementDevicesPageData {
        page: "devices",
        title: "Devices",
        devices,
        total,
        online,
        user_options,
        department_options,
    };
    render_typed_page(&engine, "management-devices", &data, &user_ctx, &mkt_ctx)
}

pub async fn management_department_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return forbidden();
    }

    let Ok(Some(department)) = repositories::get_department(&pool, &id).await else {
        return (StatusCode::NOT_FOUND, Html("<h1>Department not found</h1>")).into_response();
    };

    let members = repositories::list_department_members(&pool, &department.name)
        .await
        .unwrap_or_default();
    let member_count = members.len() as i64;

    let top_tools = repositories::list_department_top_tools(&pool, &department.name, 10)
        .await
        .unwrap_or_default();

    let mut total_input_tokens = 0i64;
    let mut total_output_tokens = 0i64;
    let mut total_requests = 0i64;
    let mut total_cost_microdollars = 0i64;
    for m in &members {
        total_input_tokens += m.input_tokens;
        total_output_tokens += m.output_tokens;
        total_requests += m.requests;
        total_cost_microdollars += m.cost_microdollars;
    }

    let assignments_url = format!(
        "/admin/access/matrix?department={}",
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
        top_tools,
        total_input_tokens,
        total_output_tokens,
        total_requests,
        total_cost_microdollars,
    };

    render_typed_page(
        &engine,
        "management-department-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}
