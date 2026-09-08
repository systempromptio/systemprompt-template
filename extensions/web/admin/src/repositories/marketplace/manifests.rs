//! Marketplace manifests read from `services/marketplaces/*/config.yaml`.
//!
//! A marketplace is the unit entitlement is granted on: its `access` block
//! declares the roles that may see it and the attribute bands (`group`,
//! `project`, …) that widen or narrow that. The admin catalog pages need the
//! declared audience *as written on disk*, not the resolved decision, so this
//! walk is deliberately a plain YAML read with no database involvement — the
//! resolved answer comes from the access matrix instead.
//!
//! Every failure is a skip with a warning: a malformed manifest must not blank
//! the whole catalog page.

use std::path::Path;

use serde::Serialize;

use crate::util::source_path::display_source_path;
use systemprompt_web_shared::error::MarketplaceError;

/// One `access.rules` band, flattened to the values it names.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceAccessBand {
    pub rule_type: String,
    pub values: Vec<String>,
    pub access: String,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MarketplaceAccessSummary {
    pub default_included: bool,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub projects: Vec<String>,
    pub bands: Vec<MarketplaceAccessBand>,
    pub justification: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceConfigSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub visibility: String,
    pub access: MarketplaceAccessSummary,
    pub plugins: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub agents: Vec<String>,
    pub source_path: String,
}

fn string_list(value: Option<&serde_yaml::Value>) -> Vec<String> {
    value
        .and_then(serde_yaml::Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn member_ids(marketplace: &serde_yaml::Value, key: &str) -> Vec<String> {
    string_list(marketplace.get(key).and_then(|m| m.get("include")))
}

fn read_bands(access: Option<&serde_yaml::Value>) -> Vec<MarketplaceAccessBand> {
    access
        .and_then(|a| a.get("rules"))
        .and_then(serde_yaml::Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(|rule| {
                    let rule_type = rule.get("rule_type")?.as_str()?.to_owned();
                    Some(MarketplaceAccessBand {
                        rule_type,
                        values: string_list(rule.get("values")),
                        access: rule
                            .get("access")
                            .and_then(serde_yaml::Value::as_str)
                            .unwrap_or("allow")
                            .to_owned(),
                        justification: rule
                            .get("justification")
                            .and_then(serde_yaml::Value::as_str)
                            .map(str::to_owned),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// Why: `groups` and `projects` are projections of the bands, not separate YAML
// keys. The catalog pages want them named because those two bands are the only
// ones this deployment's people model uses; the rest stay in `bands` so a new
// dimension still renders rather than vanishing.
fn values_for(bands: &[MarketplaceAccessBand], rule_type: &str) -> Vec<String> {
    bands
        .iter()
        .filter(|b| b.rule_type == rule_type && b.access == "allow")
        .flat_map(|b| b.values.iter().cloned())
        .collect()
}

fn read_access(marketplace: &serde_yaml::Value) -> MarketplaceAccessSummary {
    let access = marketplace.get("access");
    let bands = read_bands(access);
    MarketplaceAccessSummary {
        default_included: access
            .and_then(|a| a.get("default_included"))
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false),
        roles: string_list(access.and_then(|a| a.get("roles"))),
        groups: values_for(&bands, "group"),
        projects: values_for(&bands, "project"),
        justification: access
            .and_then(|a| a.get("justification"))
            .and_then(serde_yaml::Value::as_str)
            .map(str::to_owned),
        bands,
    }
}

fn parse_manifest(
    raw: &str,
    fallback_id: &str,
    source_path: String,
) -> Option<MarketplaceConfigSummary> {
    let doc: serde_yaml::Value = serde_yaml::from_str(raw).ok()?;
    let marketplace = doc.get("marketplace")?;
    let id = marketplace
        .get("id")
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or(fallback_id)
        .to_owned();
    if id.is_empty() {
        return None;
    }
    let text = |key: &str, fallback: &str| {
        marketplace
            .get(key)
            .and_then(serde_yaml::Value::as_str)
            .unwrap_or(fallback)
            .to_owned()
    };
    Some(MarketplaceConfigSummary {
        name: text("name", &id),
        version: text("version", "0.0.0"),
        description: text("description", ""),
        visibility: text("visibility", "private"),
        enabled: marketplace
            .get("enabled")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(true),
        access: read_access(marketplace),
        plugins: member_ids(marketplace, "plugins"),
        mcp_servers: member_ids(marketplace, "mcp_servers"),
        agents: member_ids(marketplace, "agents"),
        source_path,
        id,
    })
}

pub fn list_marketplace_configs(
    services_path: &Path,
) -> Result<Vec<MarketplaceConfigSummary>, MarketplaceError> {
    let root = services_path.join("marketplaces");
    let mut out: Vec<MarketplaceConfigSummary> = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&root)? {
        let dir = entry?.path();
        if !dir.is_dir() {
            continue;
        }
        let cfg = dir.join("config.yaml");
        if !cfg.exists() {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&cfg) else {
            tracing::warn!(path = %cfg.display(), "skipped unreadable marketplace config");
            continue;
        };
        let fallback = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let source_path = display_source_path(&cfg, services_path);
        if let Some(summary) = parse_manifest(&raw, fallback, source_path) {
            out.push(summary);
        } else {
            tracing::warn!(path = %cfg.display(), "skipped invalid marketplace yaml");
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
