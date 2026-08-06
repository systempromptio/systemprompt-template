//! The homepage and navigation page-data providers, and the homepage
//! prerenderer.
//!
//! All three ignore the context they are handed — their whole job is to turn
//! the loaded YAML config into template context — but each is only reachable
//! through a `PageContext` or a `PagePrepareContext`, and both need a real
//! `WebConfig`. The deployment's own `services/web/config.yaml` supplies one,
//! which also makes these tests fail if that file ever stops matching the
//! shape the runtime deserialises it into.

use std::sync::Arc;

use systemprompt::extension::prelude::{
    PageContext, PageDataProvider, PagePrepareContext, PagePrerenderer,
};
use systemprompt::models::services::WebConfig;
use systemprompt_web_site::homepage::{
    HomepageConfig, HomepagePageDataProvider, HomepagePrerenderer,
};
use systemprompt_web_site::navigation::{NavigationConfig, NavigationPageDataProvider};

const WEB_CONFIG_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../services/web/config.yaml"
);

fn web_config() -> WebConfig {
    let raw = std::fs::read_to_string(WEB_CONFIG_PATH).expect("the deployment ships a web config");
    serde_yaml::from_str(&raw).expect("services/web/config.yaml deserialises into a WebConfig")
}

fn homepage_config() -> Arc<HomepageConfig> {
    let yaml = "\
hero:\n\
\x20 title: Governance for agents\n\
\x20 subtitle: Every agent call, audited.\n\
\x20 cta: Get started\n\
\x20 cta_secondary: Read the docs\n\
value_props:\n\
\x20 - id: audited\n\
\x20   title: Audited\n\
\x20   subtitle: Every call lands in the spine.\n\
\x20   icon: shield\n";
    Arc::new(serde_yaml::from_str(yaml).expect("the homepage fixture matches HomepageConfig"))
}

fn navigation_config() -> Arc<NavigationConfig> {
    let yaml = "\
header:\n\
\x20 items:\n\
\x20   - id: docs\n\
\x20     label: Documentation\n\
\x20     href: /documentation\n\
footer:\n\
\x20 legal:\n\
\x20   - path: /legal/privacy\n\
\x20     label: Privacy\n\
social:\n\
\x20 - href: https://example.test/x\n\
\x20   type: x\n\
\x20   label: X\n";
    Arc::new(serde_yaml::from_str(yaml).expect("the navigation fixture matches NavigationConfig"))
}

fn page_data(provider: &dyn PageDataProvider, page_type: &str) -> serde_json::Value {
    let config = web_config();
    let erased = ();
    let ctx = PageContext::new(page_type, &config, &erased, &erased);
    block_on(provider.provide_page_data(&ctx))
}

// The provider and prerenderer traits are async; these bodies never await
// anything that yields, so a current-thread runtime per call keeps the tests
// free of a shared one.
fn block_on<T>(future: impl Future<Output = Result<T, systemprompt::traits::ProviderError>>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(future)
        .expect("the provider succeeds")
}

#[test]
fn the_deployments_web_config_still_deserialises() {
    let config = web_config();

    assert!(
        !config.branding.name.is_empty(),
        "the shipped config names the site"
    );
}

#[test]
fn the_homepage_provider_serves_only_the_homepage() {
    let provider = HomepagePageDataProvider::new(homepage_config());

    assert_eq!(provider.provider_id(), "homepage");
    assert_eq!(provider.applies_to_pages(), vec!["homepage"]);
    assert_eq!(provider.priority(), 50);
}

#[test]
fn the_homepage_provider_nests_its_config_under_site_homepage() {
    let provider = HomepagePageDataProvider::new(homepage_config());

    let data = page_data(&provider, "homepage");

    assert_eq!(
        data["site"]["homepage"]["hero"]["title"],
        "Governance for agents"
    );
    assert_eq!(
        data["site"]["homepage"]["value_props"][0]["title"],
        "Audited"
    );
}

