//! What the roles page renders: the role cards, the two tables' rows, the
//! sortable headers and the filter menus.
//!
//! Split from [`super::data`] along the line between deciding which rows
//! survive and deciding what they look like. Everything here reads a
//! [`RolesQuery`] and writes a view type; nothing here filters.

use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::roles::entitlements::RoleEntitlementRow;
use crate::repositories::roles::members::RoleHolderRow;
use crate::types::Role;
use crate::types::access_control::AccessDecision;

use super::data::RolesQuery;
use super::view::{RoleCardView, RoleEntitlementView, RoleHolderView, RolePillView, SelectOption};

fn label_for(role: &str) -> String {
    role.parse::<Role>()
        .map_or_else(|()| role.to_owned(), |r| r.label().to_owned())
}

fn tone_for(role: &str) -> &'static str {
    match role {
        "platform_admin" => "err",
        "admin" => "warn",
        "developer" | "project_manager" => "accent",
        _ => "ok",
    }
}

pub(super) fn cards(
    rows: &[RoleHolderRow],
    entitlements: &[RoleEntitlementRow],
    query: &RolesQuery,
    known: &[String],
) -> Vec<RoleCardView> {
    known
        .iter()
        .map(|role| {
            let id = role.as_str();
            let member_count = rows
                .iter()
                .filter(|r| r.roles.iter().any(|h| h == id))
                .count();
            let manual_count = rows
                .iter()
                .filter(|r| r.manual_roles.iter().any(|h| h == id))
                .count();
            let entitlement_count = entitlements.iter().filter(|e| e.role == id).count();
            let active = query.field("role") == Some(id);
            RoleCardView {
                id: id.to_owned(),
                label: label_for(id),
                member_count,
                manual_count,
                directory_count: member_count - manual_count,
                entitlement_count,
                tone: tone_for(id),
                href: if active {
                    query.url_with(&[("role", "")])
                } else {
                    query.url_with(&[("role", id)])
                },
                active,
                note: format!("{entitlement_count} entitlements · {manual_count} by hand"),
            }
        })
        .collect()
}

fn initials(name: &str) -> String {
    name.split(|c: char| c.is_whitespace() || c == '-' || c == '.' || c == '@')
        .filter(|p| !p.is_empty())
        .take(2)
        .filter_map(|p| p.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

fn pill(row: &RoleHolderRow, role: &str, query: &RolesQuery) -> RolePillView {
    let is_manual = row.manual_roles.iter().any(|m| m == role);
    RolePillView {
        role: role.to_owned(),
        label: label_for(role),
        is_manual,
        tone: if is_manual { tone_for(role) } else { "muted" },
        active: query.field("role") == Some(role),
    }
}

pub(super) fn member_views(rows: &[RoleHolderRow], query: &RolesQuery) -> Vec<RoleHolderView> {
    rows.iter()
        .map(|row| {
            let id = row.user_id.as_str();
            let display_name = row.display_name.clone().unwrap_or_else(|| id.to_owned());
            let roles: Vec<RolePillView> = row.roles.iter().map(|r| pill(row, r, query)).collect();
            let manual_roles: Vec<RolePillView> =
                roles.iter().filter(|p| p.is_manual).cloned().collect();
            RoleHolderView {
                initials: initials(&display_name),
                display_name,
                email: row
                    .email
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                has_manual: !manual_roles.is_empty(),
                roles,
                manual_roles,
                is_active: row.is_active,
                status: if row.is_active { "Active" } else { "Inactive" },
                status_tone: if row.is_active { "ok" } else { "muted" },
                requests_30d: row.requests_30d,
                cost_30d_microdollars: row.cost_30d_microdollars,
                href: format!("/admin/users/{}", urlencoding::encode(id)),
                user_id: row.user_id.clone(),
            }
        })
        .collect()
}

pub(super) fn entitlement_views(rows: &[RoleEntitlementRow]) -> Vec<RoleEntitlementView> {
    rows.iter()
        .map(|row| RoleEntitlementView {
            role_label: label_for(&row.role),
            role: row.role.clone(),
            entity_type_label: row.entity_type.replace('_', " "),
            entity_type: row.entity_type.clone(),
            entity_id: row.entity_id.clone(),
            access: row.access.to_string(),
            access_tone: match row.access {
                AccessDecision::Allow => "ok",
                AccessDecision::Deny => "err",
            },
            default_included: row.default_included,
            default_label: if row.default_included {
                "Open by default"
            } else {
                "Closed by default"
            },
            href: format!(
                "/admin/access-control?entity_kind={}&subject_kind=role",
                row.entity_type
            ),
        })
        .collect()
}

pub(super) fn sort_headers(query: &RolesQuery) -> Vec<SortHeaderView> {
    let current = query.sort_key();
    let descending = query.descending();
    [
        (
            "person",
            "Person",
            "sp-col-person",
            "The account the role is held by.",
        ),
        (
            "roles",
            "Roles",
            "sp-col-roles",
            "Every role this person holds. A coloured role was granted by hand; a grey one comes from the directory.",
        ),
        (
            "requests",
            "Requests 30d",
            "sp-table__cell--num sp-col-num",
            "Gateway requests this account made in the last 30 days.",
        ),
        (
            "cost",
            "Cost 30d",
            "sp-table__cell--num sp-col-num",
            "What those requests cost.",
        ),
    ]
    .into_iter()
    .map(|(key, label, class, hint)| {
        let active = current == key;
        let next_dir = if active && !descending { "desc" } else { "" };
        SortHeaderView {
            label,
            class,
            hint,
            url: query.url_with(&[("sort", key), ("dir", next_dir)]),
            active,
            aria_sort: match (active, descending) {
                (true, false) => "ascending",
                (true, true) => "descending",
                (false, _) => "none",
            },
            indicator: match (active, descending) {
                (true, false) => "▲",
                (true, true) => "▼",
                (false, _) => "",
            },
        }
    })
    .collect()
}

pub(super) fn options(
    entries: &[(&str, &str)],
    all_label: &str,
    selected: Option<&str>,
) -> Vec<SelectOption> {
    let mut out = vec![SelectOption {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(entries.iter().map(|(value, label)| SelectOption {
        value: (*value).to_owned(),
        label: (*label).to_owned(),
        selected: selected == Some(*value),
    }));
    out
}

pub(super) fn role_options(query: &RolesQuery, known: &[String]) -> Vec<SelectOption> {
    let entries: Vec<(&str, &str)> = known.iter().map(|r| (r.as_str(), r.as_str())).collect();
    options(&entries, "All roles", query.field("role"))
}

pub(super) fn source_options(query: &RolesQuery) -> Vec<SelectOption> {
    options(
        &[
            ("manual", "Granted by hand"),
            ("directory", "From directory"),
        ],
        "Any grant source",
        query.field("source"),
    )
}

pub(super) fn status_options(query: &RolesQuery) -> Vec<SelectOption> {
    options(
        &[("active", "Active"), ("inactive", "Inactive")],
        "Any status",
        query.field("status"),
    )
}
