use harness_desktop::views::docks::permission_info::{Preview, preview};
use serde_json::json;

#[test]
fn a_diff_wins_over_the_files_it_touches() {
    let metadata = json!({ "files": ["a.rs"], "diff": "--- a/a.rs\n+++ b/a.rs\n" });
    assert!(matches!(preview(&metadata), Some(Preview::Diff(_))));
}

#[test]
fn commands_urls_and_paths_are_found() {
    assert_eq!(
        preview(&json!({ "command": "ls -la" })),
        Some(Preview::Command("ls -la".to_owned()))
    );
    assert_eq!(
        preview(&json!({ "url": "https://example.com", "host": "example.com" })),
        Some(Preview::Url("https://example.com".to_owned()))
    );
    assert_eq!(
        preview(&json!({ "path": "src/main.rs" })),
        Some(Preview::Path("src/main.rs".to_owned()))
    );
    assert_eq!(
        preview(&json!({ "files": ["a.rs", "b.rs"] })),
        Some(Preview::Path("a.rs, b.rs".to_owned()))
    );
}

#[test]
fn empty_metadata_has_no_preview() {
    assert_eq!(preview(&json!({ "command": "  " })), None);
    assert_eq!(preview(&json!(null)), None);
}
