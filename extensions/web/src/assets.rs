use systemprompt::extension::AssetDefinition;

pub fn web_assets(paths: &dyn systemprompt::extension::AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");

    vec![
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
            storage_css.join("homepage-pricing.css"),
            "css/homepage-pricing.css",
        ),
        AssetDefinition::css(
            storage_css.join("homepage-playbooks.css"),
            "css/homepage-playbooks.css",
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
        AssetDefinition::css(
            storage_css.join("animation-memory-loop.css"),
            "css/animation-memory-loop.css",
        ),
        AssetDefinition::css(storage_css.join("feature-base.css"), "css/feature-base.css"),
        AssetDefinition::css(storage_css.join("feature-rust.css"), "css/feature-rust.css"),
        AssetDefinition::css(storage_css.join("feature-cli.css"), "css/feature-cli.css"),
        AssetDefinition::css(
            storage_css.join("feature-memory.css"),
            "css/feature-memory.css",
        ),
        AssetDefinition::css(
            storage_css.join("feature-closed-loop.css"),
            "css/feature-closed-loop.css",
        ),
        AssetDefinition::css(
            storage_css.join("feature-agentic-mesh.css"),
            "css/feature-agentic-mesh.css",
        ),
        AssetDefinition::css(
            storage_css.join("animation-cli-remote.css"),
            "css/animation-cli-remote.css",
        ),
        AssetDefinition::css(
            storage_css.join("content-cards.css"),
            "css/content-cards.css",
        ),
        AssetDefinition::css(storage_css.join("playbook.css"), "css/playbook.css"),
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("terminal-demo.js"), "js/terminal-demo.js"),
        AssetDefinition::js(storage_js.join("blog-images.js"), "js/blog-images.js"),
    ]
}
