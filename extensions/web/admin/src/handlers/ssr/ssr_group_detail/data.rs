//! Per-tab loaders for the group detail page.
//!
//! Only the active tab's data is fetched. The counts on the tab labels are the
//! exception — they are three cheap reads that must be right on every tab, or
//! the strip would advertise an empty tab as full.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::services::marketplaces::load_marketplaces;
use crate::types::groups::{GroupAdMappingRow, GroupMemberRow};

use crate::repositories::people_usage::breakdown::{
    LinkedScopeRow, ModelUsageRow, list_linked_scopes, list_scope_top_models,
    list_scope_top_skills, list_scope_top_tools,
};
use crate::repositories::people_usage::{
    DEFAULT_WINDOW_DAYS, LEADERBOARD_LIMIT, MemberUsageRow, ScopeUsageRow, get_scope_usage,
    list_daily_requests, list_member_usage,
};
use crate::repositories::scope::{Attribution, ScopeKind, ScopeQuery};

use super::super::people_view::or_default;
use super::super::types::{AccessSectionView, UserOptionView};
use super::tabs::TabCounts;

pub(super) struct MembersData {
    pub rows: Vec<GroupMemberRow>,
    pub usage: std::collections::HashMap<String, MemberUsageRow>,
    pub active: std::collections::HashSet<String>,
    pub addable: Vec<UserOptionView>,
}

pub(super) struct UsageTabData {
    pub models: Vec<ModelUsageRow>,
    pub daily: Vec<i64>,
    pub skills: Vec<(String, i64)>,
    pub tools: Vec<(String, i64)>,
    pub leaderboard: Vec<MemberUsageRow>,
}

// Why: One marketplace and whether this group is entitled to it.
pub(super) struct MarketplaceOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assigned: bool,
}

pub(super) async fn load_counts(pool: &PgPool, group_id: &str) -> TabCounts {
    let q = member_query(group_id);
    let (members, projects, mappings, marketplaces) = tokio::join!(
        repositories::groups::members::list_group_members(pool, group_id),
        list_linked_scopes(pool, &q),
        repositories::groups::mappings::list_group_ad_mappings(pool, group_id),
        repositories::groups::marketplaces::list_group_marketplace_ids(pool, group_id),
    );
    TabCounts {
        members: members.map(|m| m.len() as i64).unwrap_or_default(),
        marketplaces: marketplaces.map(|m| m.len() as i64).unwrap_or_default(),
        projects: projects.map(|p| p.len() as i64).unwrap_or_default(),
        mappings: mappings.map(|m| m.len() as i64).unwrap_or_default(),
    }
}

// Why: "who is in this group" is full membership — a person in two groups is
// in both — so every listing and breakdown binds this. It deliberately
// overlaps with the other groups, and the screens that use it say so.
const fn member_query(group_id: &str) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeKind::Group,
        Attribution::Member,
        group_id,
        DEFAULT_WINDOW_DAYS,
    )
}

// Why: every headline total is exclusive, so this group's figures are its
// slice of an instance the group totals partition. Reading a member-attributed
// total beside the other groups' would double-count anyone in two of them and
// leave a KPI row that adds up to more than the estate spent.
const fn exclusive_query(group_id: &str) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeKind::Group,
        Attribution::Exclusive,
        group_id,
        DEFAULT_WINDOW_DAYS,
    )
}

pub(super) async fn load_usage(pool: &PgPool, group_id: &str) -> ScopeUsageRow {
    let q = exclusive_query(group_id);
    or_default("group usage", get_scope_usage(pool, &q).await)
}

// Why: Which marketplaces this group is entitled to, against the whole catalog.
//
// The unassigned rows matter as much as the assigned ones: the tab is an
// editor, and an editor that lists only what is already granted cannot grant
// anything.
pub(super) async fn load_marketplace_options(
    pool: &PgPool,
    group_id: &str,
) -> Vec<MarketplaceOption> {
    let assigned: Vec<String> =
        repositories::groups::marketplaces::list_group_marketplace_ids(pool, group_id)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "group marketplace ids failed"))
            .unwrap_or_default();

    load_marketplaces()
        .into_iter()
        .map(|m| MarketplaceOption {
            assigned: assigned.iter().any(|id| id == m.id.as_str()),
            id: m.id.to_string(),
            name: m.name,
            description: m.description,
        })
        .collect()
}

