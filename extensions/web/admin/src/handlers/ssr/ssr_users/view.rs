//! Shaping the loaded data into the detail page's tab views.
//!
//! Pure transforms: every function takes what `data.rs` and the repositories
//! returned and writes the view type the `user-detail` template reads.

use chrono::{DateTime, Utc};

use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::{format_token_total, short_id};
use crate::handlers::ssr::list_view::SelectOptionView;
use crate::handlers::ssr::types::{UserRuntimeView, UserTokenView};
use crate::repositories::governance::effective::EffectivePermissions;
use crate::types::UserDetail;

use super::context::{
    AccessTabView, ActivityTabView, CategoryRowView, DetailKpiView, EventRowView, IdentityTabView,
    RoleChoiceView, SessionRowView, ToolRowView, UserHeaderView,
};
use super::data::DetailExtras;

// Why: one date format across every tab. An absent timestamp renders as an
// em dash, never as the epoch or as "now".
pub(super) fn stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(
        || "—".to_owned(),
        |t| t.format("%Y-%m-%d %H:%M").to_string(),
    )
}

pub(super) fn display_name(detail: &UserDetail) -> String {
    detail
        .display_name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| detail.user_id.as_str().to_owned())
}

pub(super) fn header(detail: &UserDetail) -> UserHeaderView {
    let encoded = urlencoding::encode(detail.user_id.as_str()).into_owned();
    UserHeaderView {
        name: display_name(detail),
        email: detail
            .email
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        created_at: stamp(Some(detail.created_at)),
        last_active: stamp(Some(detail.last_active)),
        status_label: if detail.is_active {
            "Active"
        } else {
            "Inactive"
        },
        status_tone: if detail.is_active { "ok" } else { "muted" },
        permissions_url: format!("/admin/access-control?user_id={encoded}"),
        requests_url: format!("/admin/requests?user_id={encoded}"),
        user_id: detail.user_id.clone(),
    }
}

pub(super) fn kpis(
    detail: &UserDetail,
    runtime: &UserRuntimeView,
    tokens_count: i64,
) -> DetailKpiView {
    DetailKpiView {
        requests_display: format_token_total(runtime.requests),
        tokens_in_display: format_token_total(runtime.tokens_in),
        tokens_out_display: format_token_total(runtime.tokens_out),
        last_request: runtime
            .last_request_at
            .clone()
            .unwrap_or_else(|| "Never".to_owned()),
        events_display: format_token_total(detail.total_events),
        tokens_count,
    }
}

// Why: every role the instance knows plus every role this account already
// holds, so an unusual grant is a checked box rather than a silently dropped
// one when the form is saved.
fn role_choices(known: &[String], held: &[String]) -> Vec<RoleChoiceView> {
    let mut ids: Vec<String> = vec!["user".to_owned(), "admin".to_owned()];
    for role in known.iter().chain(held.iter()) {
        if !ids.contains(role) {
            ids.push(role.clone());
        }
    }
    ids.into_iter()
        .map(|id| RoleChoiceView {
            held: held.contains(&id),
            id,
        })
        .collect()
}

pub(super) fn identity_tab(
    detail: &UserDetail,
    extras: DetailExtras,
    departments: &[String],
    known_roles: &[String],
) -> IdentityTabView {
    let encoded = urlencoding::encode(detail.user_id.as_str()).into_owned();
    IdentityTabView {
        display_name: detail.display_name.clone().unwrap_or_default(),
        email: detail
            .email
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        is_active: detail.is_active,
        department_options: departments
            .iter()
            .map(|name| SelectOptionView {
                value: name.clone(),
                label: name.clone(),
                selected: *name == extras.department,
            })
            .collect(),
        department: extras.department,
        role_choices: role_choices(known_roles, &extras.roles),
        has_marketplaces: !extras.assignments.marketplaces.is_empty(),
        marketplaces: extras.assignments.marketplaces.clone(),
        assignments: extras.assignments,
        matrix_url: format!("/admin/access-control?user_id={encoded}"),
    }
}

pub(super) const fn access_tab(
    effective: EffectivePermissions,
    tokens: Vec<UserTokenView>,
) -> AccessTabView {
    AccessTabView {
        has_gateway_routes: !effective.gateway_routes.is_empty(),
        has_mcp_servers: !effective.mcp_servers.is_empty(),
        effective,
        has_tokens: !tokens.is_empty(),
        tokens,
        tokens_url: "/admin/access-tokens",
    }
}

pub(super) fn activity_tab(detail: &UserDetail, runtime: UserRuntimeView) -> ActivityTabView {
    let categories: Vec<CategoryRowView> = detail
        .activity_summary
        .iter()
        .map(|c| CategoryRowView {
            category: c.category.clone(),
            count: c.count,
        })
        .collect();
    let tools: Vec<ToolRowView> = detail
        .top_tools
        .iter()
        .map(|t| ToolRowView {
            tool_name: t.tool_name.clone(),
            count: t.count,
        })
        .collect();
    let sessions: Vec<SessionRowView> = detail
        .sessions
        .iter()
        .map(|s| SessionRowView {
            session_id: s.session_id.clone(),
            short_id: short_id(s.session_id.as_str()),
            detail_url: session_detail_url(&s.session_id),
            started_at: stamp(s.started_at),
            total_events: s.total_events,
            tool_uses: s.tool_uses,
            prompts: s.prompts,
            errors: s.errors,
            error_tone: if s.errors > 0 { "err" } else { "muted" },
        })
        .collect();
    let events: Vec<EventRowView> = detail
        .recent_activity
        .iter()
        .map(|e| EventRowView {
            category: e.category.to_string(),
            description: e.description.clone(),
            created_at: stamp(Some(e.created_at)),
            created_at_title: e.created_at.to_rfc3339(),
        })
        .collect();
    ActivityTabView {
        runtime,
        has_categories: !categories.is_empty(),
        categories,
        has_tools: !tools.is_empty(),
        tools,
        has_sessions: !sessions.is_empty(),
        sessions,
        has_events: !events.is_empty(),
        events,
    }
}
