use std::path::Path;
use systemprompt::extension::AssetDefinition;

pub fn web_assets(paths: &dyn systemprompt::extension::AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");
    let storage_admin = paths.storage_files().join("admin").join("compiled");

    let mut assets = css_assets(&storage_css);
    assets.extend(public_js_assets(&storage_js));
    assets.extend(service_js_assets(&storage_js));
    assets.extend(admin_assets(&storage_css, &storage_js));
    assets.extend(page_js_assets(&storage_js));
    assets.extend(admin_html_assets(&storage_admin));
    assets
}

// ---------------------------------------------------------------------------
// CSS assets — split by logical prefix group
// ---------------------------------------------------------------------------

fn css_assets(storage_css: &Path) -> Vec<AssetDefinition> {
    let mut v = core_css(storage_css);
    v.extend(homepage_css(storage_css));
    v.extend(blog_css(storage_css));
    v.extend(docs_css(storage_css));
    v.extend(paper_css(storage_css));
    v.extend(feature_page_css(storage_css));
    v.extend(syntax_css(storage_css));
    v.extend(feature_base_css(storage_css));
    v.extend(playbook_css(storage_css));
    v.extend(presentation_css(storage_css));
    v
}

fn core_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("core/variables.css"), "css/core/variables.css"),
        AssetDefinition::css(p.join("core/fonts.css"), "css/core/fonts.css"),
        AssetDefinition::css(p.join("core/reset.css"), "css/core/reset.css"),
        AssetDefinition::css(
            p.join("components/header-core.css"),
            "css/components/header-core.css",
        ),
        AssetDefinition::css(
            p.join("components/header-dropdown.css"),
            "css/components/header-dropdown.css",
        ),
        AssetDefinition::css(p.join("components/footer.css"), "css/components/footer.css"),
        AssetDefinition::css(
            p.join("components/mobile-menu.css"),
            "css/components/mobile-menu.css",
        ),
        AssetDefinition::css(
            p.join("components/cta-buttons.css"),
            "css/components/cta-buttons.css",
        ),
    ]
}

fn homepage_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("homepage-hero.css"), "css/homepage-hero.css"),
        AssetDefinition::css(
            p.join("homepage-demo-terminal.css"),
            "css/homepage-demo-terminal.css",
        ),
        AssetDefinition::css(
            p.join("homepage-demo-responsive.css"),
            "css/homepage-demo-responsive.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-titles.css"),
            "css/homepage-sections-titles.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-features.css"),
            "css/homepage-sections-features.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-steps.css"),
            "css/homepage-sections-steps.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-comparison.css"),
            "css/homepage-sections-comparison.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-technical.css"),
            "css/homepage-sections-technical.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-traits.css"),
            "css/homepage-sections-traits.css",
        ),
        AssetDefinition::css(
            p.join("homepage-sections-faq.css"),
            "css/homepage-sections-faq.css",
        ),
        AssetDefinition::css(p.join("homepage-features.css"), "css/homepage-features.css"),
        AssetDefinition::css(
            p.join("homepage-playbooks-section.css"),
            "css/homepage-playbooks-section.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-featured.css"),
            "css/homepage-playbooks-featured.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-actions.css"),
            "css/homepage-playbooks-actions.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-categories.css"),
            "css/homepage-playbooks-categories.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-links.css"),
            "css/homepage-playbooks-links.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-ctas.css"),
            "css/homepage-playbooks-ctas.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-status.css"),
            "css/homepage-playbooks-status.css",
        ),
        AssetDefinition::css(
            p.join("homepage-playbooks-modal.css"),
            "css/homepage-playbooks-modal.css",
        ),
        AssetDefinition::css(
            p.join("homepage-architecture.css"),
            "css/homepage-architecture.css",
        ),
    ]
}

