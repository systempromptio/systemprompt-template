//! Static asset manifest for the public site.
//!
//! Every CSS and JS file the extension trait advertises is enumerated here;
//! sources live under `storage/files/`, never alongside the Rust code.

#[doc(hidden)]
pub mod css;
#[doc(hidden)]
pub mod js_services;

use systemprompt::extension::AssetDefinition;

pub fn web_assets(paths: &dyn systemprompt::extension::AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");

    let mut assets = css::css_assets(&storage_css);
    assets.extend(js_services::public_js_assets(&storage_js));
    assets.extend(js_services::service_js_assets(&storage_js));
    assets
}
