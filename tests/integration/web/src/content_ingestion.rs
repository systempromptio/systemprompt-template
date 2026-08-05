//! `IngestionService` walking a real markdown tree into the real schema.
//!
//! The interesting behaviour is at the seam between the filesystem walk and
//! the upsert: which files are picked up, what a malformed file does to the
//! rest of the run, that a re-ingest rewrites rather than duplicates, and that
//! orphan pruning only fires when the caller asks for it.

use std::path::Path;
use std::sync::Arc;

use systemprompt::identifiers::CategoryId;
use systemprompt_web_content::repository::ContentRepository;
use systemprompt_web_content::services::IngestionService;
use systemprompt_web_shared::models::IngestionOptions;
use tempfile::TempDir;

use crate::fixtures::source_id;
use crate::tempdb::TempDb;

fn category() -> CategoryId {
    CategoryId::new("guides".to_string())
}

fn article(slug: &str, title: &str, body: &str) -> String {
    format!(
        "---\ntitle: {title}\ndescription: Description for {slug}\nauthor: Test \
         Author\npublished_at: 2026-01-01\nslug: {slug}\nkeywords: alpha,beta\nkind: \
         blog\n---\n{body}\n"
    )
}

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create the fixture directory");
    }
    std::fs::write(&path, contents).expect("write the fixture file");
}

#[tokio::test]
async fn ingest_path_creates_a_row_per_markdown_file() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let dir = TempDir::new().expect("create the fixture tree");

    write_file(
        dir.path(),
        "first.md",
        &article("first", "First article", "First body"),
    );
    write_file(
        dir.path(),
        "second.md",
        &article("second", "Second article", "Second body"),
    );

    let report = service
        .ingest_path(dir.path(), &source, &category())
        .await
        .expect("ingest the fixture tree");

    assert_eq!(report.files_found, 2);
    assert_eq!(report.files_processed, 2);
    assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
    assert_eq!(
        report.orphans_deleted, 0,
        "the default options do not prune"
    );

    let stored = repo
        .get_by_slug("first")
        .await
        .expect("read back the ingested row")
        .expect("the first article was persisted");
    assert_eq!(stored.title, "First article");
    assert_eq!(stored.body, "First body");
    assert_eq!(stored.author, "Test Author");
    assert_eq!(stored.category_id.as_ref(), Some(&category()));
    assert_eq!(
        stored.source_id, source,
        "rows are attributed to the source the caller passed"
    );
    assert!(
        !stored.version_hash.is_empty(),
        "ingestion stamps a content hash so a re-run can skip unchanged files"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn ingest_path_descends_into_subdirectories_and_ignores_non_markdown() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let dir = TempDir::new().expect("create the fixture tree");

    write_file(
        dir.path(),
        "top.md",
        &article("top", "Top level", "Top body"),
    );
    write_file(
        dir.path(),
        "nested/deep/inner.md",
        &article("inner", "Nested", "Nested body"),
    );
    write_file(dir.path(), "notes.txt", "not markdown");
    write_file(dir.path(), "image.png", "not markdown either");

    let report = service
        .ingest_path(dir.path(), &source, &category())
        .await
        .expect("ingest the fixture tree");

    assert_eq!(
        report.files_found, 2,
        "only the .md files count towards files_found"
    );
    assert_eq!(report.files_processed, 2);

    let mut slugs = repo
        .get_slugs_by_source(&source)
        .await
        .expect("list the ingested slugs");
    slugs.sort();
    assert_eq!(slugs, vec!["inner".to_string(), "top".to_string()]);

    db.cleanup().await;
}

