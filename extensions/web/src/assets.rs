use std::path::Path;
use systemprompt::extension::AssetDefinition;

pub fn web_assets(paths: &dyn systemprompt::extension::AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");
    let storage_admin = paths.storage_files().join("admin").join("compiled");

    let mut assets = css_assets(&storage_css);
    assets.extend(public_js_assets(&storage_js));
    assets.extend(admin_assets(&storage_css, &storage_js));
    assets.extend(admin_html_assets(&storage_admin));
    assets
}

fn css_assets(storage_css: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(storage_css.join("core/variables.css"), "css/core/variables.css"),
        AssetDefinition::css(storage_css.join("core/fonts.css"), "css/core/fonts.css"),
        AssetDefinition::css(storage_css.join("core/reset.css"), "css/core/reset.css"),
        AssetDefinition::css(storage_css.join("components/header-core.css"), "css/components/header-core.css"),
        AssetDefinition::css(storage_css.join("components/header-dropdown.css"), "css/components/header-dropdown.css"),
        AssetDefinition::css(storage_css.join("components/footer.css"), "css/components/footer.css"),
        AssetDefinition::css(storage_css.join("components/mobile-menu.css"), "css/components/mobile-menu.css"),
        AssetDefinition::css(storage_css.join("components/cta-buttons.css"), "css/components/cta-buttons.css"),
        AssetDefinition::css(storage_css.join("homepage-hero.css"), "css/homepage-hero.css"),
        AssetDefinition::css(storage_css.join("homepage-demo-terminal.css"), "css/homepage-demo-terminal.css"),
        AssetDefinition::css(storage_css.join("homepage-demo-responsive.css"), "css/homepage-demo-responsive.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-titles.css"), "css/homepage-sections-titles.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-features.css"), "css/homepage-sections-features.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-steps.css"), "css/homepage-sections-steps.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-comparison.css"), "css/homepage-sections-comparison.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-technical.css"), "css/homepage-sections-technical.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-traits.css"), "css/homepage-sections-traits.css"),
        AssetDefinition::css(storage_css.join("homepage-sections-faq.css"), "css/homepage-sections-faq.css"),
        AssetDefinition::css(storage_css.join("homepage-features.css"), "css/homepage-features.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-section.css"), "css/homepage-playbooks-section.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-featured.css"), "css/homepage-playbooks-featured.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-actions.css"), "css/homepage-playbooks-actions.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-categories.css"), "css/homepage-playbooks-categories.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-links.css"), "css/homepage-playbooks-links.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-ctas.css"), "css/homepage-playbooks-ctas.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-status.css"), "css/homepage-playbooks-status.css"),
        AssetDefinition::css(storage_css.join("homepage-playbooks-modal.css"), "css/homepage-playbooks-modal.css"),
        AssetDefinition::css(storage_css.join("homepage-architecture.css"), "css/homepage-architecture.css"),
        AssetDefinition::css(storage_css.join("blog-variables.css"), "css/blog-variables.css"),
        AssetDefinition::css(storage_css.join("blog-background.css"), "css/blog-background.css"),
        AssetDefinition::css(storage_css.join("blog-base.css"), "css/blog-base.css"),
        AssetDefinition::css(storage_css.join("blog-post-header.css"), "css/blog-post-header.css"),
        AssetDefinition::css(storage_css.join("blog-social-bar.css"), "css/blog-social-bar.css"),
        AssetDefinition::css(storage_css.join("blog-featured-image.css"), "css/blog-featured-image.css"),
        AssetDefinition::css(storage_css.join("blog-post-content.css"), "css/blog-post-content.css"),
        AssetDefinition::css(storage_css.join("blog-breadcrumb.css"), "css/blog-breadcrumb.css"),
        AssetDefinition::css(storage_css.join("blog-page-header.css"), "css/blog-page-header.css"),
        AssetDefinition::css(storage_css.join("blog-list-controls.css"), "css/blog-list-controls.css"),
        AssetDefinition::css(storage_css.join("blog-cards.css"), "css/blog-cards.css"),
        AssetDefinition::css(storage_css.join("blog-footer.css"), "css/blog-footer.css"),
        AssetDefinition::css(storage_css.join("blog-references.css"), "css/blog-references.css"),
        AssetDefinition::css(storage_css.join("blog-related.css"), "css/blog-related.css"),
        AssetDefinition::css(storage_css.join("blog-banner.css"), "css/blog-banner.css"),
        AssetDefinition::css(storage_css.join("blog-chat-cta.css"), "css/blog-chat-cta.css"),
        AssetDefinition::css(storage_css.join("blog-social-content.css"), "css/blog-social-content.css"),
        AssetDefinition::css(storage_css.join("blog-hero.css"), "css/blog-hero.css"),
        AssetDefinition::css(storage_css.join("blog-homepage.css"), "css/blog-homepage.css"),
        AssetDefinition::css(storage_css.join("blog-platforms.css"), "css/blog-platforms.css"),
        AssetDefinition::css(storage_css.join("blog-ai-badges.css"), "css/blog-ai-badges.css"),
        AssetDefinition::css(storage_css.join("blog-content-sections.css"), "css/blog-content-sections.css"),
        AssetDefinition::css(storage_css.join("blog-content-cards.css"), "css/blog-content-cards.css"),
        AssetDefinition::css(storage_css.join("blog-provenance-panel.css"), "css/blog-provenance-panel.css"),
        AssetDefinition::css(storage_css.join("blog-provenance-sections.css"), "css/blog-provenance-sections.css"),
        AssetDefinition::css(storage_css.join("blog-provenance-header.css"), "css/blog-provenance-header.css"),
        AssetDefinition::css(storage_css.join("blog-workflow.css"), "css/blog-workflow.css"),
        AssetDefinition::css(storage_css.join("blog-provenance-details.css"), "css/blog-provenance-details.css"),
        AssetDefinition::css(storage_css.join("blog-live-dashboard.css"), "css/blog-live-dashboard.css"),
        AssetDefinition::css(storage_css.join("blog-responsive.css"), "css/blog-responsive.css"),
        AssetDefinition::css(storage_css.join("blog-code.css"), "css/blog-code.css"),
        AssetDefinition::css(storage_css.join("blog-layout-structure.css"), "css/blog-layout-structure.css"),
        AssetDefinition::css(storage_css.join("blog-layout-cards.css"), "css/blog-layout-cards.css"),
        AssetDefinition::css(storage_css.join("blog-print.css"), "css/blog-print.css"),
        AssetDefinition::css(storage_css.join("blog-typography-base.css"), "css/blog-typography-base.css"),
        AssetDefinition::css(storage_css.join("blog-typography-blocks.css"), "css/blog-typography-blocks.css"),
        AssetDefinition::css(storage_css.join("docs-layout.css"), "css/docs-layout.css"),
        AssetDefinition::css(storage_css.join("docs-header.css"), "css/docs-header.css"),
        AssetDefinition::css(storage_css.join("docs-content.css"), "css/docs-content.css"),
        AssetDefinition::css(storage_css.join("docs-pagination.css"), "css/docs-pagination.css"),
        AssetDefinition::css(storage_css.join("docs-toc.css"), "css/docs-toc.css"),
        AssetDefinition::css(storage_css.join("docs-responsive.css"), "css/docs-responsive.css"),
        AssetDefinition::css(storage_css.join("paper-layout.css"), "css/paper-layout.css"),
        AssetDefinition::css(storage_css.join("paper-content.css"), "css/paper-content.css"),
        AssetDefinition::css(storage_css.join("paper-components.css"), "css/paper-components.css"),
        AssetDefinition::css(storage_css.join("feature-page-hero.css"), "css/feature-page-hero.css"),
        AssetDefinition::css(storage_css.join("feature-page-content.css"), "css/feature-page-content.css"),
        AssetDefinition::css(storage_css.join("feature-page-responsive.css"), "css/feature-page-responsive.css"),
        AssetDefinition::css(storage_css.join("syntax-highlight.css"), "css/syntax-highlight.css"),
        AssetDefinition::css(storage_css.join("feature-base-hero.css"), "css/feature-base-hero.css"),
        AssetDefinition::css(storage_css.join("feature-base-sections.css"), "css/feature-base-sections.css"),
        AssetDefinition::css(storage_css.join("feature-base-cta.css"), "css/feature-base-cta.css"),
        AssetDefinition::css(storage_css.join("feature-base-details.css"), "css/feature-base-details.css"),
        AssetDefinition::css(storage_css.join("content-cards-base.css"), "css/content-cards-base.css"),
        AssetDefinition::css(storage_css.join("content-cards-categories.css"), "css/content-cards-categories.css"),
        AssetDefinition::css(storage_css.join("content-cards-list.css"), "css/content-cards-list.css"),
        AssetDefinition::css(storage_css.join("playbook-layout.css"), "css/playbook-layout.css"),
        AssetDefinition::css(storage_css.join("playbook-list.css"), "css/playbook-list.css"),
        AssetDefinition::css(storage_css.join("playbook-post.css"), "css/playbook-post.css"),
        AssetDefinition::css(storage_css.join("playbook-content.css"), "css/playbook-content.css"),
        AssetDefinition::css(storage_css.join("playbook-grid.css"), "css/playbook-grid.css"),
        AssetDefinition::css(storage_css.join("playbook-cards.css"), "css/playbook-cards.css"),
        AssetDefinition::css(storage_css.join("presentation.css"), "css/presentation.css"),
    ]
}

fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("terminal-demo.js"), "js/terminal-demo.js"),
        AssetDefinition::js(storage_js.join("blog-images.js"), "js/blog-images.js"),
        AssetDefinition::js(storage_js.join("homepage.js"), "js/homepage.js"),
        AssetDefinition::js(storage_js.join("presentation-nav.js"), "js/presentation-nav.js"),
    ]
}

fn admin_assets(storage_css: &Path, storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::css(storage_css.join("admin-bundle.css"), "css/admin-bundle.css"),
        AssetDefinition::js(storage_js.join("admin-bundle.js"), "js/admin-bundle.js"),
        AssetDefinition::js(storage_js.join("admin-core.js"), "js/admin-core.js"),
        AssetDefinition::js(storage_js.join("admin-plugins.js"), "js/admin-plugins.js"),
        AssetDefinition::js(storage_js.join("admin-users.js"), "js/admin-users.js"),
        AssetDefinition::js(storage_js.join("admin-marketplace.js"), "js/admin-marketplace.js"),
        AssetDefinition::js(storage_js.join("admin-org.js"), "js/admin-org.js"),
        AssetDefinition::js(storage_js.join("admin-access.js"), "js/admin-access.js"),
        AssetDefinition::js(storage_js.join("admin-audit.js"), "js/admin-audit.js"),
        AssetDefinition::js(storage_js.join("admin-achievements.js"), "js/admin-achievements.js"),
        AssetDefinition::js(storage_js.join("admin-my-workspace.js"), "js/admin-my-workspace.js"),
        AssetDefinition::js(storage_js.join("admin/login.js"), "js/admin/login.js"),
        AssetDefinition::js(storage_js.join("admin/dashboard-sse.js"), "js/admin/dashboard-sse.js"),
        AssetDefinition::js(storage_js.join("admin/sidebar-toggle.js"), "js/admin/sidebar-toggle.js"),
    ]
}

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
