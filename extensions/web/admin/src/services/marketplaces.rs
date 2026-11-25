//! Loads the configured marketplace catalog.

use crate::env::AdminEnv;
use systemprompt::manifest::services::MarketplaceConfig;

// Why: a config that fails to load yields an empty list rather than an error —
// admin pages must still render.
pub(crate) fn load_marketplaces(env: &AdminEnv) -> Vec<MarketplaceConfig> {
    let services = env.services();
    let mut entries: Vec<MarketplaceConfig> = services.marketplaces.values().cloned().collect();
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    entries
}
