//! The `systemprompt-web-content` service layer against the real schema.
//!
//! The services are thin wrappers over the repositories, so what these tests
//! pin is the part that is NOT a passthrough: the id/slug string-to-newtype
//! conversion `ContentService` performs, the click-then-redirect ordering in
//! `LinkService::process_redirect`, the uniqueness of the short codes
//! `LinkGenerationService` mints, and the default page size `SearchService`
//! applies when the request omits one.

use std::sync::Arc;

use systemprompt::identifiers::{SessionId, SourceId};
use systemprompt_web_content::repository::{
    ContentRepository, UpdateContentParams, UpdateContentSeed,
};
use systemprompt_web_content::services::{
    ContentService, LinkGenerationService, LinkService, SearchService,
};
use systemprompt_web_shared::error::BlogError;
use systemprompt_web_shared::models::{SearchRequest, UtmParams};

use crate::fixtures::{content_params, link_params, source_id};
use crate::tempdb::TempDb;

#[tokio::test]
async fn content_service_creates_and_reads_back_by_string_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = ContentService::new(Arc::clone(&db.pool));
    let source = source_id();

    let created = service
        .create(&content_params("service-create", &source))
        .await
        .expect("create content through the service");

    let by_id = service
        .get_by_id(created.id.as_str())
        .await
        .expect("read back by the string id")
        .expect("the created row resolves");

    assert_eq!(by_id.slug, "service-create");
    assert_eq!(
        by_id.id, created.id,
        "the service wraps the &str id into ContentId without mangling it"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn content_service_get_by_id_is_none_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = ContentService::new(Arc::clone(&db.pool));

    let missing = service
        .get_by_id("no-such-content")
        .await
        .expect("an absent id is not an error");

    assert!(missing.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn content_service_scopes_slug_lookups_to_the_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = ContentService::new(Arc::clone(&db.pool));
    let mine = source_id();
    let other = SourceId::new("other-source".to_string());

    let created = service
        .create(&content_params("scoped-service", &mine))
        .await
        .expect("create content");

    let by_slug = service
        .get_by_slug("scoped-service")
        .await
        .expect("slug lookup")
        .expect("the slug resolves");
    assert_eq!(by_slug.id, created.id);

    let scoped = service
        .get_by_source_and_slug(&mine, "scoped-service")
        .await
        .expect("scoped lookup")
        .expect("the slug resolves under its own source");
    assert_eq!(scoped.id, created.id);

    let foreign = service
        .get_by_source_and_slug(&other, "scoped-service")
        .await
        .expect("scoped lookup for a foreign source");
    assert!(
        foreign.is_none(),
        "the same slug under another source must not resolve"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn content_service_lists_pages_and_sources() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = ContentService::new(Arc::clone(&db.pool));
    let mine = source_id();
    let other = SourceId::new("other-source".to_string());

    for slug in ["list-a", "list-b", "list-c"] {
        service
            .create(&content_params(slug, &mine))
            .await
            .expect("create content for the source under test");
    }
    service
        .create(&content_params("elsewhere", &other))
        .await
        .expect("create content for an unrelated source");

    let page = service.list(2, 0).await.expect("list the first page");
    assert_eq!(page.len(), 2, "the limit caps the page size");

    let by_source = service.list_by_source(&mine).await.expect("list by source");
    assert_eq!(
        by_source.len(),
        3,
        "list_by_source excludes the other source's row"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn content_service_updates_then_deletes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = ContentService::new(Arc::clone(&db.pool));
    let source = source_id();

    let created = service
        .create(&content_params("service-mutable", &source))
        .await
        .expect("create content");

    let updated = service
        .update(
            &UpdateContentParams::builder(UpdateContentSeed {
                id: created.id.clone(),
                title: "Service revised title".to_string(),
                description: "Service revised description".to_string(),
                body: "Service revised body".to_string(),
                keywords: "delta".to_string(),
                version_hash: "hash-service-v2".to_string(),
            })
            .build(),
        )
        .await
        .expect("update content through the service");

    assert_eq!(updated.title, "Service revised title");
    assert_eq!(updated.version_hash, "hash-service-v2");

    service
        .delete(created.id.as_str())
        .await
        .expect("delete by the string id");

    let after = service
        .get_by_id(created.id.as_str())
        .await
        .expect("read after delete");
    assert!(after.is_none(), "the deleted row is gone");

    db.cleanup().await;
}

#[tokio::test]
async fn link_service_creates_and_resolves_a_short_code() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = LinkService::new(Arc::clone(&db.pool));

    let created = service
        .create(&link_params("svc001"))
        .await
        .expect("create a link through the service");

    let found = service
        .get_by_short_code("svc001")
        .await
        .expect("resolve the short code")
        .expect("an active link resolves");
    assert_eq!(found.id, created.id);

    let missing = service
        .get_by_short_code("nosuch")
        .await
        .expect("resolve an unknown short code");
    assert!(missing.is_none(), "an unknown short code reads as None");

    db.cleanup().await;
}

#[tokio::test]
async fn process_redirect_records_the_click_and_returns_the_target() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = LinkService::new(Arc::clone(&db.pool));

    let created = service
        .create(&link_params("svc002"))
        .await
        .expect("create a link");

    let destination = service
        .process_redirect(
            "svc002",
            SessionId::new("sess-redirect".to_string()),
            Some("integration-test"),
            Some("203.0.113.7"),
        )
        .await
        .expect("process the redirect");

    assert_eq!(
        destination, "https://example.com/svc002",
        "the redirect resolves to the link's full URL"
    );

    let clicks = service
        .get_clicks(&created.id, 10)
        .await
        .expect("read back the click rows");
    assert_eq!(clicks.len(), 1, "the redirect recorded exactly one click");
    assert_eq!(clicks[0].ip_address.as_deref(), Some("203.0.113.7"));
    assert_eq!(clicks[0].user_agent.as_deref(), Some("integration-test"));

    let performance = service
        .get_performance(&created.id)
        .await
        .expect("read the link performance rollup")
        .expect("a link with a click has a performance row");
    assert_eq!(performance.click_count, 1);
    // process_redirect goes through track_click, which never touches
    // unique_click_count — session-unique counting lives only in
    // increment_link_clicks(is_first_click), and the redirect path does not
    // call it. Pinned as observed; if redirects are meant to count uniques,
    // fix process_redirect and flip this to 1.
    assert_eq!(performance.unique_click_count, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn process_redirect_reports_an_unknown_short_code_as_link_not_found() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let service = LinkService::new(Arc::clone(&db.pool));

    let error = service
        .process_redirect(
            "ghost1",
            SessionId::new("sess-ghost".to_string()),
            None,
            None,
        )
        .await
        .expect_err("an unresolvable short code must not redirect");

    match error {
        BlogError::LinkNotFound(code) => assert_eq!(code, "ghost1"),
        other => panic!("expected LinkNotFound, got {other:?}"),
    }

    db.cleanup().await;
}

#[tokio::test]
async fn link_generation_persists_a_unique_short_code_per_call() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let generator = LinkGenerationService::new(Arc::clone(&db.pool));
    let links = LinkService::new(Arc::clone(&db.pool));

    let first = generator
        .generate(
            "https://example.com/campaign".to_string(),
            Some("launch".to_string()),
            Some(UtmParams {
                source: Some("newsletter".to_string()),
                medium: Some("email".to_string()),
                campaign: Some("launch".to_string()),
                term: None,
                content: None,
            }),
        )
        .await
        .expect("generate a campaign link");
    let second = generator
        .generate("https://example.com/campaign".to_string(), None, None)
        .await
        .expect("generate a second link for the same target");

    assert_ne!(
        first.short_code, second.short_code,
        "each generated link gets its own short code"
    );
    assert_eq!(first.link_type, "redirect");
    assert_eq!(first.campaign_name.as_deref(), Some("launch"));
    assert!(
        first.campaign_id.is_some(),
        "naming a campaign mints a campaign id"
    );
    assert!(
        second.campaign_id.is_none(),
        "omitting the campaign name leaves the campaign id unset"
    );

    let persisted = links
        .get_by_short_code(&first.short_code)
        .await
        .expect("resolve the generated short code")
        .expect("the generated link was persisted");
    assert_eq!(persisted.id, first.id);
    assert!(
        persisted.full_url().contains("utm_source=newsletter"),
        "the stored UTM parameters are reflected in the redirect target, got {}",
        persisted.full_url()
    );

    db.cleanup().await;
}

#[tokio::test]
async fn generate_for_content_binds_the_link_to_its_source_content() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content = ContentRepository::new(Arc::clone(&db.pool));
    let generator = LinkGenerationService::new(Arc::clone(&db.pool));
    let source = source_id();

    let article = content
        .create(&content_params("linked-article", &source))
        .await
        .expect("create the content the link points from");

    let link = generator
        .generate_for_content(
            "https://example.com/linked".to_string(),
            article.id.clone(),
            Some("in-article".to_string()),
        )
        .await
        .expect("generate a content-scoped link");

    assert_eq!(link.source_content_id.as_ref(), Some(&article.id));
    assert_eq!(link.campaign_name.as_deref(), Some("in-article"));
    assert_eq!(link.link_type, "redirect");

    db.cleanup().await;
}

