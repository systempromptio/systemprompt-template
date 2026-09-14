#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]
//! `docs/profile.schema.json` is the JSON Schema of core's `Profile`, and it
//! is what the self-host guides and editors validate a `profile.yaml` against.
//! It was hand-refreshed, which is how it kept describing `providers:` and
//! `gateway:` long after the loader started rejecting them. This test pins the
//! file to the type: regenerate it with `UPDATE_PROFILE_SCHEMA=1`.

use systemprompt::config::generate_schema;
use systemprompt::models::Profile;

use crate::support::repo_root;

#[test]
fn profile_schema_json_matches_the_profile_type() {
    let path = repo_root().join("docs/profile.schema.json");
    let schema = generate_schema::<Profile>().expect("schema generates");
    let generated = format!(
        "{}\n",
        serde_json::to_string_pretty(&schema).expect("schema serialises")
    );

    if std::env::var_os("UPDATE_PROFILE_SCHEMA").is_some() {
        std::fs::write(&path, &generated).expect("write docs/profile.schema.json");
        return;
    }

    let on_disk = std::fs::read_to_string(&path).expect("docs/profile.schema.json is tracked");
    assert_eq!(
        on_disk, generated,
        "docs/profile.schema.json is stale — regenerate with\n  UPDATE_PROFILE_SCHEMA=1 cargo \
         nextest run --manifest-path tests/Cargo.toml -p web-unit-tests \
         profile_schema_json_matches_the_profile_type"
    );
}

#[test]
fn profile_schema_carries_no_provider_catalog_or_gateway() {
    let schema = generate_schema::<Profile>().expect("schema generates");
    let props = schema["properties"]
        .as_object()
        .expect("Profile schema is an object");
    assert!(
        !props.contains_key("providers") && !props.contains_key("gateway"),
        "providers/gateway are services-tree files, not profile sections"
    );
}