fn blog_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("blog-variables.css"), "css/blog-variables.css"),
        AssetDefinition::css(p.join("blog-background.css"), "css/blog-background.css"),
        AssetDefinition::css(p.join("blog-base.css"), "css/blog-base.css"),
        AssetDefinition::css(p.join("blog-post-header.css"), "css/blog-post-header.css"),
        AssetDefinition::css(p.join("blog-social-bar.css"), "css/blog-social-bar.css"),
        AssetDefinition::css(
            p.join("blog-featured-image.css"),
            "css/blog-featured-image.css",
        ),
        AssetDefinition::css(p.join("blog-post-content.css"), "css/blog-post-content.css"),
        AssetDefinition::css(p.join("blog-breadcrumb.css"), "css/blog-breadcrumb.css"),
        AssetDefinition::css(p.join("blog-page-header.css"), "css/blog-page-header.css"),
        AssetDefinition::css(
            p.join("blog-list-controls.css"),
            "css/blog-list-controls.css",
        ),
        AssetDefinition::css(p.join("blog-cards.css"), "css/blog-cards.css"),
        AssetDefinition::css(p.join("blog-footer.css"), "css/blog-footer.css"),
        AssetDefinition::css(p.join("blog-references.css"), "css/blog-references.css"),
        AssetDefinition::css(p.join("blog-related.css"), "css/blog-related.css"),
        AssetDefinition::css(p.join("blog-banner.css"), "css/blog-banner.css"),
        AssetDefinition::css(p.join("blog-chat-cta.css"), "css/blog-chat-cta.css"),
        AssetDefinition::css(
            p.join("blog-social-content.css"),
            "css/blog-social-content.css",
        ),
        AssetDefinition::css(p.join("blog-hero.css"), "css/blog-hero.css"),
        AssetDefinition::css(p.join("blog-homepage.css"), "css/blog-homepage.css"),
        AssetDefinition::css(p.join("blog-platforms.css"), "css/blog-platforms.css"),
        AssetDefinition::css(p.join("blog-ai-badges.css"), "css/blog-ai-badges.css"),
        AssetDefinition::css(
            p.join("blog-content-sections.css"),
            "css/blog-content-sections.css",
        ),
        AssetDefinition::css(
            p.join("blog-content-cards.css"),
            "css/blog-content-cards.css",
        ),
        AssetDefinition::css(
            p.join("blog-provenance-panel.css"),
            "css/blog-provenance-panel.css",
        ),
        AssetDefinition::css(
            p.join("blog-provenance-sections.css"),
            "css/blog-provenance-sections.css",
        ),
        AssetDefinition::css(
            p.join("blog-provenance-header.css"),
            "css/blog-provenance-header.css",
        ),
        AssetDefinition::css(p.join("blog-workflow.css"), "css/blog-workflow.css"),
        AssetDefinition::css(
            p.join("blog-provenance-details.css"),
            "css/blog-provenance-details.css",
        ),
        AssetDefinition::css(
            p.join("blog-live-dashboard.css"),
            "css/blog-live-dashboard.css",
        ),
        AssetDefinition::css(p.join("blog-responsive.css"), "css/blog-responsive.css"),
        AssetDefinition::css(p.join("blog-code.css"), "css/blog-code.css"),
        AssetDefinition::css(
            p.join("blog-layout-structure.css"),
            "css/blog-layout-structure.css",
        ),
        AssetDefinition::css(p.join("blog-layout-cards.css"), "css/blog-layout-cards.css"),
        AssetDefinition::css(p.join("blog-print.css"), "css/blog-print.css"),
        AssetDefinition::css(
            p.join("blog-typography-base.css"),
            "css/blog-typography-base.css",
        ),
        AssetDefinition::css(
            p.join("blog-typography-blocks.css"),
            "css/blog-typography-blocks.css",
        ),
    ]
}

fn docs_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("docs-layout.css"), "css/docs-layout.css"),
        AssetDefinition::css(p.join("docs-header.css"), "css/docs-header.css"),
        AssetDefinition::css(p.join("docs-content.css"), "css/docs-content.css"),
        AssetDefinition::css(p.join("docs-pagination.css"), "css/docs-pagination.css"),
        AssetDefinition::css(p.join("docs-toc.css"), "css/docs-toc.css"),
        AssetDefinition::css(p.join("docs-responsive.css"), "css/docs-responsive.css"),
        AssetDefinition::css(
            p.join("docs-sidebar-links.css"),
            "css/docs-sidebar-links.css",
        ),
    ]
}

fn paper_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("paper-layout.css"), "css/paper-layout.css"),
        AssetDefinition::css(p.join("paper-content.css"), "css/paper-content.css"),
        AssetDefinition::css(p.join("paper-components.css"), "css/paper-components.css"),
    ]
}

fn feature_page_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("feature-page-hero.css"), "css/feature-page-hero.css"),
        AssetDefinition::css(
            p.join("feature-page-content.css"),
            "css/feature-page-content.css",
        ),
        AssetDefinition::css(
            p.join("feature-page-responsive.css"),
            "css/feature-page-responsive.css",
        ),
    ]
}

