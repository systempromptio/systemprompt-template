//! SSR page for a user's connectors — a health strip, then the cards grouped
//! by what the person should do next. The first paint is server-rendered from
//! the same snapshot the browser then polls; `connectors.js` re-renders the
//! groups in place and keeps each card's static "about" block.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::ssr_connectors_cards::{ATTENTION, ConnectorCardView, card};
use crate::error::AdminHtmlResult;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::services::connector_accounts::{Connection, get_connections};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[derive(Debug, Default, Serialize)]
struct ConnectorsSummary {
    configured: usize,
    connected: usize,
    needs_attention: usize,
    not_connected: usize,
    connected_pct: usize,
    attention_pct: usize,
    headline: String,
    next_step: Option<String>,
}

#[derive(Debug, Serialize)]
struct GroupView {
    key: &'static str,
    title: &'static str,
    note: &'static str,
    count: usize,
    cards: Vec<ConnectorCardView>,
}

#[derive(Debug, Serialize)]
struct ConnectorsPageView {
    page: &'static str,
    title: &'static str,
    user_email: String,
    user_id: UserId,
    summary: ConnectorsSummary,
    groups: Vec<GroupView>,
    has_connectors: bool,
    breadcrumbs: Vec<BreadcrumbView>,
}

const GROUPS: [(&str, &str, &str); 4] = [
    (
        "attention",
        "Needs your attention",
        "Broken or unverified — fix these first.",
    ),
    (
        "ready",
        "Ready to connect",
        "Authorize once; every client you connect uses the same account.",
    ),
    (
        "connected",
        "Connected",
        "Verified and available to every client.",
    ),
    (
        "quiet",
        "Nothing to do",
        "Built in, or not open to your account.",
    ),
];

// Why: the order a person should act in — broken first, then never
// connected, then healthy, then the ones nothing can be done about.
fn group_key(c: &Connection) -> &'static str {
    match c.status.as_str() {
        s if ATTENTION.contains(&s) && c.entitled => "attention",
        "not_connected" if c.entitled => "ready",
        "connected" => "connected",
        _ => "quiet",
    }
}

fn percent(part: usize, whole: usize) -> usize {
    (part * 100).checked_div(whole).unwrap_or(0)
}

fn summarise(connections: &[Connection]) -> ConnectorsSummary {
    let mut summary = ConnectorsSummary::default();
    let mut first_broken = None;
    let mut first_ready = None;
    for c in connections
        .iter()
        .filter(|c| c.configured && c.requires_auth)
    {
        summary.configured += 1;
        match c.status.as_str() {
            "connected" => summary.connected += 1,
            s if ATTENTION.contains(&s) => {
                summary.needs_attention += 1;
                first_broken.get_or_insert_with(|| c.display_name.clone());
            },
            "not_connected" => {
                summary.not_connected += 1;
                if c.entitled {
                    first_ready.get_or_insert_with(|| c.display_name.clone());
                }
            },
            _ => {},
        }
    }
    summary.connected_pct = percent(summary.connected, summary.configured);
    summary.attention_pct = percent(summary.needs_attention, summary.configured);
    summary.headline = match (summary.configured, summary.connected) {
        (0, _) => "No connectors on this gateway".to_owned(),
        (n, c) if n == c => "Everything is connected".to_owned(),
        (n, c) => format!("{c} of {n} connected"),
    };
    summary.next_step = match (first_broken, first_ready) {
        (Some(name), _) => Some(format!("Reconnect {name} to get it working again.")),
        (None, Some(name)) => Some(format!("Connect {name} to unlock it in every client.")),
        (None, None) => None,
    };
    summary
}

fn group(connections: &[Connection]) -> Vec<GroupView> {
    GROUPS
        .iter()
        .map(|(key, title, note)| {
            let cards: Vec<_> = connections
                .iter()
                .filter(|c| group_key(c) == *key)
                .map(card)
                .collect();
            GroupView {
                key,
                title,
                note,
                count: cards.len(),
                cards,
            }
        })
        .filter(|g| !g.cards.is_empty())
        .collect()
}

pub(crate) async fn connectors_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    let snapshot = get_connections(&pool, &user_ctx.user_id).await?;
    let connections = snapshot.connections;
    let view = ConnectorsPageView {
        page: "connectors",
        title: "Connectors",
        user_email: user_ctx.email.as_str().to_owned(),
        user_id: user_ctx.user_id.clone(),
        summary: summarise(&connections),
        has_connectors: !connections.is_empty(),
        groups: group(&connections),
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Connectors"),
        ],
    };
    Ok(render_typed_page(
        &engine,
        "connectors",
        &view,
        &user_ctx,
        &mkt_ctx,
    ))
}
