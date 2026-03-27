use super::export::{
    ManifestAuthor, MarketplaceManifest, MarketplaceMetadata, MarketplacePluginEntry,
};
use crate::error::MarketplaceError;

pub(super) fn build_marketplace(
    plugin_configs: &[(String, super::super::types::PlatformPluginConfig)],
    bundles: &[super::export::PluginBundle],
    username: &str,
    email: &str,
) -> Result<super::export::MarketplaceFile, MarketplaceError> {
    let org_ids: std::collections::HashSet<&str> =
        plugin_configs.iter().map(|(id, _)| id.as_str()).collect();

    let mut plugin_entries: Vec<MarketplacePluginEntry> = plugin_configs
        .iter()
        .map(|(_, plugin)| build_org_plugin_entry(plugin, bundles))
        .collect();

    for bundle in bundles {
        if org_ids.contains(bundle.id.as_str()) {
            continue;
        }
        plugin_entries.push(build_user_plugin_entry(bundle, username, email));
    }

    let slug: String = username
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let marketplace = MarketplaceManifest {
        schema: "https://anthropic.com/claude-code/marketplace.schema.json".to_string(),
        name: format!("{slug}-marketplace"),
        description: format!("{username}'s plugin marketplace"),
        metadata: MarketplaceMetadata {
            description: "Plugin marketplace exported from systemprompt.io".to_string(),
            version: "1.0.0".to_string(),
            plugin_root: "./plugins".to_string(),
        },
        owner: ManifestAuthor {
            name: username.to_string(),
            email: email.to_string(),
        },
        plugins: plugin_entries,
    };

    Ok(super::export::MarketplaceFile {
        path: ".claude-plugin/marketplace.json".to_string(),
        content: serde_json::to_string_pretty(&marketplace)?,
    })
}

fn build_org_plugin_entry(
    plugin: &super::super::types::PlatformPluginConfig,
    bundles: &[super::export::PluginBundle],
) -> MarketplacePluginEntry {
    let version = bundles
        .iter()
        .find(|b| b.id == plugin.base.id)
        .map_or_else(|| plugin.base.version.clone(), |b| b.version.clone());
    MarketplacePluginEntry {
        name: plugin.base.id.clone(),
        source: format!("./plugins/{}", plugin.base.id),
        description: plugin.base.description.clone(),
        version,
        author: Some(ManifestAuthor {
            name: plugin.base.author.name.clone(),
            email: plugin.base.author.email.clone(),
        }),
        category: Some(plugin.base.category.clone()),
    }
}

fn build_user_plugin_entry(
    bundle: &super::export::PluginBundle,
    username: &str,
    email: &str,
) -> MarketplacePluginEntry {
    let author = if username.is_empty() {
        None
    } else {
        Some(ManifestAuthor {
            name: username.to_string(),
            email: email.to_string(),
        })
    };
    MarketplacePluginEntry {
        name: bundle.name.clone(),
        source: format!("./plugins/{}", bundle.id),
        description: bundle.description.clone(),
        version: bundle.version.clone(),
        author,
        category: None,
    }
}
