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

pub(super) fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let site = storage_js.join("site");
    vec![
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("blog-list.js"), "js/blog-list.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("homepage.js"), "js/homepage.js"),
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

pub(super) fn service_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let p = storage_js.join("services");
    let mut v = service_core_js(&p);
    v.extend(service_plugin_js(&p));
    v.extend(service_webauthn_js(&p));
    v.extend(service_utils_js(storage_js));
    v
}

fn service_core_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        svc_js!(p, "admin-token.js"),
        svc_js!(p, "api.js"),
        svc_js!(p, "auth.js"),
        svc_js!(p, "bootstrap.js"),
        svc_js!(p, "confirm.js"),
        svc_js!(p, "dropdown.js"),
        svc_js!(p, "events.js"),
        svc_js!(p, "filter-ribbon.js"),
        svc_js!(p, "header-actions.js"),
        svc_js!(p, "header-search.js"),
        svc_js!(p, "sidebar.js"),
        svc_js!(p, "theme.js"),
        svc_js!(p, "toast.js"),
    ]
}

fn service_plugin_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
    ]
}

fn service_webauthn_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        svc_js!(p, "webauthn-helpers.js"),
        svc_js!(p, "webauthn-session.js"),
        svc_js!(p, "webauthn-login.js"),
        svc_js!(p, "webauthn-login-ui.js"),
        svc_js!(p, "webauthn-passkey.js"),
        svc_js!(p, "webauthn-passkey-helpers.js"),
        svc_js!(p, "webauthn-utils.js"),
    ]
}

fn service_utils_js(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            storage_js.join("utils/storage-safe.js"),
            "js/utils/storage-safe.js",
        ),
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
    ]
}
