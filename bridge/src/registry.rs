//! Astound-specific bridge behaviour registered through core's `inventory`
//! seams. Linked by `main.rs` (`mod registry;`) so these submissions survive
//! into the final binary — an unreferenced module would be dropped by the
//! linker before its `inventory::submit!` initializers ran.
//!
//! This is the white-label counterpart to the build-time web overlay: the
//! overlay swaps GUI presentation, this adds server-side behaviour (host apps,
//! host syncs, marketplace sources) with no edits to core.

use systemprompt_bridge::gui::server_marketplace::MarketplaceItem;
use systemprompt_bridge::gui::server_marketplace::source::{
    MarketplaceCategory, MarketplaceSource, MarketplaceSourceCtx,
};
use systemprompt_bridge::register_marketplace_source;

struct AstoundArtifactsSource;

impl MarketplaceSource for AstoundArtifactsSource {
    fn category(&self) -> MarketplaceCategory {
        MarketplaceCategory::Artifacts
    }

    fn items(&self, _ctx: &MarketplaceSourceCtx<'_>) -> Vec<MarketplaceItem> {
        vec![MarketplaceItem::new(
            "astound-welcome",
            "Astound Digital — Welcome",
            Some("Branded starter artifact contributed by the Astound bridge.".to_owned()),
            String::new(),
            "astound",
        )]
    }
}

register_marketplace_source!(AstoundArtifactsSource, priority = 10);
