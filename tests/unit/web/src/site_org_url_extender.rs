//! `OrgUrlExtender` — the extender every template relies on for the absolute
//! URLs in its `<head>`.
//!
//! Its one input is the process-wide `Config`'s external URL, which is a
//! `OnceLock`: this binary installs one fixed value, so the URLs asserted here
//! are stable regardless of which test runs first.

use systemprompt::models::Config;
use systemprompt::models::config::RateLimitConfig;
use systemprompt::models::profile::{ContentNegotiationConfig, SecurityHeadersConfig};
use systemprompt::models::services::WebConfig;
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};
use systemprompt_web_site::extenders::OrgUrlExtender;

const ORG_URL: &str = "https://astound.example";

const WEB_CONFIG_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../services/web/config.yaml"
);

fn web_config() -> WebConfig {
    let raw = std::fs::read_to_string(WEB_CONFIG_PATH).expect("the deployment ships a web config");
    serde_yaml::from_str(&raw).expect("services/web/config.yaml deserialises into a WebConfig")
}

fn install_config() {
    if Config::is_initialized() {
        return;
    }
    let _ = Config::install(Config {
        instance_id: "org-url-tests".to_owned(),
        max_concurrent_streams: 16,
        sitename: "astound-test".to_owned(),
        database_type: "postgres".to_owned(),
        database_url: "postgres://unused".to_owned(),
        database_write_url: None,
        github_link: String::new(),
        github_token: None,
        system_path: "/tmp".to_owned(),
        services_path: "/tmp".to_owned(),
        bin_path: "/tmp".to_owned(),
        skills_path: "/tmp".to_owned(),
        settings_path: "/tmp".to_owned(),
        content_config_path: "/tmp".to_owned(),
        geoip_database_path: None,
        web_path: "/tmp".to_owned(),
        web_config_path: "/tmp".to_owned(),
        web_metadata_path: "/tmp".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 0,
        metrics_port: None,
        api_server_url: ORG_URL.to_owned(),
        api_internal_url: ORG_URL.to_owned(),
        api_external_url: ORG_URL.to_owned(),
        jwt_issuer: "https://issuer.test".to_owned(),
        jwt_access_token_expiration: 3_600,
        jwt_refresh_token_expiration: 86_400,
        jwt_audiences: vec![],
        allowed_resource_audiences: vec![],
        trusted_issuers: vec![],
        id_jag_ttl_secs: 300,
        signing_key_path: std::path::PathBuf::from("signing_key.pem"),
        use_https: true,
        rate_limits: RateLimitConfig::default(),
        cors_allowed_origins: vec![],
        trusted_proxies: vec![],
        is_cloud: false,
        content_negotiation: ContentNegotiationConfig::default(),
        security_headers: SecurityHeadersConfig::default(),
        allow_registration: false,
        login_page_url: None,
        system_admin_username: "admin".to_owned(),
        system_admin_email: None,
    });
}

fn extend(mut data: serde_json::Value) -> serde_json::Value {
    install_config();
    let web = web_config();
    let item = serde_json::json!({});
    let items: Vec<serde_json::Value> = vec![];
    // OrgUrlExtender never reads the per-source config, so an empty mapping
    // exercises it fully.
    let config = Default::default();
    let erased = ();
    let ctx = ExtenderContext::builder(&item, &items, &config, &web, &erased).build();

    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(OrgUrlExtender::new().extend(&ctx, &mut data))
        .expect("the extender succeeds once a Config is installed");
    data
}

#[test]
fn the_extender_applies_to_every_template_at_the_lowest_priority() {
    let extender = OrgUrlExtender::new();

    assert_eq!(extender.extender_id(), "org-url");
    assert!(
        extender.applies_to().is_empty(),
        "an empty list means every template, since these URLs are in every head"
    );
    assert_eq!(extender.priority(), 10);
}

#[test]
fn default_and_new_build_the_same_extender() {
    assert_eq!(
        OrgUrlExtender.extender_id(),
        OrgUrlExtender::new().extender_id()
    );
}

#[test]
fn the_three_url_keys_are_absolute_and_derived_from_the_external_url() {
    let data = extend(serde_json::json!({}));

    assert_eq!(data["ORG_URL"], ORG_URL);
    assert_eq!(data["ORG_LOGO"], format!("{ORG_URL}/files/images/logo.svg"));
    assert_eq!(
        data["DEFAULT_IMAGE"],
        format!("{ORG_URL}/files/images/logo.png"),
        "the social-card fallback is a PNG; the inline logo is the SVG"
    );
}

#[test]
fn existing_keys_are_preserved_and_the_url_keys_overwrite_their_own() {
    let data = extend(serde_json::json!({ "title": "Docs", "ORG_URL": "stale" }));

    assert_eq!(data["title"], "Docs", "unrelated keys survive");
    assert_eq!(
        data["ORG_URL"], ORG_URL,
        "a stale value from an earlier extender is replaced, not kept"
    );
}

#[test]
fn a_non_object_payload_is_left_alone_rather_than_replaced() {
    let data = extend(serde_json::json!(["a", "b"]));

    assert_eq!(
        data,
        serde_json::json!(["a", "b"]),
        "there is nowhere to insert the keys, and the extender must not discard the data"
    );
}

// `Config::get`'s error arm is not reachable from this binary: the singleton is
// installed by the first test that needs it and can never be uninstalled, so a
// test asserting the uninitialised path would have to be the only test in its
// own process.
