//! Path resolution and per-source validation for `services/content/`.
//!
//! Validation is the only way to produce a `ContentSourceValidated`, so every
//! refusal here is a source that never reaches an ingestion run. The refusals
//! are deliberately per-source rather than fatal for the file: a config with
//! one broken source reports that source and keeps the rest, which is what
//! makes the error report actionable instead of a single first-failure.
//!
//! Path resolution is the other half. A source path is relative to the config
//! file's own directory unless it is absolute, which is what lets the same
//! `services/content/config.yaml` work from any working directory.

use std::fs;
use std::path::Path;

use systemprompt::identifiers::{CategoryId, SourceId};
use systemprompt_web_shared::config::{BlogConfigRaw, BlogConfigValidated, ContentSourceRaw};

fn source(id: &str, path: &str) -> ContentSourceRaw {
    ContentSourceRaw {
        source_id: SourceId::new(id),
        category_id: CategoryId::new("guides"),
        path: path.to_owned(),
        allowed_content_types: Vec::new(),
        enabled: true,
        override_existing: false,
    }
}

fn validate(
    sources: Vec<ContentSourceRaw>,
    base: &Path,
) -> Result<BlogConfigValidated, systemprompt_web_shared::config::ExtensionConfigErrors> {
    BlogConfigValidated::validate(
        BlogConfigRaw {
            content_sources: sources,
            base_url: "https://example.com".to_owned(),
            enable_link_tracking: true,
        },
        base,
    )
}

fn errors_of(sources: Vec<ContentSourceRaw>, base: &Path) -> String {
    validate(sources, base)
        .expect_err("the fixture is expected to fail validation")
        .to_string()
}

#[test]
fn an_empty_source_id_is_refused_and_named_by_its_index() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut src = source("", "docs");
    src.source_id = SourceId::new("   ");

    let rendered = errors_of(vec![src], temp.path());

    assert!(
        rendered.contains("content_sources[0].source_id"),
        "the refusal points at the offending array slot: {rendered}"
    );
    assert!(
        rendered.contains("source_id cannot be empty"),
        "the refusal says what is wrong: {rendered}"
    );
}

#[test]
fn an_empty_category_id_is_refused_after_the_source_id_passes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut src = source("guides", "docs");
    src.category_id = CategoryId::new("");

    let rendered = errors_of(vec![src], temp.path());

    assert!(
        rendered.contains("category_id cannot be empty"),
        "an empty category is its own refusal: {rendered}"
    );
    assert!(
        !rendered.contains("source_id cannot be empty"),
        "a valid source_id is not also reported: {rendered}"
    );
}

#[test]
fn an_enabled_source_whose_path_is_missing_is_refused() {
    let temp = tempfile::tempdir().expect("tempdir");

    let rendered = errors_of(vec![source("guides", "no/such/dir")], temp.path());

    assert!(
        rendered.contains("path does not exist"),
        "a missing directory is reported as missing: {rendered}"
    );
    assert!(
        rendered.contains("guides"),
        "the refusal names the source that owns the path: {rendered}"
    );
}

#[test]
fn an_enabled_source_pointing_at_a_file_is_refused_as_not_a_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("guides.md"), "# not a directory").expect("fixture file");

    let rendered = errors_of(vec![source("guides", "guides.md")], temp.path());

    assert!(
        rendered.contains("is not a directory"),
        "a file where a directory was expected is a distinct refusal: {rendered}"
    );
}

#[test]
fn a_disabled_source_skips_both_path_checks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut src = source("archive", "no/such/dir");
    src.enabled = false;

    let config = validate(vec![src], temp.path()).expect("a disabled source needs no path on disk");

    assert_eq!(config.all_sources().len(), 1);
    assert_eq!(config.enabled_sources().count(), 0);
}

#[test]
fn an_absolute_path_is_taken_verbatim_and_the_base_path_ignored() {
    let temp = tempfile::tempdir().expect("tempdir");
    let elsewhere = tempfile::tempdir().expect("second tempdir");
    let absolute = elsewhere.path().join("docs");
    fs::create_dir_all(&absolute).expect("absolute source dir");

    let config = validate(
        vec![source("guides", &absolute.to_string_lossy())],
        temp.path(),
    )
    .expect("an existing absolute directory validates");

    let resolved = config.all_sources()[0].path();
    assert!(
        resolved.ends_with("docs"),
        "the absolute path is kept: {resolved:?}"
    );
    assert!(
        !resolved.starts_with(temp.path()),
        "the base path was not prepended to an absolute path: {resolved:?}"
    );
}

