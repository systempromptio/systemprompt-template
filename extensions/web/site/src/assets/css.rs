use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! css {
    ($p:expr, $name:literal) => {
        AssetDefinition::css($p.join($name), concat!("css/", $name))
    };
}

pub(super) fn css_assets(storage_css: &Path) -> Vec<AssetDefinition> {
    let mut v = core_css(storage_css);
    v.extend(homepage_css(storage_css));
    v.extend(blog_css(storage_css));
    v.extend(docs_css(storage_css));
    v.extend(paper_css(storage_css));
    v.extend(feature_page_css(storage_css));
    v.extend(syntax_css(storage_css));
    v.extend(feature_base_css(storage_css));
    v.extend(playbook_css(storage_css));
    v
}

fn core_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"core/variables.css"),
        css!(p,"core/fonts.css"),
        css!(p,"core/reset.css"),
        css!(p,"components/header-core.css"),
        css!(p,"components/header-dropdown.css"),
        css!(p,"components/footer.css"),
        css!(p,"components/mobile-menu.css"),
        css!(p,"components/cta-buttons.css"),
    ]
}

fn homepage_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"homepage-hero.css"),
        css!(p,"homepage-demos.css"),
        css!(p,"homepage-demo-terminal.css"),
        css!(p,"homepage-demo-responsive.css"),
        css!(p,"homepage-sections-titles.css"),
        css!(p,"homepage-sections-features.css"),
        css!(p,"homepage-sections-steps.css"),
        css!(p,"homepage-sections-comparison.css"),
        css!(p,"homepage-sections-technical.css"),
        css!(p,"homepage-sections-traits.css"),
        css!(p,"homepage-sections-faq.css"),
        css!(p,"homepage-features.css"),
        css!(p,"homepage-playbooks-section.css"),
        css!(p,"homepage-playbooks-featured.css"),
        css!(p,"homepage-playbooks-actions.css"),
        css!(p,"homepage-playbooks-categories.css"),
        css!(p,"homepage-playbooks-links.css"),
        css!(p,"homepage-playbooks-ctas.css"),
        css!(p,"homepage-playbooks-status.css"),
        css!(p,"homepage-playbooks-modal.css"),
        css!(p,"homepage-architecture.css"),
    ]
}

fn blog_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"blog-variables.css"),
        css!(p,"blog-background.css"),
        css!(p,"blog-base.css"),
        css!(p,"blog-post-header.css"),
        css!(p,"blog-social-bar.css"),
        css!(p,"blog-featured-image.css"),
        css!(p,"blog-post-content.css"),
        css!(p,"blog-breadcrumb.css"),
        css!(p,"blog-page-header.css"),
        css!(p,"blog-list-controls.css"),
        css!(p,"blog-cards.css"),
        css!(p,"blog-footer.css"),
        css!(p,"blog-references.css"),
        css!(p,"blog-related.css"),
        css!(p,"blog-banner.css"),
        css!(p,"blog-chat-cta.css"),
        css!(p,"blog-social-content.css"),
        css!(p,"blog-hero.css"),
        css!(p,"blog-homepage.css"),
        css!(p,"blog-platforms.css"),
        css!(p,"blog-ai-badges.css"),
        css!(p,"blog-content-sections.css"),
        css!(p,"blog-content-cards.css"),
        css!(p,"blog-provenance-panel.css"),
        css!(p,"blog-provenance-sections.css"),
        css!(p,"blog-provenance-header.css"),
        css!(p,"blog-workflow.css"),
        css!(p,"blog-provenance-details.css"),
        css!(p,"blog-live-dashboard.css"),
        css!(p,"blog-responsive.css"),
        css!(p,"blog-code.css"),
        css!(p,"blog-layout-structure.css"),
        css!(p,"blog-layout-cards.css"),
        css!(p,"blog-print.css"),
        css!(p,"blog-typography-base.css"),
        css!(p,"blog-typography-blocks.css"),
    ]
}

fn docs_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"docs-layout.css"),
        css!(p,"docs-header.css"),
        css!(p,"docs-content.css"),
        css!(p,"docs-pagination.css"),
        css!(p,"docs-toc.css"),
        css!(p,"docs-responsive.css"),
        css!(p,"docs-sidebar-links.css"),
    ]
}

fn paper_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"paper-layout.css"),
        css!(p,"paper-content.css"),
        css!(p,"paper-components.css"),
    ]
}

fn feature_page_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"feature-page-hero.css"),
        css!(p,"feature-page-content.css"),
        css!(p,"feature-page-responsive.css"),
    ]
}

fn feature_base_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"feature-base-hero.css"),
        css!(p,"feature-base-sections.css"),
        css!(p,"feature-base-cta.css"),
        css!(p,"feature-base-details.css"),
        css!(p,"content-cards-base.css"),
        css!(p,"content-cards-categories.css"),
        css!(p,"content-cards-list.css"),
    ]
}

fn playbook_css(p: &Path) -> Vec<AssetDefinition> {
    vec![
        css!(p,"playbook-layout.css"),
        css!(p,"playbook-list.css"),
        css!(p,"playbook-post.css"),
        css!(p,"playbook-content.css"),
        css!(p,"playbook-grid.css"),
        css!(p,"playbook-cards.css"),
    ]
}

fn syntax_css(p: &Path) -> Vec<AssetDefinition> {
    vec![css!(p,"syntax-highlight.css")]
}
