//! `copy_extension_assets` is the last step between a file in `storage/` and
//! the bytes the browser gets, and its required/optional split is what stops a
//! half-published site: a missing *required* asset aborts the whole copy, a
//! missing optional one is counted and skipped. Nested destinations must also
//! have their directories created — `web/dist/` starts empty on a fresh
//! deploy, so a copy that assumed the parent existed would fail every time.

use systemprompt::extension::{AssetDefinition, AssetType};
use systemprompt_web_extension::jobs::internals::{copy_all_assets, copy_asset};
use tempfile::TempDir;

#[tokio::test]
async fn copying_creates_missing_parent_directories() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let source = src.path().join("admin-bundle.css");
    std::fs::write(&source, "body{}").unwrap();

    let asset = AssetDefinition::css(&source, "css/admin/admin-bundle.css");
    copy_asset(dist.path(), "web", &asset).await.unwrap();

    let dest = dist.path().join("css/admin/admin-bundle.css");
    assert_eq!(std::fs::read_to_string(dest).unwrap(), "body{}");
}

#[tokio::test]
async fn copying_a_missing_source_is_an_error() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let asset = AssetDefinition::js(src.path().join("absent.js"), "js/absent.js");
    assert!(copy_asset(dist.path(), "web", &asset).await.is_err());
}

#[tokio::test]
async fn a_missing_optional_asset_is_counted_and_the_rest_still_copy() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let present = src.path().join("present.css");
    std::fs::write(&present, "a{}").unwrap();

    let assets = vec![
        (
            "web",
            AssetDefinition::builder(
                src.path().join("absent.css"),
                "css/absent.css",
                AssetType::Css,
            )
            .optional()
            .build(),
        ),
        ("web", AssetDefinition::css(&present, "css/present.css")),
    ];

    let (copied, failed) = copy_all_assets(dist.path(), assets).await.unwrap();
    assert_eq!((copied, failed), (1, 1));
    assert!(dist.path().join("css/present.css").exists());
}

#[tokio::test]
async fn a_missing_required_asset_aborts_the_whole_copy() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let later = src.path().join("later.css");
    std::fs::write(&later, "a{}").unwrap();

    let assets = vec![
        (
            "web",
            AssetDefinition::css(src.path().join("absent.css"), "css/absent.css"),
        ),
        ("web", AssetDefinition::css(&later, "css/later.css")),
    ];

    assert!(copy_all_assets(dist.path(), assets).await.is_err());
    assert!(!dist.path().join("css/later.css").exists());
}

#[tokio::test]
async fn an_empty_asset_list_copies_nothing_successfully() {
    let dist = TempDir::new().unwrap();
    let (copied, failed) = copy_all_assets(dist.path(), Vec::new()).await.unwrap();
    assert_eq!((copied, failed), (0, 0));
}
