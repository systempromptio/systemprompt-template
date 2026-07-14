use systemprompt::loader::ConfigLoader;
use systemprompt::models::services::MarketplaceConfig;

/// Load the YAML-defined marketplaces from
/// `services/marketplaces/*/config.yaml`.
///
/// Returns an empty list if config loading fails — admin pages must still
/// render. Sorted by marketplace id for stable ordering across requests.
pub(crate) fn load_marketplaces() -> Vec<MarketplaceConfig> {
    let Ok(services) = ConfigLoader::load() else {
        return Vec::new();
    };

    let mut entries: Vec<MarketplaceConfig> = services.marketplaces.into_values().collect();
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    entries
}
