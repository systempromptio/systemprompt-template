//! Loads the configured marketplace catalog.

use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::MarketplaceConfig;

// Why: a config that fails to load yields an empty list rather than an error —
// admin pages must still render.
pub(crate) fn load_marketplaces() -> Vec<MarketplaceConfig> {
    let Ok(services) = ConfigLoader::load() else {
        return Vec::new();
    };

    let mut entries: Vec<MarketplaceConfig> = services.marketplaces.into_values().collect();
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    entries
}
