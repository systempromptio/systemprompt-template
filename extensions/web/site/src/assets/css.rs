use std::path::Path;
use systemprompt::extension::AssetDefinition;

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