fn feature_base_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("feature-base-hero.css"), "css/feature-base-hero.css"),
        AssetDefinition::css(
            p.join("feature-base-sections.css"),
            "css/feature-base-sections.css",
        ),
        AssetDefinition::css(p.join("feature-base-cta.css"), "css/feature-base-cta.css"),
        AssetDefinition::css(
            p.join("feature-base-details.css"),
            "css/feature-base-details.css",
        ),
        AssetDefinition::css(
            p.join("content-cards-base.css"),
            "css/content-cards-base.css",
        ),
        AssetDefinition::css(
            p.join("content-cards-categories.css"),
            "css/content-cards-categories.css",
        ),
        AssetDefinition::css(
            p.join("content-cards-list.css"),
            "css/content-cards-list.css",
        ),
    ]
}

fn playbook_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(p.join("playbook-layout.css"), "css/playbook-layout.css"),
        AssetDefinition::css(p.join("playbook-list.css"), "css/playbook-list.css"),
        AssetDefinition::css(p.join("playbook-post.css"), "css/playbook-post.css"),
        AssetDefinition::css(p.join("playbook-content.css"), "css/playbook-content.css"),
        AssetDefinition::css(p.join("playbook-grid.css"), "css/playbook-grid.css"),
        AssetDefinition::css(p.join("playbook-cards.css"), "css/playbook-cards.css"),
    ]
}

fn presentation_css(p: &Path) -> Vec<AssetDefinition> {
    vec![AssetDefinition::css(
        p.join("presentation.css"),
        "css/presentation.css",
    )]
}

fn syntax_css(p: &Path) -> Vec<AssetDefinition> {
    vec![AssetDefinition::css(
        p.join("syntax-highlight.css"),
        "css/syntax-highlight.css",
    )]
}

// ---------------------------------------------------------------------------
// Public JS assets
// ---------------------------------------------------------------------------

fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
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

// ---------------------------------------------------------------------------
// Service JS assets — split by logical prefix group
// ---------------------------------------------------------------------------

fn service_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
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

// ---------------------------------------------------------------------------
// Admin assets
// ---------------------------------------------------------------------------

fn admin_assets(storage_css: &Path, storage_js: &Path) -> Vec<AssetDefinition> {
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

// ---------------------------------------------------------------------------
// Page JS assets — split by logical prefix group
// ---------------------------------------------------------------------------

fn page_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let pages = storage_js.join("pages");
    let mut v = page_admin_core_js(&pages);
    v.extend(page_admin_cc_js(&pages));
    v.extend(page_admin_marketplace_js(&pages));
    v.extend(page_admin_my_js(&pages));
    v.extend(page_admin_org_js(&pages));
    v.extend(page_admin_plugin_js(&pages));
    v
}

fn page_admin_core_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(pages.join("admin-access.js"), "js/pages/admin-access.js"),
        AssetDefinition::js(
            pages.join("admin-access-bulk.js"),
            "js/pages/admin-access-bulk.js",
        ),
        AssetDefinition::js(
            pages.join("admin-access-panel.js"),
            "js/pages/admin-access-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-achievements.js"),
            "js/pages/admin-achievements.js",
        ),
        AssetDefinition::js(
            pages.join("admin-agent-edit.js"),
            "js/pages/admin-agent-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-agents-helpers.js"),
            "js/pages/admin-agents-helpers.js",
        ),
        AssetDefinition::js(pages.join("admin-audit.js"), "js/pages/admin-audit.js"),
        AssetDefinition::js(pages.join("admin-billing.js"), "js/pages/admin-billing.js"),
        AssetDefinition::js(
            pages.join("admin-dashboard-charts.js"),
            "js/pages/admin-dashboard-charts.js",
        ),
        AssetDefinition::js(
            pages.join("admin-dashboard.js"),
            "js/pages/admin-dashboard.js",
        ),
        AssetDefinition::js(pages.join("admin-events.js"), "js/pages/admin-events.js"),
        AssetDefinition::js(pages.join("admin-export.js"), "js/pages/admin-export.js"),
        AssetDefinition::js(pages.join("admin-jobs.js"), "js/pages/admin-jobs.js"),
        AssetDefinition::js(
            pages.join("admin-leaderboard.js"),
            "js/pages/admin-leaderboard.js",
        ),
        AssetDefinition::js(pages.join("admin-profile.js"), "js/pages/admin-profile.js"),
        AssetDefinition::js(
            pages.join("admin-settings.js"),
            "js/pages/admin-settings.js",
        ),
        AssetDefinition::js(
            pages.join("admin-skill-edit.js"),
            "js/pages/admin-skill-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-users-actions.js"),
            "js/pages/admin-users-actions.js",
        ),
        AssetDefinition::js(pages.join("admin-users.js"), "js/pages/admin-users.js"),
    ]
}

