use std::path::Path;
use systemprompt::extension::AssetDefinition;

#[allow(clippy::too_many_lines)]
pub fn web_assets(paths: &dyn systemprompt::extension::AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");
    let storage_admin = paths.storage_files().join("admin").join("compiled");

    let mut assets = vec![
        AssetDefinition::css(
            storage_css.join("core/variables.css"),
            "css/core/variables.css",
        ),
        AssetDefinition::css(storage_css.join("core/fonts.css"), "css/core/fonts.css"),
        AssetDefinition::css(storage_css.join("core/reset.css"), "css/core/reset.css"),
        AssetDefinition::css(
            storage_css.join("components/header.css"),
            "css/components/header.css",
        ),
        AssetDefinition::css(
            storage_css.join("components/footer.css"),
            "css/components/footer.css",
        ),
        AssetDefinition::css(
            storage_css.join("components/mobile-menu.css"),
            "css/components/mobile-menu.css",
        ),
        AssetDefinition::css(
            storage_css.join("components/cta-buttons.css"),
            "css/components/cta-buttons.css",
        ),
        AssetDefinition::css(storage_css.join("homepage.css"), "css/homepage.css"),
        AssetDefinition::css(
            storage_css.join("homepage-hero.css"),
            "css/homepage-hero.css",
        ),
        AssetDefinition::css(
            storage_css.join("homepage-demo.css"),
            "css/homepage-demo.css",
        ),
        AssetDefinition::css(
            storage_css.join("homepage-sections.css"),
            "css/homepage-sections.css",
        ),
        AssetDefinition::css(
            storage_css.join("homepage-playbooks.css"),
            "css/homepage-playbooks.css",
        ),
        AssetDefinition::css(
            storage_css.join("homepage-architecture.css"),
            "css/homepage-architecture.css",
        ),
        AssetDefinition::css(storage_css.join("blog.css"), "css/blog.css"),
        AssetDefinition::css(storage_css.join("blog-code.css"), "css/blog-code.css"),
        AssetDefinition::css(storage_css.join("blog-layout.css"), "css/blog-layout.css"),
        AssetDefinition::css(storage_css.join("blog-print.css"), "css/blog-print.css"),
        AssetDefinition::css(
            storage_css.join("blog-typography.css"),
            "css/blog-typography.css",
        ),
        AssetDefinition::css(storage_css.join("docs.css"), "css/docs.css"),
        AssetDefinition::css(storage_css.join("paper.css"), "css/paper.css"),
        AssetDefinition::css(storage_css.join("feature-page.css"), "css/feature-page.css"),
        AssetDefinition::css(
            storage_css.join("syntax-highlight.css"),
            "css/syntax-highlight.css",
        ),
        AssetDefinition::css(storage_css.join("feature-base.css"), "css/feature-base.css"),
        AssetDefinition::css(
            storage_css.join("content-cards.css"),
            "css/content-cards.css",
        ),
        AssetDefinition::css(storage_css.join("playbook.css"), "css/playbook.css"),
        AssetDefinition::css(storage_css.join("presentation.css"), "css/presentation.css"),
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
    ];

    assets.push(AssetDefinition::css(
        storage_css.join("admin-bundle.css"),
        "css/admin-bundle.css",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-bundle.js"),
        "js/admin-bundle.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-core.js"),
        "js/admin-core.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-plugins.js"),
        "js/admin-plugins.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-users.js"),
        "js/admin-users.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-marketplace.js"),
        "js/admin-marketplace.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-org.js"),
        "js/admin-org.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-access.js"),
        "js/admin-access.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-audit.js"),
        "js/admin-audit.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-achievements.js"),
        "js/admin-achievements.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-my-workspace.js"),
        "js/admin-my-workspace.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin/login.js"),
        "js/admin/login.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin/dashboard-sse.js"),
        "js/admin/dashboard-sse.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin/sidebar-toggle.js"),
        "js/admin/sidebar-toggle.js",
    ));
    assets.extend(admin_html_assets(&storage_admin));

    assets
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