pub(super) async fn load_usage_tab(pool: &PgPool, group_id: &str) -> UsageTabData {
    let q = member_query(group_id);
    let (models, daily, skills, tools, members) = tokio::join!(
        list_scope_top_models(pool, &q, LEADERBOARD_LIMIT),
        list_daily_requests(pool, &q),
        list_scope_top_skills(pool, &q, LEADERBOARD_LIMIT),
        list_scope_top_tools(pool, &q, LEADERBOARD_LIMIT),
        list_member_usage(pool, &q),
    );
    let mut leaderboard: Vec<MemberUsageRow> = or_default("group member leaderboard", members)
        .into_values()
        .collect();
    leaderboard.sort_by(|a, b| {
        b.cost_microdollars
            .cmp(&a.cost_microdollars)
            .then_with(|| b.requests.cmp(&a.requests))
    });
    leaderboard.truncate(LEADERBOARD_LIMIT as usize);
    let models = or_default("group model mix", models);
    let daily = or_default("group daily requests", daily)
        .into_iter()
        .map(|d| d.requests)
        .collect();
    let skills = or_default("group top skills", skills)
        .into_iter()
        .map(|s| (s.skill, s.invocations))
        .collect();
    let tools = or_default("group top tools", tools)
        .into_iter()
        .map(|t| {
            (
                format!("{} · {}", t.server_name, t.tool_name),
                t.invocations,
            )
        })
        .collect();
    UsageTabData {
        models,
        daily,
        skills,
        tools,
        leaderboard,
    }
}

pub(super) async fn load_members(pool: &PgPool, group_id: &str, can_manage: bool) -> MembersData {
    let rows = repositories::groups::members::list_group_members(pool, group_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "group members failed"))
        .unwrap_or_default();
    let q = member_query(group_id);
    let usage = or_default("group member usage", list_member_usage(pool, &q).await);
    let active = load_enabled_user_ids(pool).await;
    let addable = if can_manage {
        load_addable_users(pool, &rows).await
    } else {
        Vec::new()
    };
    MembersData {
        rows,
        usage,
        active,
        addable,
    }
}

// Why: the macro form, so a column this query names but the schema lacks is
// a build error. A runtime `query_scalar` would have its error swallowed by
// `unwrap_or_default`, the set would come back empty, and every member on the
// tab would render as disabled.
async fn load_enabled_user_ids(pool: &PgPool) -> std::collections::HashSet<String> {
    sqlx::query_scalar!("SELECT id FROM users WHERE status = 'active'")
        .fetch_all(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "enabled user ids failed"))
        .unwrap_or_default()
        .into_iter()
        .collect()
}

// Why: Candidates for the Add member dialog: every account not already in the
// group. Rendered server-side as `<option>` rows so the dialog needs no
// search endpoint of its own.
async fn load_addable_users(pool: &PgPool, members: &[GroupMemberRow]) -> Vec<UserOptionView> {
    let existing: std::collections::HashSet<&str> =
        members.iter().map(|m| m.user_id.as_str()).collect();
    // Why: the macro form for the same reason as the enabled-account set
    // above — this is a raw query over four columns of `users`, and a rename
    // in any of them would empty the dialog rather than fail the build.
    sqlx::query!(
        r#"SELECT id AS "id!",
                  COALESCE(display_name, full_name, name) AS "name?",
                  email AS "email!"
           FROM users
           WHERE NOT ('anonymous' = ANY(roles))
           ORDER BY COALESCE(display_name, full_name, name, id)"#
    )
    .fetch_all(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "addable user list failed"))
    .unwrap_or_default()
    .into_iter()
    .filter(|row| !existing.contains(row.id.as_str()))
    .map(|row| UserOptionView {
        label: row
            .name
            .map_or_else(|| row.email.clone(), |n| format!("{n} ({})", row.email)),
        user_id: UserId::new(row.id),
    })
    .collect()
}

pub(super) async fn load_projects(pool: &PgPool, group_id: &str) -> Vec<LinkedScopeRow> {
    or_default(
        "group projects",
        list_linked_scopes(pool, &member_query(group_id)).await,
    )
}

pub(super) async fn load_mappings(pool: &PgPool, group_id: &str) -> Vec<GroupAdMappingRow> {
    repositories::groups::mappings::list_group_ad_mappings(pool, group_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "group AD mappings failed"))
        .unwrap_or_default()
}

// Why: The Access tab: every catalog entity resolved for a subject holding only
// this group's membership, plus the group's own rules so the toggles can
// show `inherit` where no rule of its own exists.
pub(super) async fn load_access(pool: &PgPool, group_id: &str) -> Vec<AccessSectionView> {
    let Ok(services_path) = crate::handlers::shared::get_services_path() else {
        tracing::warn!("services path unavailable; group access matrix skipped");
        return Vec::new();
    };
    let mut sections = crate::handlers::access_control::build_matrix_sections(&services_path);
    if !sections.iter().any(|(kind, _, _)| kind == "marketplace") {
        let rows: Vec<(String, String, Option<String>)> = load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name, Some(m.description)))
            .collect();
        sections.insert(
            0,
            ("marketplace".to_owned(), "Marketplaces".to_owned(), rows),
        );
    }

    let subject = repositories::users::access_control::group_subject(group_id);
    let resolved =
        repositories::users::access_control::resolve_subject_matrix(pool, &subject, sections)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "group access matrix failed"))
            .unwrap_or_default();

    let own_rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access rules failed"))
        .unwrap_or_default();

    super::view::access_sections(resolved, &own_rules, group_id)
}
