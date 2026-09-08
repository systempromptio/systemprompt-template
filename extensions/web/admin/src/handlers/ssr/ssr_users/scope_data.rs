//! The group, project and role choices the detail page's Membership and
//! Identity tabs edit.
//!
//! The detail page's membership checkboxes carry the *source* of each
//! membership — a directory-owned one is re-projected at every sign-in, so
//! offering to uncheck it would be a control that silently undoes itself.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;

use super::super::types::{MembershipChoiceView, RoleChoiceView};

fn membership_source(sources: &[String]) -> &'static str {
    if sources.iter().any(|s| s == "adfs") {
        "adfs"
    } else if sources.iter().any(|s| s == "manual") {
        "manual"
    } else {
        "derived"
    }
}

// Why: Every group as a checkbox, with the source of the membership the user
// already holds. A directory-sourced membership is reported so the template
// can render it read-only.
pub(super) async fn group_choices(pool: &PgPool, user_id: &UserId) -> Vec<MembershipChoiceView> {
    let held = repositories::groups::members::list_group_ids_for_user(pool, user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: group ids unavailable"))
        .unwrap_or_default();
    let mut out = Vec::new();
    for group in repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: group listing failed"))
        .unwrap_or_default()
    {
        let is_held = held.iter().any(|id| id == &group.id);
        let source = if is_held {
            let members = repositories::groups::members::list_group_members(pool, &group.id)
                .await
                .unwrap_or_default();
            members
                .iter()
                .find(|m| m.user_id == user_id.as_str())
                .map_or("derived", |m| membership_source(&m.sources))
        } else {
            "manual"
        };
        out.push(MembershipChoiceView {
            id: group.id,
            name: group.name,
            held: is_held,
            source,
            from_directory: is_held && source == "adfs",
        });
    }
    out
}

pub(super) async fn project_choices(pool: &PgPool, user_id: &UserId) -> Vec<MembershipChoiceView> {
    let held = repositories::projects::members::list_project_ids_for_user(pool, user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: project ids unavailable"))
        .unwrap_or_default();
    repositories::projects::crud::list_projects(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: project listing failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|project| {
            let is_held = held.iter().any(|id| id == &project.id);
            MembershipChoiceView {
                id: project.id,
                name: project.name,
                held: is_held,
                source: "manual",
                from_directory: false,
            }
        })
        .collect()
}

// Why: The six flat roles as checkboxes. `platform_admin` is marked rather
// than filtered here so the template can decide whether to render it at all —
// only a platform admin may see or grant it.
pub(super) fn role_choices(held: &[String]) -> Vec<RoleChoiceView> {
    let mut ids: Vec<String> = crate::types::Role::ALL
        .iter()
        .map(|r| r.as_str().to_owned())
        .chain(held.iter().cloned())
        .collect();
    ids.sort();
    ids.dedup();
    ids.into_iter()
        .map(|id| {
            let role = id.parse::<crate::types::Role>().ok();
            RoleChoiceView {
                label: role.map_or_else(|| id.clone(), |r| r.label().to_owned()),
                held: held.contains(&id),
                platform_only: id == "platform_admin",
                id,
            }
        })
        .collect()
}
