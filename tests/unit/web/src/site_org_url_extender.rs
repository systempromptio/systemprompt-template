//! `OrgUrlExtender` — the extender every template relies on for the absolute
//! URLs in its `<head>`. Its one input is the external URL it is built with.

use systemprompt::manifest::services::WebConfig;
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};
use systemprompt_web_site::extenders::OrgUrlExtender;

const ORG_URL: &str = "https://acme.example";

const WEB_CONFIG_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../services/web/config.yaml"
);

fn web_config() -> WebConfig {
    let raw = std::fs::read_to_string(WEB_CONFIG_PATH).expect("the deployment ships a web config");
    serde_yaml::from_str(&raw).expect("services/web/config.yaml deserialises into a WebConfig")
}

fn extend(mut data: serde_json::Value) -> serde_json::Value {
    let web = web_config();
    let item = serde_json::json!({});
    let items: Vec<serde_json::Value> = vec![];
    // OrgUrlExtender never reads the per-source config, so an empty mapping
    // exercises it fully.
    let config = Default::default();
    let erased = systemprompt::traits::Dependencies::new();
    let ctx = ExtenderContext::builder(&item, &items, &config, &web, &erased).build();

    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(OrgUrlExtender::new(ORG_URL.to_owned()).extend(&ctx, &mut data))
        .expect("the extender succeeds");
    data
}

#[test]
fn the_extender_applies_to_every_template_at_the_lowest_priority() {
    let extender = OrgUrlExtender::new(ORG_URL.to_owned());

    assert_eq!(extender.extender_id(), "org-url");
    assert!(
        extender.applies_to().is_empty(),
        "an empty list means every template, since these URLs are in every head"
    );
    assert_eq!(extender.priority(), 10);
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
