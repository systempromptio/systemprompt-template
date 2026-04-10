use std::path::Path;
use systemprompt::extension::AssetDefinition;

pub(super) fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("terminal-demo.js"), "js/terminal-demo.js"),
        AssetDefinition::js(storage_js.join("blog-images.js"), "js/blog-images.js"),
        AssetDefinition::js(storage_js.join("homepage.js"), "js/homepage.js"),
        AssetDefinition::js(
            storage_js.join("presentation-nav.js"),
            "js/presentation-nav.js",
        ),
    ]
}

pub(super) fn service_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let p = storage_js.join("services");
    let mut v = service_core_js(&p);
    v.extend(service_cc_js(&p));
    v.extend(service_control_center_js(&p));
    v.extend(service_entity_js(&p));
    v.extend(service_plugin_js(&p));
    v.extend(service_skill_js(&p));
    v.extend(service_webauthn_js(&p));
    v.extend(service_utils_js(storage_js));
    v
}

fn service_core_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(p.join("api.js"), "js/services/api.js"),
        AssetDefinition::js(p.join("auth.js"), "js/services/auth.js"),
        AssetDefinition::js(p.join("bootstrap.js"), "js/services/bootstrap.js"),
        AssetDefinition::js(p.join("confirm.js"), "js/services/confirm.js"),
        AssetDefinition::js(p.join("dropdown.js"), "js/services/dropdown.js"),
        AssetDefinition::js(p.join("events.js"), "js/services/events.js"),
        AssetDefinition::js(p.join("header-actions.js"), "js/services/header-actions.js"),
        AssetDefinition::js(p.join("install-widget.js"), "js/services/install-widget.js"),
        AssetDefinition::js(
            p.join("onboarding-banner.js"),
            "js/services/onboarding-banner.js",
        ),
        AssetDefinition::js(p.join("sidebar.js"), "js/services/sidebar.js"),
        AssetDefinition::js(p.join("sse-client.js"), "js/services/sse-client.js"),
        AssetDefinition::js(p.join("table-sort.js"), "js/services/table-sort.js"),
        AssetDefinition::js(p.join("theme.js"), "js/services/theme.js"),
        AssetDefinition::js(p.join("sp-toast.js"), "js/services/sp-toast.js"),
        AssetDefinition::js(p.join("toast.js"), "js/services/toast.js"),
        AssetDefinition::js(p.join("toc-highlight.js"), "js/services/toc-highlight.js"),
    ]
}

fn service_cc_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            p.join("cc-cards-helpers.js"),
            "js/services/cc-cards-helpers.js",
        ),
        AssetDefinition::js(
            p.join("cc-cards-render-metrics.js"),
            "js/services/cc-cards-render-metrics.js",
        ),
        AssetDefinition::js(
            p.join("cc-cards-render-sections.js"),
            "js/services/cc-cards-render-sections.js",
        ),
        AssetDefinition::js(
            p.join("cc-cards-render-session.js"),
            "js/services/cc-cards-render-session.js",
        ),
        AssetDefinition::js(
            p.join("cc-charts-render.js"),
            "js/services/cc-charts-render.js",
        ),
        AssetDefinition::js(
            p.join("cc-charts-setup.js"),
            "js/services/cc-charts-setup.js",
        ),
        AssetDefinition::js(
            p.join("cc-charts-tooltip.js"),
            "js/services/cc-charts-tooltip.js",
        ),
        AssetDefinition::js(
            p.join("cc-feed-reorder.js"),
            "js/services/cc-feed-reorder.js",
        ),
        AssetDefinition::js(p.join("cc-stats-api.js"), "js/services/cc-stats-api.js"),
        AssetDefinition::js(
            p.join("cc-stats-charts.js"),
            "js/services/cc-stats-charts.js",
        ),
        AssetDefinition::js(p.join("cc-stats-daily.js"), "js/services/cc-stats-daily.js"),
        AssetDefinition::js(
            p.join("cc-stats-header.js"),
            "js/services/cc-stats-header.js",
        ),
        AssetDefinition::js(
            p.join("cc-stats-suggestions.js"),
            "js/services/cc-stats-suggestions.js",
        ),
        AssetDefinition::js(p.join("cc-stats-ui.js"), "js/services/cc-stats-ui.js"),
    ]
}

fn service_control_center_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            p.join("control-center-batch.js"),
            "js/services/control-center-batch.js",
        ),
        AssetDefinition::js(
            p.join("control-center-cards-render.js"),
            "js/services/control-center-cards-render.js",
        ),
        AssetDefinition::js(
            p.join("control-center-cards.js"),
            "js/services/control-center-cards.js",
        ),
        AssetDefinition::js(
            p.join("control-center-feed.js"),
            "js/services/control-center-feed.js",
        ),
        AssetDefinition::js(
            p.join("control-center-limits.js"),
            "js/services/control-center-limits.js",
        ),
        AssetDefinition::js(
            p.join("control-center-stats.js"),
            "js/services/control-center-stats.js",
        ),
        AssetDefinition::js(
            p.join("control-center-turns.js"),
            "js/services/control-center-turns.js",
        ),
    ]
}

