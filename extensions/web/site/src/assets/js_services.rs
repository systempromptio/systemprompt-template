//! Shared JavaScript service module definitions.

use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! svc_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/services/", $name))
    };
}

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

#[doc(hidden)]
pub fn service_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let p = storage_js.join("services");
    let mut v = service_core_js(&p);
    v.extend(service_utils_js(storage_js));
    v
}

fn service_core_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        svc_js!(p, "api.js"),
        svc_js!(p, "auth.js"),
        svc_js!(p, "bootstrap.js"),
        svc_js!(p, "confirm.js"),
        svc_js!(p, "dropdown.js"),
        svc_js!(p, "events.js"),
        svc_js!(p, "export-columns.js"),
        svc_js!(p, "export-formats.js"),
        svc_js!(p, "export-preview.js"),
        svc_js!(p, "export-scope.js"),
        svc_js!(p, "export-url.js"),
        svc_js!(p, "export.js"),
        svc_js!(p, "filter-ribbon.js"),
        svc_js!(p, "header-actions.js"),
        svc_js!(p, "header-search-list.js"),
        svc_js!(p, "header-search.js"),
        svc_js!(p, "nav-groups.js"),
        svc_js!(p, "scope.js"),
        svc_js!(p, "sidebar.js"),
        svc_js!(p, "table-expand.js"),
        svc_js!(p, "toast.js"),
        svc_js!(p, "validity.js"),
    ]
}

fn service_utils_js(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            storage_js.join("components/sp-toast.js"),
            "js/components/sp-toast.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-confirm-dialog.js"),
            "js/components/sp-confirm-dialog.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-confirm-dialog-view.js"),
            "js/components/sp-confirm-dialog-view.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-copy.js"),
            "js/components/sp-copy.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-tabs.js"),
            "js/components/sp-tabs.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-help.js"),
            "js/components/sp-help.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-table-select.js"),
            "js/components/sp-table-select.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart.js"),
            "js/components/sp-chart.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-scale.js"),
            "js/components/sp-chart-scale.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-draw.js"),
            "js/components/sp-chart-draw.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-tooltip.js"),
            "js/components/sp-chart-tooltip.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-sync-plane.js"),
            "js/components/sp-sync-plane.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-access-review.js"),
            "js/components/sp-access-review.js",
        ),
    ]
}
