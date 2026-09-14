#[test]
fn displayed_bridge_version_equals_workspace_release() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let workspace: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace manifest"),
    )
    .expect("workspace TOML");
    let bridge: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join("bridge/Cargo.toml")).expect("bridge manifest"),
    )
    .expect("bridge TOML");
    assert_eq!(
        bridge["package"]["version"],
        workspace["workspace"]["package"]["version"]
    );
}
