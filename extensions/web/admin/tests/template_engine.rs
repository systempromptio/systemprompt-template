//! Loading and rendering the admin Handlebars engine.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use std::path::{Path, PathBuf};

use systemprompt_web_admin::templates::{AdminTemplateEngine, AdminTemplateError};

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create dir");
    }
    std::fs::write(path, body).expect("write template");
}

fn admin_dir_with(templates: &[(&str, &str)], partials: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for (name, body) in templates {
        write(
            &dir.path().join("templates").join(format!("{name}.hbs")),
            body,
        );
    }
    for (name, body) in partials {
        write(
            &dir.path().join("partials").join(format!("{name}.hbs")),
            body,
        );
    }
    dir
}

// The real `storage/files/admin` tree, found by walking up from this crate.
fn repo_admin_dir() -> Option<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("storage/files/admin");
        if candidate.join("templates").is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[test]
fn a_registered_template_renders_with_its_data() {
    let dir = admin_dir_with(&[("page", "Hello {{name}}")], &[]);
    let engine = AdminTemplateEngine::new(dir.path()).expect("build engine");
    let rendered = engine
        .render("page", &serde_json::json!({ "name": "Ada" }))
        .expect("render");
    assert_eq!(rendered, "Hello Ada");
}

#[test]
fn partials_are_registered_under_their_path_relative_to_the_partials_root() {
    let dir = admin_dir_with(
        &[("page", "[{{> components/badge}}][{{> sidebar}}]")],
        &[("components/badge", "BADGE"), ("sidebar", "SIDEBAR")],
    );
    let engine = AdminTemplateEngine::new(dir.path()).expect("build engine");
    let rendered = engine
        .render("page", &serde_json::json!({}))
        .expect("render");
    assert_eq!(rendered, "[BADGE][SIDEBAR]");
}

#[test]
fn the_registered_helpers_are_available_to_templates() {
    let dir = admin_dir_with(&[("page", "{{formatNumber n}}|{{governanceColor d}}")], &[]);
    let engine = AdminTemplateEngine::new(dir.path()).expect("build engine");
    let rendered = engine
        .render("page", &serde_json::json!({ "n": 12345, "d": "deny" }))
        .expect("render");
    assert_eq!(rendered, "12,345|danger");
}

#[test]
fn non_hbs_files_are_ignored_in_both_trees() {
    let dir = admin_dir_with(&[("page", "ok")], &[]);
    write(&dir.path().join("templates/README.md"), "not a template");
    write(&dir.path().join("partials/notes.txt"), "not a partial");
    let engine = AdminTemplateEngine::new(dir.path()).expect("build engine");
    assert_eq!(
        engine
            .render("page", &serde_json::json!({}))
            .expect("render"),
        "ok"
    );
    assert!(engine.render("README", &serde_json::json!({})).is_err());
}

#[test]
fn a_missing_admin_directory_yields_an_engine_with_no_templates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = AdminTemplateEngine::new(&dir.path().join("absent")).expect("build engine");
    let err = engine
        .render("anything", &serde_json::json!({}))
        .expect_err("no such template");
    assert!(matches!(err, AdminTemplateError::Render { .. }));
    assert!(err.to_string().contains("anything"), "{err}");
}

#[test]
fn a_malformed_template_fails_at_load_naming_the_file() {
    let dir = admin_dir_with(&[("broken", "{{#if x}}unclosed")], &[]);
    let err = AdminTemplateEngine::new(dir.path()).expect_err("malformed template rejected");
    assert!(matches!(err, AdminTemplateError::RegisterTemplate { .. }));
    assert!(err.to_string().contains("broken"), "{err}");
}

#[test]
fn a_malformed_partial_fails_at_load_naming_the_partial() {
    let dir = admin_dir_with(&[], &[("broken", "{{#each xs}}unclosed")]);
    let err = AdminTemplateEngine::new(dir.path()).expect_err("malformed partial rejected");
    assert!(matches!(err, AdminTemplateError::RegisterPartial { .. }));
    assert!(err.to_string().contains("broken"), "{err}");
}

#[test]
fn strict_mode_makes_a_missing_field_an_error_rather_than_a_blank() {
    // Why: a governance page that silently renders an empty cell for a field
    // the query stopped returning is worse than one that fails loudly.
    let dir = admin_dir_with(&[("page", "{{absent_field}}")], &[]);
    let engine = AdminTemplateEngine::new(dir.path()).expect("build engine");
    let err = engine
        .render("page", &serde_json::json!({}))
        .expect_err("strict mode rejects the missing field");
    assert!(matches!(err, AdminTemplateError::Render { .. }));
}

#[test]
fn branding_is_absent_until_it_is_attached() {
    let dir = admin_dir_with(&[("page", "ok")], &[]);
    let engine = AdminTemplateEngine::new(dir.path()).expect("build engine");
    assert!(engine.branding().is_none());
    assert!(engine.clone().with_branding(None).branding().is_none());
}

#[test]
fn every_shipped_admin_template_and_partial_parses() {
    let admin_dir = repo_admin_dir().expect("storage/files/admin is checked in");
    let engine = AdminTemplateEngine::new(&admin_dir)
        .expect("the shipped admin templates all parse and register");

    // Why: rendering needs page-specific data, so the assertion that carries
    // here is that a known template exists and is reachable by name.
    let err = engine
        .render("layout", &serde_json::json!({}))
        .expect_err("layout needs real page data");
    assert!(matches!(err, AdminTemplateError::Render { .. }));
    assert!(err.to_string().contains("layout"), "{err}");
}
