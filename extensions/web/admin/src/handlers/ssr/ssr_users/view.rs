//! View-model assembly for the user roster and detail pages.
//!
//! Pure transforms from repository rows into the serde structs the `users` /
//! `user-detail` templates render: marketplace resolution, per-user
//! enrichment, and department grouping. `UsersPageData` / `UserDetailPageData`
//! are fully self-contained typed structs; callers in `mod.rs` populate them
//! directly and hand them to `render_typed_page` with no post-hoc value
//! mutation.

use crate::repositories;

use super::super::types::{DepartmentGroup, EnrichedUserView, UserMarketplaceRef};

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

/// Resolve effective marketplaces for a user: every YAML-defined marketplace is
/// granted by default, then `access_control_rules` rows (matching the user's id
/// or department) are applied as allow/deny overrides.
pub(super) fn resolve_marketplaces(
    yaml_defaults: &[(String, String)],
    overrides: &[&repositories::UserMarketplaceOverride],
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

pub(super) fn enrich_users(
    users: &[crate::types::UserSummary],
    aggregates: &[repositories::UserManagementAggregate],
    runtime: &[repositories::UserRuntimeAggregate],
    overrides: &[repositories::UserMarketplaceOverride],
    yaml_marketplaces: &[(String, String)],
) -> Vec<EnrichedUserView> {
    let agg_map: std::collections::HashMap<&str, &repositories::UserManagementAggregate> =
        aggregates.iter().map(|a| (a.user_id.as_str(), a)).collect();
    let rt_map: std::collections::HashMap<&str, &repositories::UserRuntimeAggregate> =
        runtime.iter().map(|r| (r.user_id.as_str(), r)).collect();
    let mut ovr_map: std::collections::HashMap<&str, Vec<&repositories::UserMarketplaceOverride>> =
        std::collections::HashMap::new();
    for o in overrides {
        ovr_map.entry(o.user_id.as_str()).or_default().push(o);
    }

    users
        .iter()
        .map(|u| {
            let agg = agg_map.get(u.user_id.as_str());
            let rt = rt_map.get(u.user_id.as_str());
            let device_freshness =
                freshness_for(rt.and_then(|r| r.newest_device_seen_at)).to_owned();
            let user_overrides = ovr_map.get(u.user_id.as_str()).cloned().unwrap_or_default();
            let marketplaces = resolve_marketplaces(yaml_marketplaces, &user_overrides);
            EnrichedUserView {
                user_id: u.user_id.clone(),
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
                created_at: agg.map(|a| a.created_at.to_rfc3339()).unwrap_or_default(),
                marketplaces,
                assigned_skills_count: agg.map_or(0, |a| a.assigned_skills_count),
                devices_count: agg.map_or(0, |a| a.devices_count),
                connected_agents: rt.map_or(0, |r| r.connected_agents),
                total_agents: rt.map_or(0, |r| r.total_agents),
                lifetime_tokens: rt.map_or(0, |r| r.lifetime_tokens),
                device_freshness,
            }
        })
        .collect()
}

/// Group enriched users by department. "Default" first, then alphabetical;
/// users with no department go last as "Unassigned".
pub(super) fn group_by_department(users: Vec<EnrichedUserView>) -> Vec<DepartmentGroup> {
    let mut buckets: std::collections::BTreeMap<String, Vec<EnrichedUserView>> =
        std::collections::BTreeMap::new();
    for u in users {
        let key = if u.department.is_empty() {
            "Unassigned".to_owned()
        } else {
            u.department.clone()
        };
        buckets.entry(key).or_default().push(u);
    }

    let mut groups: Vec<DepartmentGroup> = buckets
        .into_iter()
        .map(|(department, mut users)| {
            users.sort_by(|a, b| {
                let an = a.display_name.as_deref().unwrap_or(a.user_id.as_str());
                let bn = b.display_name.as_deref().unwrap_or(b.user_id.as_str());
                an.to_lowercase().cmp(&bn.to_lowercase())
            });
            let total_tokens = users.iter().map(|u| u.lifetime_tokens).sum();
            let total_sessions = users.iter().map(|u| u.sessions).sum();
            DepartmentGroup {
                user_count: users.len(),
                total_tokens,
                total_sessions,
                department,
                users,
            }
        })
        .collect();

    groups.sort_by(|a, b| {
        fn rank(name: &str) -> u8 {
            match name {
                "Default" => 0,
                "Unassigned" => 2,
                _ => 1,
            }
        }
        rank(&a.department).cmp(&rank(&b.department)).then_with(|| {
            a.department
                .to_lowercase()
                .cmp(&b.department.to_lowercase())
        })
    });
    groups
}
