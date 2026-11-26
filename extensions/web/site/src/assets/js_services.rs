//! Public-site JavaScript definitions. The service modules and components the
//! console also loads are declared by the admin crate, which every build links.

use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! site_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/site/", $name))
    };
}

#[doc(hidden)]
pub fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let site = storage_js.join("site");
    vec![
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("homepage.js"), "js/homepage.js"),
        AssetDefinition::js(storage_js.join("motion-flag.js"), "js/motion-flag.js"),
        site_js!(&site, "analytics-handlers.js"),
        site_js!(&site, "analytics-metrics.js"),
        site_js!(&site, "analytics-state.js"),
        site_js!(&site, "analytics-transport.js"),
        site_js!(&site, "copy-buttons.js"),
        site_js!(&site, "docs-evidence-gallery.js"),
        site_js!(&site, "docs-export.js"),
        site_js!(&site, "docs-nav.js"),
        site_js!(&site, "docs-pagination.js"),
        site_js!(&site, "docs-toc.js"),
        site_js!(&site, "dom-throttle.js"),
        site_js!(&site, "mcp-connect-modal.js"),
        site_js!(&site, "status-api.js"),
        site_js!(&site, "status-card.js"),
        site_js!(&site, "status-render.js"),
    ]
}
