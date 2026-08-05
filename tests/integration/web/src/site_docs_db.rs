//! The documentation content enricher and the queries beneath it.
//!
//! `DocsContentDataProvider::enrich_content` is the only caller of
//! `repositories::docs`, and its `ContentDataContext` is publicly
//! constructible — so driving the provider covers both layers at once,
//! including the child-listing rules (root vs nested, one level only) that a
//! query test alone would not pin.

use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};
use systemprompt::identifiers::SourceId;
use systemprompt_web_content::repository::ContentRepository;
use systemprompt_web_shared::models::{ContentKind, ContentLinkMetadata, CreateContentParams};
use systemprompt_web_site::docs::DocsContentDataProvider;

use crate::fixtures::content_params;
use crate::tempdb::TempDb;

fn documentation() -> SourceId {
    SourceId::new("documentation".to_string())
}

fn doc(slug: &str, kind: ContentKind) -> CreateContentParams {
    let mut params = content_params(slug, &documentation());
    params.kind = kind;
    params
}

async fn seed(db: &TempDb, params: &CreateContentParams) -> String {
    ContentRepository::new(Arc::clone(&db.pool))
        .create(params)
        .await
        .expect("seed a documentation row")
        .id
        .as_str()
        .to_owned()
}

// The provider reads its pool back out of the context as `Arc<Database>`, so
// the erased value has to be exactly that type.
fn database(db: &TempDb) -> Arc<Database> {
    Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ))
}

async fn enrich(
    db: &TempDb,
    content_id: &str,
) -> Result<serde_json::Value, systemprompt::traits::ProviderError> {
    let pool = database(db);
    let ctx = ContentDataContext::new(content_id, "documentation", &pool);
    let mut item = serde_json::json!({ "slug": "seeded" });
    DocsContentDataProvider::new()
        .enrich_content(&ctx, &mut item)
        .await?;
    Ok(item)
}

fn child_slugs(item: &serde_json::Value) -> Vec<String> {
    item.get("children")
        .and_then(|c| c.as_array())
        .expect("an index page carries a children array")
        .iter()
        .map(|c| {
            c.get("slug")
                .and_then(|s| s.as_str())
                .expect("every child names its slug")
                .to_owned()
        })
        .collect()
}

#[tokio::test]
async fn the_provider_declares_the_documentation_source_it_enriches() {
    let provider = DocsContentDataProvider::new();

    assert_eq!(provider.provider_id(), "docs-content-enricher");
    assert_eq!(provider.applies_to_sources(), vec!["documentation"]);
}

#[tokio::test]
async fn enrichment_splices_the_stored_relations_onto_the_item() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let params = doc("relations", ContentKind::Docs)
        .with_after_reading_this(vec!["Run the pipeline".to_owned()])
        .with_related_playbooks(vec![ContentLinkMetadata {
            title: "Playbook".to_owned(),
            url: "/documentation/playbook".to_owned(),
        }])
        .with_related_code(vec![ContentLinkMetadata {
            title: "Source".to_owned(),
            url: "https://example.test/src".to_owned(),
        }]);
    let content_id = seed(&db, &params).await;

    let item = enrich(&db, &content_id).await.expect("enrich the item");

    assert_eq!(
        item["after_reading_this"],
        serde_json::json!(["Run the pipeline"])
    );
    assert_eq!(item["related_playbooks"][0]["title"], "Playbook");
    assert_eq!(item["related_code"][0]["url"], "https://example.test/src");
    assert!(
        item.get("children").is_none(),
        "an ordinary docs page lists no children"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn enrichment_defaults_absent_relations_to_empty_arrays() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_id = seed(&db, &doc("bare", ContentKind::Docs)).await;

    let item = enrich(&db, &content_id).await.expect("enrich the item");

    for key in ["after_reading_this", "related_playbooks", "related_code"] {
        assert_eq!(
            item[key],
            serde_json::json!([]),
            "{key} is an empty array, not null, when nothing was stored"
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn an_index_page_lists_every_top_level_sibling() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let index = seed(&db, &doc("index", ContentKind::DocsIndex)).await;
    for slug in ["alpha", "beta", "alpha/nested"] {
        seed(&db, &doc(slug, ContentKind::Docs)).await;
    }

    let item = enrich(&db, &index).await.expect("enrich the index page");

    let mut slugs = child_slugs(&item);
    slugs.sort();
    assert_eq!(
        slugs,
        vec!["alpha", "beta"],
        "the root listing takes only slugs with no separator, and excludes itself"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_index_page_with_no_siblings_lists_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let index = seed(&db, &doc("index", ContentKind::DocsIndex)).await;

    let item = enrich(&db, &index).await.expect("enrich the index page");

    assert!(child_slugs(&item).is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn a_nested_list_page_lists_only_its_immediate_children() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let parent = seed(&db, &doc("guides", ContentKind::DocsList)).await;
    for slug in [
        "guides/first",
        "guides/second",
        "guides/first/deeper",
        "other/unrelated",
    ] {
        seed(&db, &doc(slug, ContentKind::Docs)).await;
    }

    let item = enrich(&db, &parent).await.expect("enrich the list page");

    let mut slugs = child_slugs(&item);
    slugs.sort();
    assert_eq!(
        slugs,
        vec!["guides/first", "guides/second"],
        "one level down only — a grandchild and a foreign subtree are both excluded"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn children_carry_the_documentation_url_built_from_their_slug() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let index = seed(&db, &doc("index", ContentKind::DocsIndex)).await;
    seed(&db, &doc("governance", ContentKind::Docs)).await;

    let item = enrich(&db, &index).await.expect("enrich the index page");

    let child = &item["children"][0];
    assert_eq!(child["url"], "/documentation/governance");
    assert_eq!(child["title"], "Title for governance");
    assert_eq!(child["description"], "Description for governance");

    db.cleanup().await;
}

#[tokio::test]
async fn enrichment_reports_a_content_id_that_matches_no_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let error = enrich(&db, "content_does_not_exist")
        .await
        .expect_err("an unknown content id cannot be enriched");

    assert!(
        error.to_string().contains("content_does_not_exist"),
        "the error names the id it could not resolve: {error}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn enrichment_reports_a_context_carrying_no_database() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let absent = ();
    let ctx = ContentDataContext::new("content_any", "documentation", &absent);
    let mut item = serde_json::json!({});

    let error = DocsContentDataProvider::new()
        .enrich_content(&ctx, &mut item)
        .await
        .expect_err("without a pool there is nothing to enrich from");

    assert!(
        !error.to_string().is_empty(),
        "the missing pool surfaces as an error rather than an unenriched item"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn get_children_static_is_the_same_listing_the_enricher_uses() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db, &doc("index", ContentKind::DocsIndex)).await;
    seed(&db, &doc("alpha", ContentKind::Docs)).await;

    let children = DocsContentDataProvider::new()
        .get_children_static(&db.pool, &documentation(), "")
        .await;

    assert_eq!(
        children.len(),
        1,
        "an empty slug is treated as the root, exactly as 'index' is"
    );
    assert_eq!(children[0].slug, "alpha");

    db.cleanup().await;
}

#[tokio::test]
async fn a_source_with_no_documentation_rows_yields_no_children() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let children = DocsContentDataProvider::new()
        .get_children_static(&db.pool, &SourceId::new("empty".to_string()), "index")
        .await;

    assert!(children.is_empty());

    db.cleanup().await;
}
