//! Shapes the gateway route table: provider surfaces, declared and
//! resolved-only rows, and the KPI strip above them.

use systemprompt::loader::ServicesBootstrap;

use super::view::{GatewayKpiView, GatewayRouteRow, ProviderOptionView, ResolvedOnlyRow};
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

pub(super) fn load_surfaces() -> Surfaces {
    let providers = ServicesBootstrap::get().map_or_else(
        |_| Vec::new(),
        |services| {
            services
                .providers
                .providers
                .iter()
                .map(|p| ProviderOptionView {
                    name: p.name.as_str().to_owned(),
                    surface: p.surface.as_tag().to_owned(),
                    model_count: p.models.len(),
                    advertised: p.surface.is_advertised(),
                })
                .collect()
        },
    );
    Surfaces { providers }
}

pub(super) fn route_rows(
    config: &GatewayConfigView,
    surfaces: &Surfaces,
    grants: &std::collections::HashMap<String, i64>,
    dispatchable: &[String],
) -> Vec<GatewayRouteRow> {
    let last = config.routes.len().saturating_sub(1);
    config
        .routes
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let (surface, surface_tone) = surfaces.surface_of(&route.provider);
            GatewayRouteRow {
                index,
                surface,
                surface_tone,
                upstream_model: route.upstream_model.clone().unwrap_or_default(),
                requires: yaml_summary(route.requires.as_ref()),
                when: yaml_summary(route.when.as_ref()),
                has_pricing: route.pricing.is_some(),
                header_count: route.extra_headers.len(),
                grants: grants.get(&route.id).copied().unwrap_or(0),
                dispatchable: dispatchable.contains(&route.id),
                is_first: index == 0,
                is_last: index == last,
                matrix_url: format!(
                    "/admin/access-control?entity_type=gateway_route&entity_id={}",
                    route.id
                ),
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
) -> Vec<ResolvedOnlyRow> {
    resolved
        .iter()
        .filter(|r| !declared.iter().any(|d| d.id == r.id))
        .map(|r| ResolvedOnlyRow {
            matrix_url: format!(
                "/admin/access-control?entity_type=gateway_route&entity_id={}",
                r.id
            ),
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
            sub: format!("{} on {}", config.auth_scheme, config.inference_path_prefix),
            tone: if config.enabled { "ok" } else { "warn" },
        },
        GatewayKpiView {
            label: "Routes declared",
            value: rows.len().to_string(),
            sub: format!("{resolved_extra} resolved but not in the file"),
            tone: "",
        },
        GatewayKpiView {
            label: "Providers",
            value: surfaces.providers.len().to_string(),
            sub: format!("{backend} routes on a backend-only provider"),
            tone: "",
        },
        GatewayKpiView {
            label: "Unknown provider",
            value: unknown.to_string(),
            sub: "routes naming a provider the registry lacks".to_owned(),
            tone: if unknown > 0 { "err" } else { "ok" },
        },
        GatewayKpiView {
            label: "Governed routes",
            value: governed.to_string(),
            sub: "carry a requires: classification".to_owned(),
            tone: "",
        },
        GatewayKpiView {
            label: "Route grants",
            value: grants.to_string(),
            sub: "access-control rules on routes".to_owned(),
            tone: "",
        },
    ]
}
