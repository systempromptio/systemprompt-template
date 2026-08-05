//! `SearchRepository` against the real schema.

use std::sync::Arc;

use systemprompt::identifiers::CategoryId;
use systemprompt_web_content::repository::{ContentRepository, SearchRepository};

use crate::fixtures::{content_params, source_id};
use crate::tempdb::TempDb;

#[tokio::test]
async fn search_by_category_returns_only_that_category() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_repo = ContentRepository::new(Arc::clone(&db.pool));
    let search = SearchRepository::new(Arc::clone(&db.pool));
    let source = source_id();
    let guides = CategoryId::new("guides".to_string());
    let news = CategoryId::new("news".to_string());

    content_repo
        .create(&content_params("in-guides", &source).with_category_id(Some(guides.clone())))
        .await
        .expect("create content in the category under test");
    content_repo
        .create(&content_params("in-news", &source).with_category_id(Some(news)))
        .await
        .expect("create content in another category");
    content_repo
        .create(&content_params("uncategorised", &source))
        .await
        .expect("create content with no category");

    let results = search
        .search_by_category(&guides, 10)
        .await
        .expect("search by category");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].slug, "in-guides");
    assert_eq!(results[0].category_id.as_ref(), Some(&guides));
    assert_eq!(
        results[0].view_count, 0,
        "content with no performance-metrics row reports zero views"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn search_by_keyword_matches_title_description_and_body() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_repo = ContentRepository::new(Arc::clone(&db.pool));
    let search = SearchRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    content_repo
        .create(&content_params("governance-primer", &source))
        .await
        .expect("create the matching article");
    content_repo
        .create(&content_params("unrelated-topic", &source))
        .await
        .expect("create a non-matching article");

    let by_title = search
        .search_by_keyword("governance", 10)
        .await
        .expect("search by keyword");
    assert_eq!(by_title.len(), 1);
    assert_eq!(by_title[0].slug, "governance-primer");

    let case_insensitive = search
        .search_by_keyword("GOVERNANCE", 10)
        .await
        .expect("search with different casing");
    assert_eq!(
        case_insensitive.len(),
        1,
        "the keyword match is ILIKE, so casing does not matter"
    );

    let limited = search
        .search_by_keyword("Body for", 1)
        .await
        .expect("search with a limit");
    assert_eq!(limited.len(), 1, "the limit caps the result count");

    db.cleanup().await;
}

#[tokio::test]
async fn search_by_keyword_returns_nothing_when_no_content_matches() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_repo = ContentRepository::new(Arc::clone(&db.pool));
    let search = SearchRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    content_repo
        .create(&content_params("only-article", &source))
        .await
        .expect("create the sole article");

    let results = search
        .search_by_keyword("quicksilver-nonexistent", 10)
        .await
        .expect("search for a keyword nothing contains");

    assert!(
        results.is_empty(),
        "a keyword with no matches returns empty"
    );

    db.cleanup().await;
}
