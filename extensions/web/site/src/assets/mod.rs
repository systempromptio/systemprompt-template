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
    let storage_video = paths.storage_files().join("video");

    let mut assets = css::css_assets(&storage_css);
    assets.push(AssetDefinition::image(
        storage_video.join("showreel.mp4"),
        "video/showreel.mp4",
    ));
    // Why: client downloads (`storage/files/downloads/`) are deliberately NOT
    // declared here. `/files/**` is served straight from `storage/files/`, so
    // the bridge tarball and `install.sh` are already reachable at
    // `/files/downloads/<asset>` without being copied into `web/dist`.
    assets.extend(js_services::public_js_assets(&storage_js));
    assets.extend(js_services::service_js_assets(&storage_js));
    assets
}
