//! The admin CSS bundle is served as one file with no source map, so cascade
//! order is decided entirely by the order `collect_css_files` returns paths
//! in. That order is the numeric filename prefix (`12-login.css` before
//! `19-profile.css`), which only holds because the paths are sorted; and
//! non-CSS files in the same directory must never reach the bundle.
//!
//! `concatenate_css_files` reports failures rather than returning an error,
//! because the job turns a non-zero failure count into its own message — a
//! silently skipped file would ship a stylesheet missing a rule.

use std::path::PathBuf;

use systemprompt_web_extension::jobs::internals::{collect_css_files, concatenate_css_files};
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, body: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, body).unwrap();
    path
}

#[tokio::test]
async fn collected_files_are_sorted_by_path() {
    let dir = TempDir::new().unwrap();
    write(&dir, "19-profile.css", "b{}");
    write(&dir, "01-reset.css", "a{}");
    write(&dir, "12-login.css", "c{}");

    let files = collect_css_files(dir.path()).await.unwrap();
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["01-reset.css", "12-login.css", "19-profile.css"]);
}

#[tokio::test]
async fn non_css_files_are_excluded() {
    let dir = TempDir::new().unwrap();
    write(&dir, "keep.css", "a{}");
    write(&dir, "notes.md", "# no");
    write(&dir, "noextension", "x");

    let files = collect_css_files(dir.path()).await.unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("keep.css"));
}

#[tokio::test]
async fn a_missing_directory_is_an_error_not_an_empty_bundle() {
    let dir = TempDir::new().unwrap();
    let result = collect_css_files(&dir.path().join("absent")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn concatenation_joins_with_a_newline_and_counts_each_file() {
    let dir = TempDir::new().unwrap();
    let files = vec![
        write(&dir, "a.css", "a{}"),
        write(&dir, "b.css", "b{}"),
        write(&dir, "c.css", "c{}"),
    ];

    let (bundle, bundled, failed) = concatenate_css_files(&files).await;
    assert_eq!(bundle, "a{}\nb{}\nc{}");
    assert_eq!((bundled, failed), (3, 0));
}

#[tokio::test]
async fn an_unreadable_file_is_counted_and_the_rest_still_bundle() {
    let dir = TempDir::new().unwrap();
    let files = vec![
        write(&dir, "a.css", "a{}"),
        dir.path().join("gone.css"),
        write(&dir, "c.css", "c{}"),
    ];

    let (bundle, bundled, failed) = concatenate_css_files(&files).await;
    assert_eq!(bundle, "a{}\nc{}");
    assert_eq!((bundled, failed), (2, 1));
}

#[tokio::test]
async fn an_empty_file_list_bundles_to_nothing() {
    let (bundle, bundled, failed) = concatenate_css_files(&[]).await;
    assert!(bundle.is_empty());
    assert_eq!((bundled, failed), (0, 0));
}