fn page_admin_cc_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-cc-report-build.js"),
            "js/pages/admin-cc-report-build.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-actions.js"),
            "js/pages/admin-control-center-actions.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-panels.js"),
            "js/pages/admin-control-center-panels.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-report.js"),
            "js/pages/admin-control-center-report.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center-sse.js"),
            "js/pages/admin-control-center-sse.js",
        ),
        AssetDefinition::js(
            pages.join("admin-control-center.js"),
            "js/pages/admin-control-center.js",
        ),
    ]
}

fn page_admin_marketplace_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-browse-plugins.js"),
            "js/pages/admin-browse-plugins.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-browse-panel-helpers.js"),
            "js/pages/admin-marketplace-browse-panel-helpers.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-browse-panel.js"),
            "js/pages/admin-marketplace-browse-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-browse.js"),
            "js/pages/admin-marketplace-browse.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-versions-panel-helpers.js"),
            "js/pages/admin-marketplace-versions-panel-helpers.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-versions-panel.js"),
            "js/pages/admin-marketplace-versions-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-marketplace-versions.js"),
            "js/pages/admin-marketplace-versions.js",
        ),
    ]
}

fn page_admin_my_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-my-activity.js"),
            "js/pages/admin-my-activity.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-agent-edit.js"),
            "js/pages/admin-my-agent-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-agents.js"),
            "js/pages/admin-my-agents.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-hooks.js"),
            "js/pages/admin-my-hooks.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-marketplace.js"),
            "js/pages/admin-my-marketplace.js",
        ),
        AssetDefinition::js(pages.join("admin-my-mcp.js"), "js/pages/admin-my-mcp.js"),
        AssetDefinition::js(
            pages.join("admin-my-plugin-edit.js"),
            "js/pages/admin-my-plugin-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-plugins.js"),
            "js/pages/admin-my-plugins.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-secrets.js"),
            "js/pages/admin-my-secrets.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-skill-edit.js"),
            "js/pages/admin-my-skill-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-my-skills.js"),
            "js/pages/admin-my-skills.js",
        ),
    ]
}

fn page_admin_org_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-org-access-panel.js"),
            "js/pages/admin-org-access-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-access.js"),
            "js/pages/admin-org-access.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents-events-helpers.js"),
            "js/pages/admin-org-agents-events-helpers.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents-events.js"),
            "js/pages/admin-org-agents-events.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents-panel.js"),
            "js/pages/admin-org-agents-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-agents.js"),
            "js/pages/admin-org-agents.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-hooks.js"),
            "js/pages/admin-org-hooks.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-marketplaces-panel.js"),
            "js/pages/admin-org-marketplaces-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-marketplaces.js"),
            "js/pages/admin-org-marketplaces.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-mcp-panel.js"),
            "js/pages/admin-org-mcp-panel.js",
        ),
        AssetDefinition::js(pages.join("admin-org-mcp.js"), "js/pages/admin-org-mcp.js"),
        AssetDefinition::js(
            pages.join("admin-org-skills-events.js"),
            "js/pages/admin-org-skills-events.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-skills-panel.js"),
            "js/pages/admin-org-skills-panel.js",
        ),
        AssetDefinition::js(
            pages.join("admin-org-skills.js"),
            "js/pages/admin-org-skills.js",
        ),
    ]
}

fn page_admin_plugin_js(pages: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            pages.join("admin-plugin-edit.js"),
            "js/pages/admin-plugin-edit.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugin-wizard-review.js"),
            "js/pages/admin-plugin-wizard-review.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugin-wizard-steps.js"),
            "js/pages/admin-plugin-wizard-steps.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugin-wizard.js"),
            "js/pages/admin-plugin-wizard.js",
        ),
        AssetDefinition::js(
            pages.join("admin-plugins-list.js"),
            "js/pages/admin-plugins-list.js",
        ),
    ]
}

// ---------------------------------------------------------------------------
// Admin HTML assets
// ---------------------------------------------------------------------------

fn admin_html_assets(compiled_dir: &Path) -> Vec<AssetDefinition> {
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