#[tokio::test]
async fn a_malformed_file_is_reported_without_aborting_the_run() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let dir = TempDir::new().expect("create the fixture tree");

    write_file(
        dir.path(),
        "good.md",
        &article("good", "Good article", "Good body"),
    );
    write_file(
        dir.path(),
        "no-frontmatter.md",
        "Just a body, no frontmatter",
    );

    let report = service
        .ingest_path(dir.path(), &source, &category())
        .await
        .expect("a malformed file must not fail the whole ingest");

    assert_eq!(report.files_found, 2);
    assert_eq!(report.files_processed, 1, "only the well-formed file lands");
    assert_eq!(report.errors.len(), 1);
    assert!(
        report.errors[0].contains("no-frontmatter.md"),
        "the error names the offending file, got {}",
        report.errors[0]
    );
    assert!(
        !report.is_success(),
        "a report carrying errors is not a success"
    );
    assert!(
        repo.get_by_slug("good")
            .await
            .expect("read back the good row")
            .is_some(),
        "the well-formed file still landed"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn re_ingesting_a_changed_file_rewrites_the_row_instead_of_duplicating_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let dir = TempDir::new().expect("create the fixture tree");

    write_file(
        dir.path(),
        "evolving.md",
        &article("evolving", "Original title", "Original body"),
    );
    service
        .ingest_path(dir.path(), &source, &category())
        .await
        .expect("first ingest");
    let first = repo
        .get_by_slug("evolving")
        .await
        .expect("read the first version")
        .expect("the row exists after the first ingest");

    write_file(
        dir.path(),
        "evolving.md",
        &article("evolving", "Revised title", "Revised body"),
    );
    service
        .ingest_path_with_options(
            dir.path(),
            &source,
            &category(),
            IngestionOptions::default().with_override(true),
        )
        .await
        .expect("second ingest");

    let second = repo
        .get_by_slug("evolving")
        .await
        .expect("read the second version")
        .expect("the row still exists after the second ingest");

    assert_eq!(
        second.id, first.id,
        "the slug keeps its identity across runs"
    );
    assert_eq!(second.title, "Revised title");
    assert_eq!(second.body, "Revised body");
    assert_ne!(
        second.version_hash, first.version_hash,
        "a changed file gets a new content hash"
    );
    assert_eq!(
        repo.get_slugs_by_source(&source)
            .await
            .expect("list slugs")
            .len(),
        1,
        "re-ingesting rewrites in place rather than inserting a second row"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn delete_orphans_prunes_rows_whose_file_disappeared() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let dir = TempDir::new().expect("create the fixture tree");

    write_file(
        dir.path(),
        "keeper.md",
        &article("keeper", "Keeper", "Keeper body"),
    );
    write_file(
        dir.path(),
        "goner.md",
        &article("goner", "Goner", "Goner body"),
    );
    service
        .ingest_path(dir.path(), &source, &category())
        .await
        .expect("seed both articles");

    std::fs::remove_file(dir.path().join("goner.md")).expect("remove one fixture file");

    let report = service
        .ingest_path_with_options(
            dir.path(),
            &source,
            &category(),
            IngestionOptions::default().with_delete_orphans(true),
        )
        .await
        .expect("re-ingest with pruning on");

    assert_eq!(report.files_found, 1);
    assert_eq!(report.orphans_deleted, 1);
    assert_eq!(
        repo.get_slugs_by_source(&source).await.expect("list slugs"),
        vec!["keeper".to_string()],
        "only the slug still on disk survives"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn pruning_is_skipped_when_the_walk_found_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let dir = TempDir::new().expect("create the fixture tree");

    write_file(
        dir.path(),
        "survivor.md",
        &article("survivor", "Survivor", "Survivor body"),
    );
    service
        .ingest_path(dir.path(), &source, &category())
        .await
        .expect("seed the article");

    std::fs::remove_file(dir.path().join("survivor.md")).expect("empty the fixture tree");

    let report = service
        .ingest_path_with_options(
            dir.path(),
            &source,
            &category(),
            IngestionOptions::default().with_delete_orphans(true),
        )
        .await
        .expect("re-ingest an empty tree with pruning on");

    assert_eq!(report.files_found, 0);
    assert_eq!(
        report.orphans_deleted, 0,
        "an empty walk must not be read as 'every row is an orphan'"
    );
    assert!(
        repo.get_by_slug("survivor")
            .await
            .expect("read back the row")
            .is_some(),
        "a vanished content directory does not wipe the source's rows"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_rfc3339_publication_timestamp_is_accepted_alongside_a_bare_date() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let dir = TempDir::new().expect("create the fixture tree");
    write_file(
        dir.path(),
        "timestamped.md",
        &article("timestamped", "Timestamped", "Body").replace(
            "published_at: 2026-01-01",
            "published_at: 2026-01-01T09:30:00Z",
        ),
    );

    let report = service
        .ingest_path(dir.path(), &source_id(), &category())
        .await
        .expect("ingest the fixture tree");

    assert_eq!(report.files_processed, 1);
    let stored = ContentRepository::new(Arc::clone(&db.pool))
        .get_by_slug("timestamped")
        .await
        .expect("read the row back")
        .expect("the article was persisted");
    assert_eq!(
        stored.published_at.to_rfc3339(),
        "2026-01-01T09:30:00+00:00"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_publication_date_in_no_recognised_format_is_reported_as_an_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = IngestionService::new(Arc::clone(&db.pool));
    let dir = TempDir::new().expect("create the fixture tree");
    write_file(
        dir.path(),
        "undated.md",
        &article("undated", "Undated", "Body")
            .replace("published_at: 2026-01-01", "published_at: last Tuesday"),
    );

    let report = service
        .ingest_path(dir.path(), &source_id(), &category())
        .await
        .expect("the walk completes and reports the file");

    assert_eq!(report.files_found, 1);
    assert_eq!(report.files_processed, 0);
    assert_eq!(report.errors.len(), 1);
    assert!(
        report.errors[0].contains("Invalid datetime"),
        "unexpected error: {}",
        report.errors[0]
    );

    db.cleanup().await;
}
