use std::path::Path;

pub(super) struct MarketplaceIdentity {
    pub name: String,
    pub owner_name: String,
    pub owner_email: String,
}

pub(super) fn load_marketplace_identity(services_path: &Path) -> MarketplaceIdentity {
    let metadata_path = services_path.join("web/metadata.yaml");
    if let Ok(content) = std::fs::read_to_string(&metadata_path) {
        if let Ok(val) = serde_yaml::from_str::<serde_json::Value>(&content) {
            let site_name = val
                .get("site")
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("marketplace");
            let site_author = val
                .get("site")
                .and_then(|s| s.get("author"))
                .and_then(|a| a.as_str())
                .unwrap_or(site_name);
            let base_url = val
                .get("site")
                .and_then(|s| s.get("baseUrl"))
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let email = if base_url.is_empty() {
                String::new()
            } else {
                let domain = base_url
                    .trim_start_matches("https://")
                    .trim_start_matches("http://");
                format!("hello@{domain}")
            };

            return MarketplaceIdentity {
                name: format!("{}-marketplace", site_name.to_lowercase().replace(' ', "-")),
                owner_name: site_author.to_string(),
                owner_email: email,
            };
        }
    }
    MarketplaceIdentity {
        name: "marketplace".to_string(),
        owner_name: "Marketplace".to_string(),
        owner_email: String::new(),
    }
}

pub(super) fn build_marketplace(
    plugin_configs: &[(String, super::super::types::PlatformPluginConfig)],
    bundles: &[super::export::PluginBundle],
    identity: &MarketplaceIdentity,
) -> Result<super::export::MarketplaceFile, anyhow::Error> {
    let org_ids: std::collections::HashSet<&str> =
        plugin_configs.iter().map(|(id, _)| id.as_str()).collect();

    let mut plugin_entries: Vec<serde_json::Value> = plugin_configs
        .iter()
        .map(|(_, plugin)| {
            let mut entry = serde_json::json!({
                "name": plugin.base.id,
                "source": format!("./plugins/{}", plugin.base.id),
                "description": plugin.base.description,
            });
            if let Some(bundle) = bundles.iter().find(|b| b.id == plugin.base.id) {
                entry["version"] = serde_json::Value::String(bundle.version.clone());
            } else {
                entry["version"] = serde_json::Value::String(plugin.base.version.clone());
            }
            let mut author_obj = serde_json::json!({ "name": plugin.base.author.name });
            author_obj["email"] = serde_json::Value::String(plugin.base.author.email.clone());
            entry["author"] = author_obj;
            entry["category"] = serde_json::Value::String(plugin.base.category.clone());
            entry
        })
        .collect();

    for bundle in bundles {
        if org_ids.contains(bundle.id.as_str()) {
            continue;
        }
        plugin_entries.push(serde_json::json!({
            "name": bundle.id,
            "source": format!("./plugins/{}", bundle.id),
            "description": bundle.description,
            "version": bundle.version,
        }));
    }

    let mut marketplace = serde_json::json!({
        "$schema": "https://anthropic.com/claude-code/marketplace.schema.json",
        "name": identity.name,
        "description": format!("{}'s plugin marketplace", identity.owner_name),
        "owner": {
            "name": identity.owner_name,
        },
        "metadata": {
            "description": format!("Plugin marketplace exported from {}", identity.owner_name),
            "version": "1.0.0",
            "pluginRoot": "./plugins"
        },
        "plugins": plugin_entries
    });
    if !identity.owner_email.is_empty() {
        marketplace["owner"]["email"] = serde_json::Value::String(identity.owner_email.clone());
    }

    Ok(super::export::MarketplaceFile {
        path: ".claude-plugin/marketplace.json".to_string(),
        content: serde_json::to_string_pretty(&marketplace)?,
    })
}
