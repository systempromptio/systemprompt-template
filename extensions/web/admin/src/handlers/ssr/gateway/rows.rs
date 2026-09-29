//! Shapes the gateway route table: provider surfaces, declared and
//! resolved-only rows, and the KPI strip above them.

use super::models::matrix_url;
use super::view::{GatewayKpiView, GatewayRouteRow, ProviderOptionView, ResolvedOnlyRow};
use crate::repositories::config::gateway::{RouteLabels, derive_route_label};
use crate::types::{GatewayConfigView, GatewayRouteView};

fn yaml_summary(value: Option<&serde_yaml::Value>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    serde_yaml::to_string(value)
        .inspect_err(|e| tracing::warn!(error = %e, "gateway: yaml render failed"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && *l != "---")
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) struct Surfaces {
    pub(super) providers: Vec<ProviderOptionView>,
}

impl Surfaces {
    pub(super) fn surface_of(&self, provider: &str) -> (String, &'static str) {
        match self.providers.iter().find(|p| p.name == provider) {
            None => ("unknown".to_owned(), "err"),
            Some(p) if p.advertised => (p.surface.clone(), "ok"),
            Some(p) => (p.surface.clone(), "warn"),
        }
    }
}

pub(super) fn load_surfaces(labels: &RouteLabels) -> Surfaces {
    let providers = labels
        .providers
        .iter()
        .map(|p| ProviderOptionView {
            name: p.id.clone(),
            label: p.label.clone(),
            surface: p.surface.clone(),
            model_count: p.model_count,
            advertised: p.advertised,
        })
        .collect();
    Surfaces { providers }
}

pub(super) fn route_rows(
    config: &GatewayConfigView,
    surfaces: &Surfaces,
    grants: &std::collections::HashMap<String, i64>,
    dispatchable: &[String],
    labels: &RouteLabels,
) -> Vec<GatewayRouteRow> {
    let last = config.routes.len().saturating_sub(1);
    config
        .routes
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let (surface, surface_tone) = surfaces.surface_of(&route.provider);
            // Why: the table reads the DB rows, which a console save updates
            // at once; the label resolver reads the booted services and lags
            // a restart behind. A name on the row wins so an edit shows.
            let provider_label = labels
                .find_provider(&route.provider)
                .map_or_else(|| route.provider.clone(), |p| p.label.clone());
            let label = route
                .name
                .as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map_or_else(
                    || derive_route_label(&route.model_pattern, &provider_label),
                    str::to_owned,
                );
            GatewayRouteRow {
                index,
                surface,
                surface_tone,
                name: route.name.clone().unwrap_or_default(),
                description: route.description.clone().unwrap_or_default(),
                label,
                provider_label,
                upstream_model: route.upstream_model.clone().unwrap_or_default(),
                requires: yaml_summary(route.requires.as_ref()),
                when: yaml_summary(route.when.as_ref()),
                has_pricing: route.pricing.is_some(),
                header_count: route.extra_headers.len(),
                grants: grants.get(&route.id).copied().unwrap_or(0),
                dispatchable: dispatchable.contains(&route.id),
                is_first: index == 0,
                is_last: index == last,
                matrix_url: matrix_url(&route.id),
                id: route.id.clone(),
                model_pattern: route.model_pattern.clone(),
                provider: route.provider.clone(),
            }
        })
        .collect()
}

pub(super) fn resolved_only(
    declared: &[GatewayRouteRow],
    resolved: &[GatewayRouteView],
    labels: &RouteLabels,
) -> Vec<ResolvedOnlyRow> {
    resolved
        .iter()
        .filter(|r| !declared.iter().any(|d| d.id == r.id))
        .map(|r| ResolvedOnlyRow {
            matrix_url: matrix_url(&r.id),
            label: labels.label_of(&r.id),
            id: r.id.clone(),
            model_pattern: r.model_pattern.clone(),
            provider: r.provider.clone(),
            upstream_model: r.upstream_model.clone().unwrap_or_default(),
        })
        .collect()
}

pub(super) fn kpis(
    config: &GatewayConfigView,
    rows: &[GatewayRouteRow],
    resolved_extra: usize,
    surfaces: &Surfaces,
) -> Vec<GatewayKpiView> {
    let backend = rows.iter().filter(|r| r.surface == "backend").count();
    let unknown = rows.iter().filter(|r| r.surface == "unknown").count();
    let grants: i64 = rows.iter().map(|r| r.grants).sum();
    let governed = rows.iter().filter(|r| !r.requires.is_empty()).count();
    vec![
        GatewayKpiView {
            label: "Gateway",
            value: if config.enabled { "On" } else { "Off" }.to_owned(),
            note: format!("{} on {}", config.auth_scheme, config.inference_path_prefix),
            tone: if config.enabled { "ok" } else { "warn" },
        },
        GatewayKpiView {
            label: "Routes declared",
            value: rows.len().to_string(),
            note: format!("{resolved_extra} resolved but not in the file"),
            tone: "",
        },
        GatewayKpiView {
            label: "Providers",
            value: surfaces.providers.len().to_string(),
            note: format!("{backend} routes on a backend-only provider"),
            tone: "",
        },
        GatewayKpiView {
            label: "Unknown provider",
            value: unknown.to_string(),
            note: "routes naming a provider the registry lacks".to_owned(),
            tone: if unknown > 0 { "err" } else { "ok" },
        },
        GatewayKpiView {
            label: "Governed routes",
            value: governed.to_string(),
            note: "carry a requires: classification".to_owned(),
            tone: "",
        },
        GatewayKpiView {
            label: "Route grants",
            value: grants.to_string(),
            note: "access-control rules on routes".to_owned(),
            tone: "",
        },
    ]
}
