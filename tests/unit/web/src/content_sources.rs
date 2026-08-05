//! A `ContentSourceValidated` can only be produced by validation, so its
//! accessors are the sole view downstream ingestion has of a source. The
//! filter that matters is `enabled_sources`: a disabled source stays in
//! `all_sources` (its path is never even checked for existence) and must not
//! leak into an ingestion run.

use std::path::Path;
use systemprompt::identifiers::{CategoryId, SourceId};
use systemprompt_web_shared::config::{BlogConfigRaw, BlogConfigValidated, ContentSourceRaw};

// The crate's own `src/` is a directory that is guaranteed to exist, so an
// enabled source can be validated without creating fixtures on disk.
fn base_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn source(id: &str, path: &str, enabled: bool) -> ContentSourceRaw {
    ContentSourceRaw {
        source_id: SourceId::new(id),
        category_id: CategoryId::new("guides"),
        path: path.to_owned(),
        allowed_content_types: vec!["guide".to_owned(), "blog".to_owned()],
        enabled,
        override_existing: enabled,
    }
}

fn validated(sources: Vec<ContentSourceRaw>) -> BlogConfigValidated {
    BlogConfigValidated::validate(
        BlogConfigRaw {
            content_sources: sources,
            base_url: "https://example.com".to_owned(),
            enable_link_tracking: true,
        },
        base_path(),
    )
    .expect("fixture config validates")
}

#[test]
fn accessors_expose_the_validated_source_verbatim() {
    let config = validated(vec![source("guides", "src", true)]);
    let src = &config.all_sources()[0];

    assert_eq!(src.source_id().as_str(), "guides");
    assert_eq!(src.category_id().as_str(), "guides");
    assert!(src.path().ends_with("src"), "got {:?}", src.path());
    assert_eq!(src.allowed_content_types(), ["guide", "blog"]);
    assert!(src.is_enabled());
    assert!(src.override_existing());
}

#[test]
fn a_disabled_source_is_validated_without_its_path_having_to_exist() {
    let config = validated(vec![source("archive", "no/such/dir", false)]);
    let src = &config.all_sources()[0];

    assert!(!src.is_enabled());
    assert!(!src.override_existing());
    assert!(src.path().ends_with("no/such/dir"), "got {:?}", src.path());
}

#[test]
fn enabled_sources_filters_out_the_disabled_ones() {
    let config = validated(vec![
        source("guides", "src", true),
        source("archive", "no/such/dir", false),
    ]);
    assert_eq!(config.all_sources().len(), 2);

    let enabled: Vec<&str> = config
        .enabled_sources()
        .map(|s| s.source_id().as_str())
        .collect();
    assert_eq!(enabled, ["guides"]);
}

#[test]
fn a_config_with_no_sources_yields_an_empty_iterator() {
    let config = validated(Vec::new());
    assert!(config.all_sources().is_empty());
    assert_eq!(config.enabled_sources().count(), 0);
}
