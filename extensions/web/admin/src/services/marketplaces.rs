use systemprompt::loader::ConfigLoader;
use systemprompt::models::services::MarketplaceConfig;

/// Load the YAML-defined marketplaces from `services/marketplaces/*/config.yaml`.
///
/// Returns an empty list if config loading fails — admin pages must still render.
/// Sorted by marketplace id for stable ordering across requests.
pub fn load_marketplaces() -> Vec<MarketplaceConfig> {
    let Ok(services) = ConfigLoader::load() else {
        return Vec::new();
    };

    let mut entries: Vec<MarketplaceConfig> = services.marketplaces.into_values().collect();
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    entries
}

/// Inverse map: `plugin_id` -> `[(marketplace_id, marketplace_name)]` for badge rendering.
pub fn plugin_to_marketplaces() -> std::collections::HashMap<String, Vec<(String, String)>> {
    let mut map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    for mp in load_marketplaces() {
        for plugin_id in &mp.plugins.include {
            map.entry(plugin_id.clone())
                .or_default()
                .push((mp.id.as_str().to_string(), mp.name.clone()));
        }
    }
    map
}
