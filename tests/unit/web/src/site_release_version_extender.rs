//! `ReleaseVersionExtender` — the extender that puts the running build's own
//! version into every template context.
//!
//! The value is `env!("CARGO_PKG_VERSION")` of the *site* crate, which takes
//! `version.workspace = true`. This test crate does not — it sits at 0.0.0 —
//! so its own `CARGO_PKG_VERSION` is the wrong constant to compare against.
//! The workspace version is read from the root manifest instead, which is the
//! single value `scripts/sync-release-version.sh` moves on every release.
//! Hosted documentation reaches this value through the `@RELEASE_VERSION@`
//! token, which is why an absent or stale key is a visible defect on every
//! rendered page rather than a quiet one.

use serde_json::Value;
use systemprompt::models::services::WebConfig;
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};
use systemprompt_web_site::extenders::ReleaseVersionExtender;


// Why: the site crate inherits `version.workspace = true`, so the constant it
// bakes in is the root manifest's [workspace.package] version — not this
// crate's, which is 0.0.0 and would make the assertion compare nothing.
fn workspace_version() -> String {
    let raw = std::fs::read_to_string(crate::support::repo_root().join("Cargo.toml"))
        .expect("the root manifest is readable");
    let table = raw
        .split_once("[workspace.package]")
        .expect("the root manifest declares [workspace.package]")
        .1;
    table
        .lines()
        .find_map(|line| line.trim().strip_prefix("version = "))
        .expect("[workspace.package] declares a version")
        .trim()
        .trim_matches('"')
        .to_owned()
}


// Why: WebConfig has no Default, and the extender never reads it, so the
// deployment's own config is the cheapest value that type-checks — the same
// source site_org_url_extender.rs uses.
fn web_config() -> WebConfig {
    let raw = std::fs::read_to_string(crate::support::repo_root().join("services/web/config.yaml"))
        .expect("the deployment ships a web config");
    serde_yaml::from_str(&raw).expect("services/web/config.yaml deserialises into a WebConfig")
}

fn extend(mut data: Value) -> Value {
    let item = serde_json::json!({});
    let items: Vec<Value> = vec![];
    // ReleaseVersionExtender reads neither the per-source config nor the web
    // config, so these values exercise it fully.
    let config = Default::default();
    let web = web_config();
    let erased = ();
    let ctx = ExtenderContext::builder(&item, &items, &config, &web, &erased).build();

    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(ReleaseVersionExtender::new().extend(&ctx, &mut data))
        .expect("the extender reads no external state and cannot fail");
    data
}

#[test]
fn the_extender_applies_to_every_template_at_the_same_priority_as_the_url_keys() {
    let extender = ReleaseVersionExtender::new();

    assert_eq!(extender.extender_id(), "release-version");
    assert!(
        extender.applies_to().is_empty(),
        "an empty list means every template, since the footer carries the version site-wide"
    );
    assert_eq!(extender.priority(), 10);
}

#[test]
// Why: `default_constructed_unit_structs` wants the bare `ReleaseVersionExtender`,
// but that is the one rewrite this test cannot take — agreement between the two
// constructors is the whole assertion, so writing the literal would leave it
// comparing nothing.
#[allow(clippy::default_constructed_unit_structs)]
fn default_and_new_build_the_same_extender() {
    assert_eq!(
        ReleaseVersionExtender::default().extender_id(),
        ReleaseVersionExtender::new().extender_id()
    );
}

#[test]
fn the_version_key_carries_the_crate_version_the_binary_was_built_from() {
    let data = extend(serde_json::json!({}));

    assert_eq!(
        data["RELEASE_VERSION"],
        Value::String(workspace_version()),
        "the site crate bakes in the workspace version, so a release bump must \
         reach every rendered page without anyone editing a template"
    );
}

#[test]
fn existing_keys_are_preserved_and_a_stale_version_is_overwritten() {
    let data = extend(serde_json::json!({ "title": "Docs", "RELEASE_VERSION": "0.0.0" }));

    assert_eq!(data["title"], "Docs", "unrelated keys survive");
    assert_ne!(
        data["RELEASE_VERSION"], "0.0.0",
        "a stale value from an earlier extender is replaced, not kept"
    );
}

#[test]
fn a_non_object_payload_is_left_alone_rather_than_replaced() {
    let data = extend(serde_json::json!(["a", "b"]));

    assert_eq!(
        data,
        serde_json::json!(["a", "b"]),
        "there is nowhere to insert the key, and the extender must not discard the data"
    );
}
