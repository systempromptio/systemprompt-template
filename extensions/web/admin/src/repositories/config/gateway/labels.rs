//! Human-facing labels for gateway routes and providers.
//!
//! A route id is generated (`slug + fnv1a6`) and a provider id is a key, so
//! neither reads well on a page. Every console surface that names a route —
//! the access-control rules, the audience grid, a person's matrix, the role
//! entitlements and the gateway page — resolves it here, so the same route
//! carries the same label everywhere. A declared `name` wins; otherwise the
//! label is derived from the model pattern and the provider's display name,
//! and the generated id is demoted to a subtitle the page may still show.

use std::collections::HashMap;

use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::ServicesConfig;
use systemprompt::models::services::ProviderEntry;
use systemprompt_web_shared::error::MarketplaceError;

use super::catalog::dispatchable_routes;

#[derive(Debug, Clone)]
pub struct RouteLabel {
    pub id: String,
    pub label: String,
    pub declared_name: bool,
    pub description: Option<String>,
    pub model_pattern: String,
    pub provider: String,
    pub provider_label: String,
    pub upstream: Option<String>,
    pub endpoint: String,
    pub surface: String,
    pub wire: String,
    pub fallback: Option<String>,
}

impl RouteLabel {
    // Why: One line under the label: what the route matches and where it goes.
    #[must_use]
    pub fn subtitle(&self) -> String {
        let mut s = format!("{} → {}", self.model_pattern, self.provider_label);
        if let Some(upstream) = &self.upstream {
            s.push_str(&format!(" as {upstream}"));
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct ProviderLabel {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub endpoint: String,
    pub surface: String,
    pub wire: String,
    pub model_count: usize,
    pub advertised: bool,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RouteLabels {
    by_id: HashMap<String, RouteLabel>,
    pub routes: Vec<RouteLabel>,
    pub providers: Vec<ProviderLabel>,
}

impl RouteLabels {
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&RouteLabel> {
        self.by_id.get(id)
    }

    // Why: The label for a route id, or the id itself for one this catalog does
    // not know — never an empty string.
    #[must_use]
    pub fn label_of(&self, id: &str) -> String {
        self.find(id)
            .map_or_else(|| id.to_owned(), |l| l.label.clone())
    }

    #[must_use]
    pub fn find_provider(&self, id: &str) -> Option<&ProviderLabel> {
        self.providers.iter().find(|p| p.id == id)
    }
}

pub fn get_route_labels(services: &ServicesConfig) -> Result<RouteLabels, MarketplaceError> {
    let providers: Vec<ProviderLabel> = services
        .providers
        .providers
        .iter()
        .map(provider_label)
        .collect();
    let routes: Vec<RouteLabel> = dispatchable_routes(services)?
        .into_iter()
        .map(|route| {
            let entry = services.providers.find_provider(&route.provider);
            let provider_label = derive_provider_label(&route.provider, entry);
            let declared = route
                .name
                .as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty());
            let fallback = route.fallback_provider.as_deref().map(|fp| {
                let fp_label = derive_provider_label(fp, services.providers.find_provider(fp));
                match &route.fallback_upstream_model {
                    Some(model) => format!("{fp_label} as {model}"),
                    None => fp_label,
                }
            });
            RouteLabel {
                label: declared.map_or_else(
                    || derive_route_label(&route.model_pattern, &provider_label),
                    str::to_owned,
                ),
                declared_name: declared.is_some(),
                id: route.id,
                description: route.description,
                model_pattern: route.model_pattern,
                provider: route.provider,
                provider_label,
                upstream: route.upstream_model,
                endpoint: entry.map(|e| e.endpoint.clone()).unwrap_or_default(),
                surface: entry
                    .map(|e| e.surface.as_tag().to_owned())
                    .unwrap_or_default(),
                wire: entry.map(|e| e.wire.to_string()).unwrap_or_default(),
                fallback,
            }
        })
        .collect();
    let by_id = routes.iter().map(|r| (r.id.clone(), r.clone())).collect();
    Ok(RouteLabels {
        by_id,
        routes,
        providers,
    })
}

pub fn get_route_labels_from_services() -> Result<RouteLabels, MarketplaceError> {
    get_route_labels(ServicesBootstrap::get()?)
}

fn provider_label(entry: &ProviderEntry) -> ProviderLabel {
    ProviderLabel {
        id: entry.name.as_str().to_owned(),
        label: derive_provider_label(entry.name.as_str(), Some(entry)),
        description: entry.description.clone(),
        endpoint: entry.endpoint.clone(),
        surface: entry.surface.as_tag().to_owned(),
        wire: entry.wire.to_string(),
        model_count: entry.models.len(),
        advertised: entry.surface.is_advertised(),
        models: entry
            .models
            .iter()
            .map(|m| m.id.as_str().to_owned())
            .collect(),
    }
}

// Why: The provider's display name, falling back to a known spelling of the id
// and then to the id title-cased.
#[must_use]
pub fn derive_provider_label(id: &str, entry: Option<&ProviderEntry>) -> String {
    if let Some(name) = entry
        .and_then(|e| e.display_name.as_deref())
        .map(str::trim)
        .filter(|n| !n.is_empty())
    {
        return name.to_owned();
    }
    match id {
        "anthropic" => "Anthropic".to_owned(),
        "openai" => "OpenAI".to_owned(),
        "openai-responses" => "OpenAI (Responses)".to_owned(),
        "gemini" => "Google Gemini".to_owned(),
        "vertex" => "Vertex AI".to_owned(),
        "vertex-maas" => "Vertex AI MaaS".to_owned(),
        "cerebras" => "Cerebras".to_owned(),
        other => title_case(other),
    }
}

// Why: A label for a route with no declared name: the model family the pattern
// names, then the provider it reaches.
#[must_use]
pub fn derive_route_label(model_pattern: &str, provider_label: &str) -> String {
    let pattern = model_pattern.trim();
    if pattern == "*" {
        return format!("Everything else · {provider_label}");
    }
    let stem = pattern.trim_matches('*');
    let family = title_case(stem.trim_matches(|c| c == '-' || c == '.'));
    if pattern.contains('*') {
        format!("{family} models · {provider_label}")
    } else {
        format!("{family} · {provider_label}")
    }
}

fn title_case(id: &str) -> String {
    id.split(['-', '_', '.'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}
