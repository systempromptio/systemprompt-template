//! The public site is configured from YAML under `services/web/config/`, and
//! every optional key carries a `#[serde(default)]`. That is what lets an
//! operator write a minimal `navigation.yaml` or `homepage.yaml` without the
//! whole section failing to load. These tests hold the minimum viable document
//! for each config to exactly what the struct actually requires — if a field
//! loses its default, the corresponding minimal snippet stops deserialising.

use systemprompt_web_site::homepage::HomepageConfig;
use systemprompt_web_site::navigation::NavigationConfig;

#[test]
fn navigation_needs_only_a_header_with_items() {
    let config: NavigationConfig = serde_yaml::from_str(
        r"
header:
  items:
    - id: docs
      label: Docs
      href: /docs
",
    )
    .expect("minimal navigation.yaml");

    assert_eq!(config.header.items.len(), 1);
    assert!(config.header.cta.is_none());
    assert!(config.social.is_empty());
    assert!(config.docs_sidebar.is_empty());
    assert!(config.footer.legal.is_empty());

    let item = &config.header.items[0];
    assert!(!item.dropdown);
    assert!(!item.external);
    assert!(item.sections.is_empty());
    assert!(item.view_all.is_none());
}

#[test]
fn navigation_reads_a_dropdown_with_its_sections_and_view_all_link() {
    let config: NavigationConfig = serde_yaml::from_str(
        r"
header:
  items:
    - id: product
      label: Product
      href: /product
      dropdown: true
      sections:
        - title: Governance
          links:
            - label: Audit
              href: /product/audit
              description: Every decision, recorded
      view_all:
        label: See all
        href: /product
  cta:
    label: Book a demo
    href: /contact
    external: true
",
    )
    .expect("navigation.yaml with a dropdown");

    let item = &config.header.items[0];
    assert!(item.dropdown);
    assert_eq!(item.sections[0].title.as_deref(), Some("Governance"));
    assert_eq!(
        item.sections[0].links[0].description.as_deref(),
        Some("Every decision, recorded")
    );
    assert!(!item.sections[0].links[0].external);
    assert_eq!(item.view_all.as_ref().expect("view_all").href, "/product");
    assert!(config.header.cta.as_ref().expect("cta").external);
}

#[test]
fn navigation_rejects_a_header_without_items() {
    assert!(serde_yaml::from_str::<NavigationConfig>("header: {}\n").is_err());
    assert!(serde_yaml::from_str::<NavigationConfig>("social: []\n").is_err());
}

#[test]
fn an_empty_homepage_document_yields_every_section_absent() {
    let config: HomepageConfig = serde_yaml::from_str("{}").expect("empty homepage.yaml");

    assert!(config.hero.is_none());
    assert!(config.value_props.is_empty());
    assert!(config.integrations.is_none());
    assert!(config.how_it_works.is_none());
    assert!(config.use_cases.is_none());
    assert!(config.pricing.is_none());
    assert!(config.faq.is_none());
    assert!(config.final_cta.is_none());
}
