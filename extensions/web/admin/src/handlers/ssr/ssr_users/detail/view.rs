//! Shaping the loaded tab data into the detail page's context.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories::scope::defaults::ScopeDefaults;
use crate::repositories::users::enrolment::UserDeviceRow;
use crate::repositories::users::sessions::SigninSessionRow;

use crate::types::access_control::{AccessControlRule, AccessDecision};

use super::context::{
    AccessTabView, DeviceRowView, DevicesTabView, IdentityTabView, MembershipTabView,
    ScopeDefaultOptionView, UserAccessRowView, UserAccessSectionView, UserSessionRowView,
    UserSessionsTabView,
};
use super::load::{AccessData, IdentityData, MembershipData};

pub(super) use super::usage::usage_tab;

const SLACK_ISSUER: &str = "https://slack.com";

pub(super) fn identity_tab(
    detail: &crate::types::UserDetail,
    data: &IdentityData,
    role_choices: Vec<crate::handlers::ssr::types::RoleChoiceView>,
) -> IdentityTabView {
    let primary = data.identities.first();
    IdentityTabView {
        display_name: detail.display_name.clone().unwrap_or_default(),
        email: detail
            .email
            .as_ref()
            .map(|e| e.as_str().to_owned())
            .unwrap_or_default(),
        is_active: detail.is_active,
        created_at: stamp(Some(detail.created_at)),
        role_choices,
        has_adfs_groups: !data.adfs_groups.is_empty(),
        adfs_groups: data.adfs_groups.clone(),
        idp_issuer: primary.map(|i| i.issuer.clone()).unwrap_or_default(),
        external_sub: primary.map(|i| i.external_sub.clone()).unwrap_or_default(),
        linked_at: primary.map_or_else(String::new, |i| stamp(Some(i.linked_at))),
        slack_user_id: data
            .identities
            .iter()
            .find(|i| i.issuer == SLACK_ISSUER)
            .map(|i| i.external_sub.clone())
            .unwrap_or_default(),
        salesforce_username: data.salesforce_username.clone().unwrap_or_default(),
        share_token_version: data.share_token_version,
    }
}

pub(super) fn membership_tab(data: MembershipData) -> MembershipTabView {
    let defaults = data.defaults.unwrap_or_else(|| ScopeDefaults {
        primary_group_id: None,
        primary_project_id: None,
        source: "auto".to_owned(),
    });
    MembershipTabView {
        primary_group_options: scope_options(
            &data.groups,
            defaults.primary_group_id.as_deref(),
            "No primary group",
        ),
        primary_project_options: scope_options(
            &data.projects,
            defaults.primary_project_id.as_deref(),
            "No primary project",
        ),
        scope_source_is_manual: defaults.source == "manual",
        scope_source: defaults.source,
        group_choices: data.groups,
        project_choices: data.projects,
    }
}

// Why: only a container this person actually belongs to may be their primary
// one — attributing their spend to a group they are not in would put a cost on
// a page whose member list cannot explain it.
fn scope_options(
    choices: &[crate::handlers::ssr::types::MembershipChoiceView],
    selected: Option<&str>,
    none_label: &str,
) -> Vec<ScopeDefaultOptionView> {
    let mut out = vec![ScopeDefaultOptionView {
        value: String::new(),
        label: none_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(
        choices
            .iter()
            .filter(|choice| choice.held)
            .map(|choice| ScopeDefaultOptionView {
                selected: selected == Some(choice.id.as_str()),
                value: choice.id.clone(),
                label: choice.name.clone(),
            }),
    );
    out
}

pub(super) fn access_tab(data: AccessData, user_id: &UserId) -> AccessTabView {
    let Some(matrix) = data.matrix else {
        return AccessTabView {
            has_groups: false,
            sections: Vec::new(),
        };
    };
    let sections = matrix
        .sections
        .into_iter()
        .map(|section| {
            let rows: Vec<UserAccessRowView> = section
                .rows
                .into_iter()
                .map(|row| {
                    let own = own_rule(&data.rules, &section.entity_type, &row.entity_id, user_id);
                    UserAccessRowView {
                        entity_type: section.entity_type.clone(),
                        effective_tone: effective_tone(&row.effective),
                        entity_id: row.entity_id,
                        entity_name: row.entity_name,
                        effective: row.effective,
                        layer: row.source.layer,
                        detail: row.source.detail,
                        state: own.map_or("inherit", |r| match r.access {
                            AccessDecision::Allow => "allow",
                            AccessDecision::Deny => "deny",
                        }),
                        rule_id: own.map(|r| r.id.clone()).unwrap_or_default(),
                    }
                })
                .collect();
            UserAccessSectionView {
                has_rows: !rows.is_empty(),
                rows,
                entity_type: section.entity_type,
                label: section.label,
            }
        })
        .collect();
    AccessTabView {
        has_groups: !matrix.user.group_ids.is_empty(),
        sections,
    }
}

// Why: `warn` and `pending` are real resolver outcomes, not failures. A warn
// is a reach that enforcement is currently letting through; a pending is a
// hold awaiting approval. Both get a tone that says "look here", not red.
const fn effective_tone(effective: &str) -> &'static str {
    match effective.as_bytes() {
        b"allow" => "ok",
        b"deny" => "err",
        b"warn" => "warn",
        _ => "info",
    }
}

fn own_rule<'a>(
    rules: &'a [AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    user_id: &UserId,
) -> Option<&'a AccessControlRule> {
    rules.iter().find(|r| {
        r.entity_type == entity_type
            && r.entity_id == entity_id
            && r.rule_type.as_str() == "user"
            && r.rule_value == user_id.as_str()
    })
}

