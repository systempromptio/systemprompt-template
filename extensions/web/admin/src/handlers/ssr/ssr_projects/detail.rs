//! `/admin/projects/{id}`: the three tabs and everything each one reads.
//!
//! The header tiles are exclusive attribution, so a project's numbers are its
//! own slice of the instance. The members table is the one member-attributed
//! view on the page — a person on two projects appears under both — and it
//! says so above the table rather than leaving the reader to reconcile it.

use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::repositories::people_usage::breakdown::{
    LinkedScopeRow, ModelUsageRow, list_linked_scopes, list_scope_top_models,
};
use crate::repositories::people_usage::{
    DEFAULT_WINDOW_DAYS, DailyRequests, LEADERBOARD_LIMIT, MemberUsageRow, ScopeUsageRow,
    get_scope_usage, list_daily_requests, list_member_usage,
};
use crate::repositories::projects::activity::{
    ProjectCommitRow, ProjectSessionRow, SkillEffectivenessRow, ToolHealthRow,
    list_project_commits, list_project_sessions, list_project_skill_effectiveness,
    list_project_tool_health,
};
use crate::repositories::scope::{Attribution, ScopeKind, ScopeQuery};
use crate::types::UserContext;
use crate::types::projects::{ProjectMemberRow, ProjectRow};

use super::super::people_view::{
    MemberContext, MemberInput, chips, mapping_rows, member_rows, or_default,
};
use super::super::types::{
    ProjectDetailPageData, ProjectMembersTabView, ProjectSettingsTabView, UserOptionView,
};
use super::WINDOW_LABEL;
use super::detail_view::{gated_rows, kpis, tabs, usage_tab};

// Why: how many rows each detail-page section carries. Deeper questions belong
// on the pages that own them — this page's job is the shape, not the archive.
const SECTION_LIMIT: i64 = 25;

// Why: everything the three tabs read, loaded once so a tab switch is a render.
pub(super) struct DetailData {
    pub(super) usage: ScopeUsageRow,
    pub(super) members: Vec<ProjectMemberRow>,
    pub(super) member_usage: HashMap<String, MemberUsageRow>,
    pub(super) active: HashSet<String>,
    pub(super) addable: Vec<UserOptionView>,
    pub(super) groups: Vec<LinkedScopeRow>,
    pub(super) tool_calls: i64,
    pub(super) tool_success: i64,
    pub(super) skills: Vec<SkillEffectivenessRow>,
}

// Why: the Usage tab's reads, taken only when that tab is the one being drawn.
pub(super) struct ProjectUsageData {
    pub(super) daily: Vec<DailyRequests>,
    pub(super) models: Vec<ModelUsageRow>,
    pub(super) tools: Vec<ToolHealthRow>,
    pub(super) sessions: Vec<ProjectSessionRow>,
    pub(super) commits: Vec<ProjectCommitRow>,
}

const fn exclusive(project_id: &str) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeKind::Project,
        Attribution::Exclusive,
        project_id,
        DEFAULT_WINDOW_DAYS,
    )
}

const fn member_view(project_id: &str) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeKind::Project,
        Attribution::Member,
        project_id,
        DEFAULT_WINDOW_DAYS,
    )
}

pub(super) async fn load(pool: &PgPool, project_id: &str, user_ctx: &UserContext) -> DetailData {
    let q = exclusive(project_id);
    let members_q = member_view(project_id);
    let members = repositories::projects::members::list_project_members(pool, project_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "project members failed"))
        .unwrap_or_default();

    let (usage, member_usage, groups, tools, skills) = tokio::join!(
        get_scope_usage(pool, &q),
        list_member_usage(pool, &members_q),
        list_linked_scopes(pool, &members_q),
        list_project_tool_health(pool, &q, SECTION_LIMIT),
        list_project_skill_effectiveness(pool, &q, LEADERBOARD_LIMIT),
    );
    let tools = or_default("project tool health", tools);

    DetailData {
        usage: or_default("project usage", usage),
        member_usage: or_default("project member usage", member_usage),
        groups: or_default("project groups", groups),
        skills: or_default("project skills", skills),
        tool_calls: tools.iter().map(|t| t.calls).sum(),
        tool_success: tools.iter().map(|t| t.calls - t.failures).sum(),
        active: load_active_user_ids(pool).await,
        addable: if user_ctx.is_admin {
            load_addable_users(pool, &members).await
        } else {
            Vec::new()
        },
        members,
    }
}

