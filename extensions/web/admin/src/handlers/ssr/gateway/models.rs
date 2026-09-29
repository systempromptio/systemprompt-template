//! The Overview and Providers tabs: one row per provider, in
//! `providers.yaml` order, with the routes that reach it and what they
//! carry; and the dispatch table, one row per resolved route in the order
//! the dispatcher tries them.
//!
//! The routes table answers "in what order are patterns tried" and is where
//! order is edited; these views answer the question an operator actually
//! brings — which providers are live, which models they serve, and which
//! routes deliver traffic there. They are read from the same labels every
//! other page uses, so a route is named here exactly as it is in access
//! control.

use std::collections::HashMap;

use serde::Serialize;

use super::view::GatewayRouteRow;
use crate::repositories::config::gateway::{RouteLabel, RouteLabels};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProviderRouteView {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub model_pattern: String,
    pub upstream: Option<String>,
    pub fallback: Option<String>,
    pub requires: String,
    pub declared: bool,
    pub position: Option<usize>,
    pub grants: i64,
    pub matrix_url: String,
}

// Why: the overview's dispatch table — a resolved route with the provider it
// lands on, so the whole chain from pattern to upstream reads on one line.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct DispatchRow {
    pub route: ProviderRouteView,
    pub provider: String,
    pub provider_label: String,
    pub surface: String,
    pub surface_tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProviderCardView {
    pub id: String,
    pub anchor: String,
    pub label: String,
    pub description: Option<String>,
    pub endpoint: String,
    pub surface: String,
    pub surface_tone: &'static str,
    pub wire: String,
    pub advertised: bool,
    pub model_count: usize,
    pub models: Vec<String>,
    pub routes: Vec<ProviderRouteView>,
    pub route_count: usize,
    pub unreachable: bool,
    pub state_label: &'static str,
    pub state_tone: &'static str,
    pub fallback_for: Vec<String>,
    pub grants_total: i64,
    pub reached_by_url: String,
}

// Why: a route's access is read and edited on its own page, in the shared
// "Who gets this" panel.
pub(super) fn matrix_url(route: &str) -> String {
    format!(
        "{}/{}#access",
        super::route_detail::ROUTES_URL,
        urlencoding::encode(route)
    )
}

fn route_view(
    label: &RouteLabel,
    declared: &HashMap<&str, &GatewayRouteRow>,
    grants: &HashMap<String, i64>,
) -> ProviderRouteView {
    let row = declared.get(label.id.as_str());
    ProviderRouteView {
        id: label.id.clone(),
        label: label.label.clone(),
        description: label.description.clone(),
        model_pattern: label.model_pattern.clone(),
        upstream: label.upstream.clone(),
        fallback: label.fallback.clone(),
        requires: row.map(|r| r.requires.clone()).unwrap_or_default(),
        declared: row.is_some(),
        position: row.map(|r| r.index + 1),
        grants: grants.get(&label.id).copied().unwrap_or(0),
        matrix_url: matrix_url(&label.id),
    }
}

const fn surface_tone(advertised: bool) -> &'static str {
    if advertised { "ok" } else { "warn" }
}

pub(super) fn dispatch_rows(
    labels: &RouteLabels,
    rows: &[GatewayRouteRow],
    grants: &HashMap<String, i64>,
) -> Vec<DispatchRow> {
    let declared: HashMap<&str, &GatewayRouteRow> =
        rows.iter().map(|r| (r.id.as_str(), r)).collect();
    labels
        .routes
        .iter()
        .map(|r| {
            let provider = labels.find_provider(&r.provider);
            DispatchRow {
                route: route_view(r, &declared, grants),
                provider: r.provider.clone(),
                provider_label: r.provider_label.clone(),
                surface: provider.map_or_else(|| "unknown".to_owned(), |p| p.surface.clone()),
                surface_tone: provider.map_or("err", |p| surface_tone(p.advertised)),
            }
        })
        .collect()
}

pub(super) fn provider_cards(
    labels: &RouteLabels,
    rows: &[GatewayRouteRow],
    grants: &HashMap<String, i64>,
) -> Vec<ProviderCardView> {
    let declared: HashMap<&str, &GatewayRouteRow> =
        rows.iter().map(|r| (r.id.as_str(), r)).collect();
    labels
        .providers
        .iter()
        .map(|p| {
            let routes: Vec<ProviderRouteView> = labels
                .routes
                .iter()
                .filter(|r| r.provider == p.id)
                .map(|r| route_view(r, &declared, grants))
                .collect();
            let fallback_for = labels
                .routes
                .iter()
                .filter(|r| {
                    r.fallback
                        .as_deref()
                        .is_some_and(|f| f.starts_with(p.label.as_str()))
                })
                .map(|r| r.label.clone())
                .collect();
            let unreachable = routes.is_empty();
            ProviderCardView {
                id: p.id.clone(),
                anchor: format!("provider-{}", p.id),
                label: p.label.clone(),
                description: p.description.clone(),
                endpoint: p.endpoint.clone(),
                surface: p.surface.clone(),
                surface_tone: surface_tone(p.advertised),
                wire: p.wire.clone(),
                advertised: p.advertised,
                model_count: p.model_count,
                models: p.models.clone(),
                route_count: routes.len(),
                unreachable,
                state_label: if unreachable { "no route" } else { "reachable" },
                state_tone: if unreachable { "err" } else { "ok" },
                grants_total: routes.iter().map(|r| r.grants).sum(),
                reached_by_url: format!(
                    "/admin/access-control?entity_kind=gateway_route&q={}",
                    urlencoding::encode(&p.id)
                ),
                routes,
                fallback_for,
            }
        })
        .collect()
}
