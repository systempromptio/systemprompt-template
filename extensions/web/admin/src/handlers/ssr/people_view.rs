//! Row shaping shared by the group and project pages.
//!
//! Groups and projects present the same four things — members, model mix,
//! top skills or tools, and the other container their people overlap with —
//! so the shaping lives here once. Only the group's tabbed Access editor and
//! the two listing pages' header tiles are page-specific.

use std::collections::{HashMap, HashSet};

use systemprompt::identifiers::UserId;

use super::format::short_num;
use super::types::{
    MappingRowView, MemberRowView, MemberSetChipView, ModelMixRowView, NameCountRowView,
    ProjectRowView, SourceBadgeView, StatTileView,
};
use crate::repositories::people_usage::breakdown::{LinkedScopeRow, ModelUsageRow};
use crate::repositories::people_usage::{MemberUsageRow, ScopeUsageRow};

pub(crate) const DIRECTORY_SOURCE: &str = "adfs";

// Why: the pages render a partial screen rather than an error page when one
// rollup fails, so every read here collapses to its empty shape and says so
// in the log.
pub(crate) fn or_default<T: Default>(what: &'static str, result: Result<T, sqlx::Error>) -> T {
    result
        .inspect_err(|e| tracing::warn!(error = %e, what, "people usage query failed"))
        .unwrap_or_default()
}

// Why: A membership row from either container — the two tables carry identical
// shapes, so the view borrows rather than owning a third copy of them.
pub(crate) struct MemberInput<'a> {
    pub user_id: &'a str,
    pub display_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub sources: &'a [String],
    pub source_ad_groups: &'a [String],
}

pub(crate) struct MemberContext<'a> {
    pub usage: &'a HashMap<String, MemberUsageRow>,
    pub active: &'a HashSet<String>,
    pub can_manage: bool,
}

#[must_use]
pub(crate) fn format_usd(microdollars: i64) -> String {
    format!("${:.2}", microdollars as f64 / 1_000_000.0)
}

#[must_use]
pub(crate) fn share(value: i64, total: i64) -> i64 {
    if total <= 0 {
        return 0;
    }
    (value * 100 / total).clamp(0, 100)
}

pub(crate) fn stat_tiles(usage: &ScopeUsageRow, member_count: i64) -> Vec<StatTileView> {
    vec![
        StatTileView {
            label: "Members",
            value: member_count.to_string(),
        },
        StatTileView {
            label: "Active 30d",
            value: usage.active_members.to_string(),
        },
        StatTileView {
            label: "Requests 30d",
            value: short_num(usage.requests),
        },
        // Why: token counts run to seven figures, and a tile is read at a
        // glance rather than compared digit by digit. `short_num` is the same
        // banding the `formatNumber` helper applies everywhere else, so the
        // tile and the table column beneath it agree.
        StatTileView {
            label: "Tokens in / out",
            value: format!(
                "{} / {}",
                short_num(usage.tokens_in),
                short_num(usage.tokens_out)
            ),
        },
        StatTileView {
            label: "Cost 30d",
            value: format_usd(usage.cost_microdollars),
        },
    ]
}

pub(crate) fn model_rows(models: &[ModelUsageRow]) -> Vec<ModelMixRowView> {
    let total: i64 = models.iter().map(|m| m.requests).sum();
    models
        .iter()
        .map(|m| ModelMixRowView {
            model: m.model.clone(),
            provider: m.provider.clone(),
            requests: m.requests,
            tokens_in: m.tokens_in,
            tokens_out: m.tokens_out,
            cost_microdollars: m.cost_microdollars,
            share_pct: share(m.requests, total),
        })
        .collect()
}

// Why: the share bar is relative to the top row, not to the total. These are
// truncated top-ten lists, so a percentage of their own sum would claim the
// eleventh entry does not exist.
pub(crate) fn name_count_rows(rows: &[(String, i64)]) -> Vec<NameCountRowView> {
    let top = rows.first().map_or(0, |r| r.1);
    rows.iter()
        .map(|(name, count)| NameCountRowView {
            name: name.clone(),
            count: *count,
            share_pct: share(*count, top),
        })
        .collect()
}

pub(crate) fn linked_rows(rows: &[LinkedScopeRow], prefix: &str) -> Vec<ProjectRowView> {
    rows.iter()
        .map(|r| ProjectRowView {
            href: format!("/admin/{prefix}/{}", r.id),
            id: r.id.clone(),
            name: r.name.clone(),
            description: r.description.clone(),
            member_count: r.member_count,
            group_count: 0,
            active_members_30d: 0,
            requests_30d: 0,
            cost_30d_microdollars: 0,
        })
        .collect()
}

pub(crate) fn chips(rows: &[LinkedScopeRow]) -> Vec<MemberSetChipView> {
    rows.iter()
        .map(|r| MemberSetChipView {
            id: r.id.clone(),
            label: r.name.clone(),
        })
        .collect()
}

// Why: Why: a directory-sourced row cannot be removed by hand — the next
// sign-in replaces the whole `adfs` set and would write it straight back, so a
// Remove button on it would look like it had failed.
pub(crate) fn member_rows(
    members: &[MemberInput<'_>],
    ctx: &MemberContext<'_>,
) -> Vec<MemberRowView> {
    members
        .iter()
        .map(|m| {
            let usage = ctx.usage.get(m.user_id);
            let manual = m.sources.iter().any(|s| s != DIRECTORY_SOURCE);
            MemberRowView {
                detail_href: format!("/admin/users/{}", urlencoding::encode(m.user_id)),
                display_name: m.display_name.or(m.email).unwrap_or(m.user_id).to_owned(),
                email: m.email.map(ToOwned::to_owned),
                is_active: ctx.active.contains(m.user_id),
                requests_30d: usage.map_or(0, |u| u.requests),
                tokens_30d: usage.map_or(0, |u| u.tokens),
                cost_30d_microdollars: usage.map_or(0, |u| u.cost_microdollars),
                last_active: usage.and_then(|u| u.last_active.map(|t| t.to_rfc3339())),
                sources: m.sources.iter().map(|s| source_badge(s)).collect(),
                source_ad_groups: m.source_ad_groups.to_vec(),
                can_remove: ctx.can_manage && manual,
                user_id: UserId::new(m.user_id.to_owned()),
            }
        })
        .collect()
}

fn source_badge(source: &str) -> SourceBadgeView {
    match source {
        DIRECTORY_SOURCE => SourceBadgeView {
            source: source.to_owned(),
            label: "Directory".to_owned(),
            color: "info",
            title: "Written by the directory at sign-in and replaced on every sign-in".to_owned(),
        },
        "manual" => SourceBadgeView {
            source: source.to_owned(),
            label: "Manual".to_owned(),
            color: "gray",
            title: "Added by an operator in the dashboard".to_owned(),
        },
        other => SourceBadgeView {
            source: other.to_owned(),
            label: "Derived".to_owned(),
            color: "muted",
            title: "Holds no membership row at all, so falls into Unassigned".to_owned(),
        },
    }
}

pub(crate) fn mapping_rows(
    rows: impl Iterator<Item = (String, String)>,
    can_map: bool,
) -> Vec<MappingRowView> {
    rows.map(|(ad_group, source)| MappingRowView {
        can_remove: can_map && source != "yaml",
        ad_group,
        source,
    })
    .collect()
}
