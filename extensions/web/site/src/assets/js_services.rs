//! Shared JavaScript service module definitions.

use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! svc_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/services/", $name))
    };
}

pub(super) fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("terminal-demo.js"), "js/terminal-demo.js"),
        AssetDefinition::js(storage_js.join("blog-images.js"), "js/blog-images.js"),
        AssetDefinition::js(storage_js.join("homepage.js"), "js/homepage.js"),
    ]
}

pub(super) fn service_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let p = storage_js.join("services");
    let mut v = service_core_js(&p);
    v.extend(service_plugin_js(&p));
    v.extend(service_skill_js(&p));
    v.extend(service_webauthn_js(&p));
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
        svc_js!(p, "header-actions.js"),
        svc_js!(p, "header-search.js"),
        svc_js!(p, "list-page.js"),
        svc_js!(p, "sidebar.js"),
        svc_js!(p, "table-sort.js"),
        svc_js!(p, "theme.js"),
        svc_js!(p, "sp-confirm-dialog.js"),
        svc_js!(p, "sp-toast.js"),
        svc_js!(p, "toast.js"),
        svc_js!(p, "toc-highlight.js"),
    ]
}

fn service_plugin_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        svc_js!(p, "plugin-details-ui.js"),
        svc_js!(p, "plugin-details.js"),
        svc_js!(p, "plugin-env-ui.js"),
        svc_js!(p, "plugin-env.js"),
        svc_js!(p, "plugin-resources-helpers.js"),
        svc_js!(p, "plugin-resources.js"),
    ]
}

fn service_skill_js(p: &Path) -> Vec<AssetDefinition> {
    vec![svc_js!(p, "skill-files.js")]
}

fn service_webauthn_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        svc_js!(p, "webauthn-helpers.js"),
        svc_js!(p, "webauthn-login.js"),
        svc_js!(p, "webauthn-login-ui.js"),
        svc_js!(p, "webauthn-passkey.js"),
        svc_js!(p, "webauthn-passkey-helpers.js"),
        svc_js!(p, "webauthn-utils.js"),
    ]
}

fn service_utils_js(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(storage_js.join("utils/dom.js"), "js/utils/dom.js"),
        AssetDefinition::js(storage_js.join("utils/format.js"), "js/utils/format.js"),
        AssetDefinition::js(storage_js.join("utils/form.js"), "js/utils/form.js"),
    ]
}

pub(super) fn admin_assets(storage_css: &Path, storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(storage_css.join("admin-bundle.css"), "css/admin-bundle.css"),
        AssetDefinition::js(storage_js.join("admin-bundle.js"), "js/admin-bundle.js"),
        AssetDefinition::js(
            storage_js.join("admin/sidebar-toggle.js"),
            "js/admin/sidebar-toggle.js",
        ),
        AssetDefinition::js(
            storage_js.join("admin/json-tree.js"),
            "js/admin/json-tree.js",
        ),
    ]
}
