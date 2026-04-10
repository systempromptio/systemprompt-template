mod css;
mod js_pages;
mod js_services;

use systemprompt::extension::AssetDefinition;

pub fn web_assets(paths: &dyn systemprompt::extension::AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");
    let storage_admin = paths.storage_files().join("admin").join("compiled");

    let mut assets = css::css_assets(&storage_css);
    assets.extend(js_services::public_js_assets(&storage_js));
    assets.extend(js_services::service_js_assets(&storage_js));
    assets.extend(js_services::admin_assets(&storage_css, &storage_js));
    assets.extend(js_pages::page_js_assets(&storage_js));
    assets.extend(js_services::admin_html_assets(&storage_admin));
    assets
}