#[test]
fn a_relative_path_resolves_against_the_configs_own_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("content/guides")).expect("nested source dir");

    let config = validate(vec![source("guides", "content/guides")], temp.path())
        .expect("a relative path under the base validates");

    let resolved = config.all_sources()[0].path();
    assert!(
        resolved.ends_with("content/guides"),
        "the relative path is joined onto the base: {resolved:?}"
    );
}

#[test]
fn every_broken_source_is_reported_not_just_the_first() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut empty_id = source("x", "docs");
    empty_id.source_id = SourceId::new("");

    let rendered = errors_of(
        vec![empty_id, source("guides", "also/missing")],
        temp.path(),
    );

    assert!(
        rendered.contains("content_sources[0]"),
        "the first broken source is reported: {rendered}"
    );
    assert!(
        rendered.contains("content_sources[1]"),
        "validation continues past the first failure: {rendered}"
    );
}

#[test]
fn a_valid_source_survives_alongside_a_broken_one_being_rejected() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("guides")).expect("good source dir");

    let outcome = validate(
        vec![source("guides", "guides"), source("broken", "absent")],
        temp.path(),
    );

    assert!(
        outcome.is_err(),
        "one broken source fails the whole config; it is not silently dropped"
    );
}

// A `./`-prefixed path is the one form that is resolved against the profile's
// `services/` directory rather than the config file's own directory. No
// profile is bootstrapped in a test process, so this exercises the documented
// fallback: the literal `./services` relative to the working directory. The
// source is disabled so the assertion is about the resolved path alone and
// does not depend on that directory existing.
#[test]
fn a_dot_slash_services_path_resolves_against_the_profile_services_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut src = source("guides", "./services/content/guides");
    src.enabled = false;

    let config = validate(vec![src], temp.path()).expect("a disabled source always validates");

    let resolved = config.all_sources()[0].path();
    assert!(
        resolved.ends_with("content/guides"),
        "the `./services/` prefix is stripped and rejoined onto the services dir: {resolved:?}"
    );
    assert!(
        !resolved.starts_with(temp.path()),
        "a `./` path is not resolved against the config file's directory: {resolved:?}"
    );
}

#[test]
fn load_from_file_reports_a_file_that_cannot_be_read() {
    let temp = tempfile::tempdir().expect("tempdir");

    let rendered = BlogConfigValidated::load_from_file(&temp.path().join("absent.yaml"))
        .expect_err("a missing file is an error at this level")
        .to_string();

    assert!(
        rendered.contains("Failed to read config file"),
        "an unreadable file is distinguished from an unparseable one: {rendered}"
    );
}

#[test]
fn load_from_file_reports_yaml_that_does_not_parse() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("content.yaml");
    fs::write(&path, "content_sources: [\n").expect("broken yaml");

    let rendered = BlogConfigValidated::load_from_file(&path)
        .expect_err("unbalanced YAML does not parse")
        .to_string();

    assert!(
        rendered.contains("Failed to parse config YAML"),
        "the parse failure is named as such: {rendered}"
    );
}

#[test]
fn load_from_file_resolves_source_paths_against_the_files_directory() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("guides")).expect("source dir");
    let path = temp.path().join("content.yaml");
    fs::write(
        &path,
        "base_url: https://astounddigital.com\ncontent_sources:\n  - source_id: guides\n    \
         category_id: guides\n    path: guides\n",
    )
    .expect("config file");

    let config = BlogConfigValidated::load_from_file(&path).expect("the fixture config validates");

    assert_eq!(config.base_url().as_str(), "https://astounddigital.com/");
    assert!(
        config.all_sources()[0].path().ends_with("guides"),
        "the source path resolved next to the config file"
    );
    assert!(
        config.link_tracking_enabled(),
        "link tracking defaults on when the key is absent"
    );
}

#[test]
fn a_config_file_with_only_defaults_still_loads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("content.yaml");
    fs::write(&path, "{}\n").expect("empty config");

    let config = BlogConfigValidated::load_from_file(&path).expect("every field has a default");

    assert!(config.all_sources().is_empty());
    assert_eq!(config.base_url().as_str(), "https://example.com/");
}
