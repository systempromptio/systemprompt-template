//! `ContentRepository` against the real schema.

use std::sync::Arc;

use chrono::{TimeZone, Utc};
use systemprompt::identifiers::{ContentId, SourceId};
use systemprompt_web_content::repository::{
    ContentRepository, UpdateContentParams, UpdateContentSeed,
};

use crate::fixtures::{content_params, source_id};
use crate::tempdb::TempDb;

#[tokio::test]
async fn create_then_get_by_id_round_trips() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let created = repo
        .create(&content_params("round-trip", &source))
        .await
        .expect("create content");
    let fetched = repo
        .get_by_id(&created.id)
        .await
        .expect("get by id")
        .expect("created row is readable by id");

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.slug, "round-trip");
    assert_eq!(fetched.title, "Title for round-trip");
    assert_eq!(fetched.body, "Body for round-trip");
    assert_eq!(fetched.source_id, source);
    assert_eq!(fetched.version_hash, "hash-v1");

    db.cleanup().await;
}

#[tokio::test]
async fn get_by_slug_returns_the_created_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let created = repo
        .create(&content_params("by-slug", &source))
        .await
        .expect("create content");
    let fetched = repo
        .get_by_slug("by-slug")
        .await
        .expect("get by slug")
        .expect("row is readable by slug");

    assert_eq!(fetched.id, created.id);

    db.cleanup().await;
}

#[tokio::test]
async fn get_by_id_is_none_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));

    let missing = repo
        .get_by_id(&ContentId::new("no-such-content".to_string()))
        .await
        .expect("query an absent id");

    assert!(
        missing.is_none(),
        "absent id must read as None, not an error"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn get_by_source_and_slug_is_scoped_to_the_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let mine = source_id();
    let other = SourceId::new("other-source".to_string());

    let created = repo
        .create(&content_params("scoped", &mine))
        .await
        .expect("create content");

    let hit = repo
        .get_by_source_and_slug(&mine, "scoped")
        .await
        .expect("scoped lookup")
        .expect("matching source and slug resolves");
    assert_eq!(hit.id, created.id);

    let miss = repo
        .get_by_source_and_slug(&other, "scoped")
        .await
        .expect("scoped lookup for a foreign source");
    assert!(
        miss.is_none(),
        "the same slug under a different source must not resolve"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn update_replaces_the_mutable_fields() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let created = repo
        .create(&content_params("updatable", &source))
        .await
        .expect("create content");

    let params = UpdateContentParams::builder(UpdateContentSeed {
        id: created.id.clone(),
        title: "Revised title".to_string(),
        description: "Revised description".to_string(),
        body: "Revised body".to_string(),
        keywords: "gamma".to_string(),
        version_hash: "hash-v2".to_string(),
    })
    .with_image("/img/revised.png")
    .build();

    let updated = repo.update(&params).await.expect("update content");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.title, "Revised title");
    assert_eq!(updated.description, "Revised description");
    assert_eq!(updated.body, "Revised body");
    assert_eq!(updated.keywords, "gamma");
    assert_eq!(updated.version_hash, "hash-v2");
    assert_eq!(updated.image.as_deref(), Some("/img/revised.png"));
    assert_eq!(
        updated.slug, created.slug,
        "update must not disturb identity columns"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn delete_removes_the_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let created = repo
        .create(&content_params("deletable", &source))
        .await
        .expect("create content");

    repo.delete(&created.id).await.expect("delete content");

    let after = repo
        .get_by_id(&created.id)
        .await
        .expect("read after delete");
    assert!(after.is_none(), "deleted row must no longer be readable");

    db.cleanup().await;
}

#[tokio::test]
async fn get_slugs_by_source_returns_only_that_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let mine = source_id();
    let other = SourceId::new("other-source".to_string());

    for slug in ["one", "two"] {
        repo.create(&content_params(slug, &mine))
            .await
            .expect("create content for the source under test");
    }
    repo.create(&content_params("elsewhere", &other))
        .await
        .expect("create content for an unrelated source");

    let mut slugs = repo
        .get_slugs_by_source(&mine)
        .await
        .expect("list slugs by source");
    slugs.sort();

    assert_eq!(slugs, vec!["one".to_string(), "two".to_string()]);

    db.cleanup().await;
}