fn service_entity_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            p.join("engagement-state.js"),
            "js/services/engagement-state.js",
        ),
        AssetDefinition::js(
            p.join("engagement-tracker.js"),
            "js/services/engagement-tracker.js",
        ),
        AssetDefinition::js(
            p.join("entity-batch-ops.js"),
            "js/services/entity-batch-ops.js",
        ),
        AssetDefinition::js(p.join("entity-batch.js"), "js/services/entity-batch.js"),
        AssetDefinition::js(p.join("entity-common.js"), "js/services/entity-common.js"),
        AssetDefinition::js(p.join("list-page.js"), "js/services/list-page.js"),
    ]
}

fn service_plugin_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            p.join("plugin-details-ui.js"),
            "js/services/plugin-details-ui.js",
        ),
        AssetDefinition::js(p.join("plugin-details.js"), "js/services/plugin-details.js"),
        AssetDefinition::js(p.join("plugin-env-ui.js"), "js/services/plugin-env-ui.js"),
        AssetDefinition::js(p.join("plugin-env.js"), "js/services/plugin-env.js"),
        AssetDefinition::js(
            p.join("plugin-resources-helpers.js"),
            "js/services/plugin-resources-helpers.js",
        ),
        AssetDefinition::js(
            p.join("plugin-resources.js"),
            "js/services/plugin-resources.js",
        ),
    ]
}

fn service_skill_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            p.join("skill-files-editor.js"),
            "js/services/skill-files-editor.js",
        ),
        AssetDefinition::js(
            p.join("skill-files-modal.js"),
            "js/services/skill-files-modal.js",
        ),
        AssetDefinition::js(p.join("skill-files.js"), "js/services/skill-files.js"),
    ]
}

fn service_webauthn_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            p.join("webauthn-helpers.js"),
            "js/services/webauthn-helpers.js",
        ),
        AssetDefinition::js(p.join("webauthn-login.js"), "js/services/webauthn-login.js"),
        AssetDefinition::js(
            p.join("webauthn-login-ui.js"),
            "js/services/webauthn-login-ui.js",
        ),
        AssetDefinition::js(
            p.join("webauthn-passkey.js"),
            "js/services/webauthn-passkey.js",
        ),
        AssetDefinition::js(
            p.join("webauthn-passkey-helpers.js"),
            "js/services/webauthn-passkey-helpers.js",
        ),
        AssetDefinition::js(p.join("webauthn-utils.js"), "js/services/webauthn-utils.js"),
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
        AssetDefinition::js(storage_js.join("admin-core.js"), "js/admin-core.js"),
        AssetDefinition::js(storage_js.join("admin-plugins.js"), "js/admin-plugins.js"),
        AssetDefinition::js(storage_js.join("admin-users.js"), "js/admin-users.js"),
        AssetDefinition::js(
            storage_js.join("admin-marketplace.js"),
            "js/admin-marketplace.js",
        ),
        AssetDefinition::js(storage_js.join("admin-org.js"), "js/admin-org.js"),
        AssetDefinition::js(storage_js.join("admin-access.js"), "js/admin-access.js"),
        AssetDefinition::js(storage_js.join("admin-audit.js"), "js/admin-audit.js"),
        AssetDefinition::js(
            storage_js.join("admin-achievements.js"),
            "js/admin-achievements.js",
        ),
        AssetDefinition::js(
            storage_js.join("admin-my-workspace.js"),
            "js/admin-my-workspace.js",
        ),
        AssetDefinition::js(storage_js.join("admin/login.js"), "js/admin/login.js"),
        AssetDefinition::js(
            storage_js.join("admin/dashboard-sse.js"),
            "js/admin/dashboard-sse.js",
        ),
        AssetDefinition::js(
            storage_js.join("admin/sidebar-toggle.js"),
            "js/admin/sidebar-toggle.js",
        ),
    ]
}

pub(super) fn admin_html_assets(compiled_dir: &Path) -> Vec<AssetDefinition> {
    let admin = compiled_dir.join("admin");
    let presentation = compiled_dir.join("presentation");

    vec![
        AssetDefinition::html(
            admin.join("login").join("index.html"),
            "admin/login/index.html",
        ),
        AssetDefinition::html(
            presentation.join("index.html"),
            "documentation/presentation/index.html",
        ),
    ]
}
