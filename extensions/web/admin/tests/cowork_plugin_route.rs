use std::fs;

use systemprompt_web_admin::test_support::resolve_within;
use tempfile::TempDir;

#[test]
fn resolves_normal_relative_path() {
    let tmp = TempDir::new().unwrap();
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(plugin_dir.join("agents")).unwrap();
    fs::write(plugin_dir.join("agents/main.md"), b"hello").unwrap();

    let resolved = resolve_within(&plugin_dir, "agents/main.md").expect("resolves");
    assert_eq!(fs::read(&resolved).unwrap(), b"hello");
}

#[test]
fn rejects_parent_traversal_component() {
    let tmp = TempDir::new().unwrap();
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(tmp.path().join("secret.md"), b"top secret").unwrap();

    let err = resolve_within(&plugin_dir, "../secret.md").expect_err("must reject");
    assert_eq!(err, "non-canonical component");
}

#[test]
fn rejects_absolute_path() {
    let tmp = TempDir::new().unwrap();
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(&plugin_dir).unwrap();

    let err = resolve_within(&plugin_dir, "/etc/passwd").expect_err("must reject");
    assert_eq!(err, "non-canonical component");
}

#[test]
fn rejects_empty_path() {
    let tmp = TempDir::new().unwrap();
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(&plugin_dir).unwrap();

    let err = resolve_within(&plugin_dir, "").expect_err("must reject");
    assert_eq!(err, "empty path");
}

#[test]
fn rejects_directory_target() {
    let tmp = TempDir::new().unwrap();
    let plugin_dir = tmp.path().join("planner");
    fs::create_dir_all(plugin_dir.join("agents")).unwrap();

    let err = resolve_within(&plugin_dir, "agents").expect_err("must reject");
    assert_eq!(err, "not a file");
}

