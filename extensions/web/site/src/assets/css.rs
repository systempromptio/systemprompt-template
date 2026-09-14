//! Stylesheet definitions, grouped by the page family that consumes them.

use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! css {
    ($p:expr, $name:literal) => {
        AssetDefinition::css($p.join($name), concat!("css/", $name))
    };
}

#[doc(hidden)]
pub fn css_assets(storage_css: &Path) -> Vec<AssetDefinition> {
    let mut v = core_css(storage_css);
    v.extend(homepage_css(storage_css));
    v.extend(docs_css(storage_css));
    v.extend(syntax_css(storage_css));
    v.extend(skills_css(storage_css));
    v
}

fn core_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p, "core/variables.css"),
        css!(p, "core/fonts.css"),
        css!(p, "core/reset.css"),
        css!(p, "components/header-core.css"),
        css!(p, "components/header-dropdown.css"),
        css!(p, "components/footer.css"),
        css!(p, "components/mobile-menu.css"),
    ]
}

fn homepage_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p, "homepage-hero.css"),
        css!(p, "homepage-showreel.css"),
        css!(p, "homepage-getting-started.css"),
    ]
}

fn docs_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p, "docs-layout.css"),
        css!(p, "docs-header.css"),
        css!(p, "docs-content.css"),
        css!(p, "docs-evidence-gallery.css"),
        css!(p, "docs-pagination.css"),
        css!(p, "docs-toc.css"),
        css!(p, "docs-responsive.css"),
        css!(p, "docs-sidebar-links.css"),
    ]
}

fn syntax_css(p: &Path) -> Vec<AssetDefinition> {
    vec![css!(p, "syntax-highlight.css")]
}

fn skills_css(p: &Path) -> Vec<AssetDefinition> {
    vec![css!(p, "skills-page.css")]
}