#[tokio::test]
async fn delete_orphaned_slugs_drops_the_missing_and_keeps_the_found() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let survivor = repo
        .create(&content_params("survivor", &source))
        .await
        .expect("create the slug still present on disk");
    let orphan = repo
        .create(&content_params("orphan", &source))
        .await
        .expect("create the slug whose file was removed");

    let deleted = repo
        .delete_orphaned_slugs(&source, &["survivor".to_string()])
        .await
        .expect("prune orphaned slugs");

    assert_eq!(deleted, 1, "exactly the one orphan is pruned");
    assert!(
        repo.get_by_id(&orphan.id)
            .await
            .expect("read the orphan")
            .is_none(),
        "the orphaned slug is gone"
    );
    assert!(
        repo.get_by_id(&survivor.id)
            .await
            .expect("read the survivor")
            .is_some(),
        "a slug still present on disk survives the prune"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn delete_orphaned_slugs_leaves_other_sources_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let mine = source_id();
    let other = SourceId::new("other-source".to_string());

    repo.create(&content_params("mine", &mine))
        .await
        .expect("create content for the pruned source");
    let untouched = repo
        .create(&content_params("theirs", &other))
        .await
        .expect("create content for an unrelated source");

    let deleted = repo
        .delete_orphaned_slugs(&mine, &[])
        .await
        .expect("prune every slug of one source");

    assert_eq!(deleted, 1, "only the pruned source's rows are counted");
    assert!(
        repo.get_by_id(&untouched.id)
            .await
            .expect("read the other source's row")
            .is_some(),
        "pruning one source must not touch another"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_orders_by_published_at_and_honours_the_page_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    for (slug, day) in [("oldest", 1), ("middle", 2), ("newest", 3)] {
        let mut params = content_params(slug, &source);
        params.published_at = Utc
            .with_ymd_and_hms(2026, 1, day, 12, 0, 0)
            .single()
            .expect("fixed timestamp is unambiguous");
        repo.create(&params).await.expect("create content");
    }

    let page = repo.list(10, 0).await.expect("list content");
    let slugs: Vec<&str> = page.iter().map(|c| c.slug.as_str()).collect();
    assert_eq!(
        slugs,
        vec!["newest", "middle", "oldest"],
        "list is ordered newest-published first"
    );

    let second = repo.list(1, 1).await.expect("list the second page");
    assert_eq!(second.len(), 1, "limit caps the page size");
    assert_eq!(second[0].slug, "middle", "offset skips the first row");

    let by_source = repo
        .list_by_source(&source)
        .await
        .expect("list content by source");
    assert_eq!(by_source.len(), 3);

    db.cleanup().await;
}

#[tokio::test]
async fn list_all_pages_across_every_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));

    for (slug, source, day) in [
        ("from-blog", "blog", 1),
        ("from-docs", "documentation", 2),
        ("from-guides", "guides", 3),
    ] {
        let mut params = content_params(slug, &SourceId::new(source.to_string()));
        params.published_at = Utc
            .with_ymd_and_hms(2026, 1, day, 12, 0, 0)
            .single()
            .expect("fixed timestamp is unambiguous");
        repo.create(&params).await.expect("create content");
    }

    let all = repo.list_all(10, 0).await.expect("list every source");
    let slugs: Vec<&str> = all.iter().map(|c| c.slug.as_str()).collect();
    assert_eq!(
        slugs,
        vec!["from-guides", "from-docs", "from-blog"],
        "list_all spans sources, newest-published first"
    );

    let second = repo.list_all(1, 1).await.expect("list the second page");
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].slug, "from-docs");

    db.cleanup().await;
}
