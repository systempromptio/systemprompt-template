//! One marketplace as the Versions landing lists it: its current version,
//! source, size, and the window's figures across every version it has had.

use serde::Serialize;
use systemprompt::identifiers::MarketplaceId;

use super::history::short;
use crate::handlers::ssr::analysis::tone::{completion_tone, score_display};
use crate::handlers::ssr::format::format_cost;
use crate::repositories::analysis::marketplace_versions::{
    MarketplaceCompletionRollup, MarketplaceListRow,
};

#[derive(Serialize)]
pub(super) struct MarketplaceCard {
    marketplace_id: MarketplaceId,
    name: String,
    href: String,
    source: Option<String>,
    source_short: Option<String>,
    source_hash: Option<String>,
    hash: Option<String>,
    hash_short: Option<String>,
    plugin_count: Option<i32>,
    skill_count: Option<i32>,
    versions: i64,
    first_seen_at: String,
    last_changed_at: String,
    invocations: i64,
    failed_invocations: i64,
    users: i64,
    requests: i64,
    failed: i64,
    cost: String,
    completion_display: String,
    completion_tone: &'static str,
    completion_judged: i64,
    history_display: String,
    history_judged: i64,
}

pub(super) fn card(
    row: MarketplaceListRow,
    scores: Option<&MarketplaceCompletionRollup>,
) -> MarketplaceCard {
    let current = scores.and_then(|s| s.current_completion);
    MarketplaceCard {
        completion_display: score_display(current),
        completion_tone: completion_tone(current),
        completion_judged: scores.map_or(0, |s| s.current_judged),
        history_display: score_display(scores.and_then(|s| s.history_completion)),
        history_judged: scores.map_or(0, |s| s.history_judged),
        href: format!(
            "/admin/analysis/versions/{}",
            urlencoding::encode(row.marketplace_id.as_str())
        ),
        name: row
            .name
            .clone()
            .unwrap_or_else(|| row.marketplace_id.as_str().to_owned()),
        marketplace_id: row.marketplace_id,
        source_short: row.source_hash.as_deref().map(short),
        source: row.source,
        source_hash: row.source_hash,
        hash_short: row.content_hash.as_deref().map(short),
        hash: row.content_hash,
        plugin_count: row.plugin_count,
        skill_count: row.skill_count,
        versions: row.versions,
        first_seen_at: row.first_seen_at.to_rfc3339(),
        last_changed_at: row.last_changed_at.to_rfc3339(),
        invocations: row.invocations,
        failed_invocations: row.failed_invocations,
        users: row.users,
        requests: row.requests,
        failed: row.failed,
        cost: if row.cost > 0 {
            format_cost(row.cost)
        } else {
            "—".to_owned()
        },
    }
}