pub(super) fn devices_tab(rows: Vec<UserDeviceRow>) -> DevicesTabView {
    let active_count = rows.iter().filter(|r| r.revoked_at.is_none()).count();
    DevicesTabView {
        count: rows.len(),
        active_count,
        has_rows: !rows.is_empty(),
        rows: rows.into_iter().map(device_row).collect(),
    }
}

fn device_row(row: UserDeviceRow) -> DeviceRowView {
    let revoked = row.revoked_at.is_some();
    DeviceRowView {
        kind_label: match row.kind.as_str() {
            "bridge" => "Bridge",
            "cert" => "Certificate",
            _ => "Token",
        },
        label: row.label,
        detail: row.detail.unwrap_or_default(),
        created_at: stamp(row.created_at),
        last_seen: stamp(row.last_seen_at),
        status_label: if revoked { "Revoked" } else { "Active" },
        status_tone: if revoked { "muted" } else { "ok" },
        revocable: row.revocable && !revoked,
        kind: row.kind,
        id: row.id,
    }
}

pub(super) fn sessions_tab(
    rows: &[SigninSessionRow],
    user_id: &UserId,
    page: i64,
    page_size: i64,
) -> UserSessionsTabView {
    let live_count = rows.iter().filter(|r| r.revoked_at.is_none()).count();
    let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let offset = usize::try_from(page * page_size).unwrap_or(0);
    let take = usize::try_from(page_size).unwrap_or(50);
    let slice: Vec<UserSessionRowView> = rows
        .iter()
        .skip(offset)
        .take(take)
        .map(session_row)
        .collect();
    let shown = i64::try_from(slice.len()).unwrap_or(0);
    let window = PageWindow::new(page, page_size, total, shown, "sessions");
    UserSessionsTabView {
        has_rows: !slice.is_empty(),
        pagination: sessions_pagination(user_id, window),
        rows: slice,
        live_count,
    }
}

fn session_row(row: &SigninSessionRow) -> UserSessionRowView {
    let revoked = row.revoked_at.is_some();
    let expired = row.expires_at.is_some_and(|e| e < Utc::now());
    let id = row.session_id.as_str();
    UserSessionRowView {
        session_id: row.session_id.clone(),
        short_id: id.chars().take(12).collect(),
        detail_url: format!("/admin/sessions/{}", urlencoding::encode(id)),
        source: row.session_source.clone().unwrap_or_else(|| "—".to_owned()),
        ip: row.ip_address.clone().unwrap_or_default(),
        user_agent: row.user_agent.clone().unwrap_or_default(),
        requests: row.request_count,
        started_at: stamp(Some(row.started_at)),
        last_activity: stamp(Some(row.last_activity_at)),
        status_label: if revoked {
            "Revoked"
        } else if expired {
            "Expired"
        } else {
            "Live"
        },
        status_tone: if revoked {
            "muted"
        } else if expired {
            "warn"
        } else {
            "ok"
        },
        revocable: !revoked,
    }
}

fn sessions_pagination(user_id: &UserId, window: PageWindow) -> Pagination {
    let page = window.index;
    let prefix = format!(
        "/admin/users/{}?tab=sessions&",
        urlencoding::encode(user_id.as_str())
    );
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

// Why: one date format across every tab. An absent timestamp renders as an
// em dash, never as the epoch or as "now".
pub(super) fn stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(
        || "—".to_owned(),
        |t| t.format("%Y-%m-%d %H:%M").to_string(),
    )
}