#[test]
fn an_empty_homepage_config_still_renders_the_site_homepage_key() {
    let provider = HomepagePageDataProvider::new(Arc::new(HomepageConfig::default()));

    let data = page_data(&provider, "homepage");

    assert!(
        data["site"]["homepage"].is_object(),
        "the template's `site.homepage.*` lookups must not hit a missing key: {data}"
    );
    assert!(data["site"]["homepage"]["hero"].is_null());
}

#[test]
fn the_navigation_provider_applies_to_every_page_at_the_lowest_priority() {
    let provider = NavigationPageDataProvider::new(navigation_config());

    assert_eq!(provider.provider_id(), "navigation");
    assert!(
        provider.applies_to_pages().is_empty(),
        "an empty page list means every page, since nav is on all of them"
    );
    assert_eq!(
        provider.priority(),
        10,
        "nav runs before the section providers that may override its keys"
    );
}

#[test]
fn the_navigation_provider_splits_header_footer_and_social_under_site() {
    let provider = NavigationPageDataProvider::new(navigation_config());

    let data = page_data(&provider, "documentation");

    assert_eq!(data["site"]["header_nav"]["items"][0]["id"], "docs");
    assert_eq!(
        data["site"]["navigation"]["footer"]["legal"][0]["label"],
        "Privacy"
    );
    assert_eq!(data["site"]["navigation"]["social"][0]["type"], "x");
    assert!(
        data["site"]["docs_sidebar"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "an absent sidebar is an empty array, not a missing key: {data}"
    );
}

#[test]
fn navigation_emits_the_fixed_app_and_docs_routes() {
    let provider = NavigationPageDataProvider::new(navigation_config());

    let data = page_data(&provider, "homepage");

    assert_eq!(data["nav"]["app_url"], "/app");
    assert_eq!(data["nav"]["docs_url"], "/documentation");
}

#[test]
fn navigation_branding_is_null_until_it_is_supplied() {
    let without = NavigationPageDataProvider::new(navigation_config());

    let data = page_data(&without, "homepage");

    assert!(
        data["site"]["branding"].is_null(),
        "a deployment with no theme.yaml branding renders the key as null: {data}"
    );
}

#[test]
fn navigation_carries_the_branding_it_was_given() {
    let branding = systemprompt_web_site::navigation::BrandingConfig {
        name: "Astound Digital".to_owned(),
        ..Default::default()
    };
    let provider =
        NavigationPageDataProvider::new(navigation_config()).with_branding(Some(branding));

    let data = page_data(&provider, "homepage");

    assert_eq!(data["site"]["branding"]["name"], "Astound Digital");
}

#[test]
fn the_homepage_prerenderer_targets_index_html() {
    let config = web_config();
    let erased = ();
    let dist = std::path::Path::new("/nonexistent-dist");
    let ctx = PagePrepareContext::new(&config, &erased, &erased, dist);
    let prerenderer = HomepagePrerenderer::new(homepage_config());

    assert_eq!(prerenderer.page_type(), "homepage");
    assert_eq!(
        prerenderer.priority(),
        150,
        "the homepage is prepared before the lower-priority section pages"
    );

    let spec = block_on(prerenderer.prepare(&ctx)).expect("the homepage always has a render spec");

    assert_eq!(spec.output_path, std::path::PathBuf::from("index.html"));
}

#[test]
fn the_prerenderer_and_the_provider_build_the_same_context() {
    let config = homepage_config();
    let web = web_config();
    let erased = ();
    let dist = std::path::Path::new("/nonexistent-dist");

    let from_provider = page_data(
        &HomepagePageDataProvider::new(Arc::clone(&config)),
        "homepage",
    );
    let spec = block_on(
        HomepagePrerenderer::new(config)
            .prepare(&PagePrepareContext::new(&web, &erased, &erased, dist)),
    )
    .expect("the homepage always has a render spec");

    assert_eq!(
        spec.base_data, from_provider,
        "the runtime and build-time render paths must not drift"
    );
}