pub(super) async fn load_usage(pool: &PgPool, project_id: &str) -> ProjectUsageData {
    let q = exclusive(project_id);
    let (daily, models, tools, sessions, commits) = tokio::join!(
        list_daily_requests(pool, &q),
        list_scope_top_models(pool, &q, LEADERBOARD_LIMIT),
        list_project_tool_health(pool, &q, SECTION_LIMIT),
        list_project_sessions(pool, &q, SECTION_LIMIT),
        list_project_commits(pool, &q, SECTION_LIMIT),
    );
    ProjectUsageData {
        daily: or_default("project daily requests", daily),
        models: or_default("project model mix", models),
        tools: or_default("project tool health", tools),
        sessions: or_default("project sessions", sessions),
        commits: or_default("project commits", commits),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub(super) fn page_data(
    project: &ProjectRow,
    data: &DetailData,
    usage: Option<&ProjectUsageData>,
    tab: &str,
    user_ctx: &UserContext,
    mappings: Vec<crate::types::projects::ProjectAdMappingRow>,
    rules: &[crate::types::access_control::AccessControlRule],
) -> ProjectDetailPageData {
    let inputs: Vec<MemberInput<'_>> = data.members.iter().map(as_member_input).collect();
    let rows = member_rows(
        &inputs,
        &MemberContext {
            usage: &data.member_usage,
            active: &data.active,
            can_manage: user_ctx.is_admin,
        },
    );

    ProjectDetailPageData {
        page: "project-detail",
        title: project.name.clone(),
        breadcrumbs: super::breadcrumbs(&project.name),
        tabs: tabs(&project.id, tab),
        active_tab: tab.to_owned(),
        window_label: WINDOW_LABEL.to_owned(),
        kpis: kpis(data, rows.len() as i64),
        group_count: data.groups.len() as i64,
        groups_represented: chips(&data.groups),
        members: (tab == "members").then(|| ProjectMembersTabView {
            count: rows.len() as i64,
            rows,
            addable_users: data.addable.clone(),
        }),
        usage: usage.map(|u| usage_tab(u, &data.skills)),
        settings: (tab == "settings").then(|| ProjectSettingsTabView {
            name: project.name.clone(),
            description: project.description.clone().unwrap_or_default(),
            source: project.source.clone(),
            mapping_count: mappings.len() as i64,
            feeding_groups: mapping_rows(
                mappings.into_iter().map(|m| (m.ad_group, m.source)),
                user_ctx.is_platform_admin,
            ),
            gated_count: rules.len() as i64,
            gated_entities: gated_rows(rules),
            can_map: crate::types::roles_grant_platform(&user_ctx.roles),
            can_delete: user_ctx.is_admin,
        }),
        project_id: project.id.clone(),
        project_name: project.name.clone(),
        description: project.description.clone(),
        can_manage: user_ctx.is_admin,
    }
}

fn as_member_input(row: &ProjectMemberRow) -> MemberInput<'_> {
    MemberInput {
        user_id: row.user_id.as_str(),
        display_name: row.display_name.as_deref(),
        email: row.email.as_deref(),
        sources: &row.sources,
        source_ad_groups: &row.source_ad_groups,
    }
}

async fn load_active_user_ids(pool: &PgPool) -> HashSet<String> {
    // Why: `status = 'active'` is how the rest of the crate reads an enabled
    // account, and the macro form means a column rename breaks the build
    // rather than quietly reporting every member as disabled.
    sqlx::query_scalar!(r#"SELECT id AS "id!" FROM users WHERE status = 'active'"#)
        .fetch_all(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "active user ids failed"))
        .unwrap_or_default()
        .into_iter()
        .collect()
}

async fn load_addable_users(pool: &PgPool, members: &[ProjectMemberRow]) -> Vec<UserOptionView> {
    let existing: HashSet<&str> = members.iter().map(|m| m.user_id.as_str()).collect();
    sqlx::query_as::<_, (String, Option<String>, String)>(
        "SELECT id, COALESCE(display_name, full_name, name), email
         FROM users
         WHERE NOT ('anonymous' = ANY(roles))
         ORDER BY COALESCE(display_name, full_name, name, id)",
    )
    .fetch_all(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "addable user list failed"))
    .unwrap_or_default()
    .into_iter()
    .filter(|(id, _, _)| !existing.contains(id.as_str()))
    .map(|(id, name, email)| UserOptionView {
        label: name.map_or_else(|| email.clone(), |n| format!("{n} ({email})")),
        user_id: UserId::new(id),
    })
    .collect()
}