#[tokio::test]
async fn search_service_applies_a_default_limit_of_twenty() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content = ContentRepository::new(Arc::clone(&db.pool));
    let search = SearchService::new(Arc::clone(&db.pool));
    let source = source_id();

    for n in 0..25 {
        content
            .create(&content_params(&format!("bulk-{n:02}"), &source))
            .await
            .expect("create a matching article");
    }

    let defaulted = search
        .search(&SearchRequest {
            query: "Body for bulk".to_string(),
            filters: None,
            limit: None,
        })
        .await
        .expect("search with no explicit limit");

    assert_eq!(
        defaulted.results.len(),
        20,
        "an absent limit falls back to 20 even though 25 rows match"
    );
    assert_eq!(
        defaulted.total,
        defaulted.results.len(),
        "total counts the returned page, not the whole match set"
    );

    let explicit = search
        .search(&SearchRequest {
            query: "Body for bulk".to_string(),
            filters: None,
            limit: Some(3),
        })
        .await
        .expect("search with an explicit limit");
    assert_eq!(explicit.results.len(), 3);
    assert_eq!(explicit.total, 3);

    db.cleanup().await;
}

#[tokio::test]
async fn search_service_returns_an_empty_response_when_nothing_matches() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content = ContentRepository::new(Arc::clone(&db.pool));
    let search = SearchService::new(Arc::clone(&db.pool));
    let source = source_id();

    content
        .create(&content_params("only-article", &source))
        .await
        .expect("create the sole article");

    let response = search
        .search(&SearchRequest {
            query: "quicksilver-nonexistent".to_string(),
            filters: None,
            limit: None,
        })
        .await
        .expect("search for something absent");

    assert!(response.results.is_empty());
    assert_eq!(response.total, 0);

    db.cleanup().await;
}
