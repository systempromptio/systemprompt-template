//! Marketplace manifests read from `services/marketplaces/*/config.yaml`.
//!
//! A marketplace is the unit entitlement is granted on, but who reaches it is
//! no longer written in its manifest: it is declared in
//! `services/access-control/rules.yaml` and enforced from the database. The
//! `access` summary on each manifest is therefore filled from the database by
//! [`super::manifests_access::attach_access`], never from this file, so the
//! catalog shows what is enforced rather than what a file once said.
//!
//! Every failure is a skip with a warning: a malformed manifest must not blank
//! the whole catalog page.

use std::path::Path;
use systemprompt::identifiers::MarketplaceId;

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
    pub id: MarketplaceId,
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
        id: MarketplaceId::new(id.clone()),
        version: text("version", "0.0.0"),
        description: text("description", ""),
        visibility: text("visibility", "private"),
        enabled: marketplace
            .get("enabled")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(true),
        access: MarketplaceAccessSummary::default(),
        plugins: member_ids(marketplace, "plugins"),
        mcp_servers: member_ids(marketplace, "mcp_servers"),
        agents: member_ids(marketplace, "agents"),
        source_path,
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
