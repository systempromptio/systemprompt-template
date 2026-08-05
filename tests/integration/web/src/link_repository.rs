//! `LinkRepository` against the real schema.

use std::sync::Arc;

use systemprompt::identifiers::CampaignId;
use systemprompt_web_content::repository::{ContentRepository, LinkRepository};

use crate::fixtures::{content_params, link_params, source_id};
use crate::tempdb::TempDb;

#[tokio::test]
async fn create_link_returns_the_persisted_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = LinkRepository::new(Arc::clone(&db.pool));

    let created = repo
        .create_link(
            &link_params("abc123")
                .with_link_text(Some("Read the guide".to_string()))
                .with_campaign_name(Some("launch".to_string())),
        )
        .await
        .expect("create link");

    assert_eq!(created.short_code, "abc123");
    assert_eq!(created.target_url, "https://example.com/abc123");
    assert_eq!(created.link_type, "cta");
    assert_eq!(created.link_text.as_deref(), Some("Read the guide"));
    assert_eq!(created.campaign_name.as_deref(), Some("launch"));
    assert_eq!(created.is_active, Some(true));
    assert_eq!(
        created.click_count,
        Some(0),
        "a fresh link starts with no clicks recorded"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn get_link_by_short_code_finds_an_active_link() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = LinkRepository::new(Arc::clone(&db.pool));

    let created = repo
        .create_link(&link_params("hit001"))
        .await
        .expect("create link");

    let found = repo
        .get_link_by_short_code("hit001")
        .await
        .expect("look up by short code")
        .expect("an active link resolves");

    assert_eq!(found.id, created.id);

    db.cleanup().await;
}

#[tokio::test]
async fn get_link_by_short_code_misses_unknown_and_inactive_codes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = LinkRepository::new(Arc::clone(&db.pool));

    repo.create_link(&link_params("off001").with_is_active(false))
        .await
        .expect("create a deactivated link");

    let unknown = repo
        .get_link_by_short_code("nosuch")
        .await
        .expect("look up an unknown short code");
    assert!(unknown.is_none(), "an unknown short code reads as None");

    let inactive = repo
        .get_link_by_short_code("off001")
        .await
        .expect("look up a deactivated short code");
    assert!(
        inactive.is_none(),
        "the lookup filters on is_active, so a deactivated link is not resolvable"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_links_by_campaign_returns_only_that_campaign() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = LinkRepository::new(Arc::clone(&db.pool));
    let campaign = CampaignId::new("spring-2026".to_string());
    let other = CampaignId::new("autumn-2026".to_string());

    for code in ["camp01", "camp02"] {
        repo.create_link(&link_params(code).with_campaign_id(Some(campaign.clone())))
            .await
            .expect("create a link in the campaign under test");
    }
    repo.create_link(&link_params("camp03").with_campaign_id(Some(other.clone())))
        .await
        .expect("create a link in an unrelated campaign");

    let listed = repo
        .list_links_by_campaign(&campaign)
        .await
        .expect("list links by campaign");
    let mut codes: Vec<&str> = listed.iter().map(|l| l.short_code.as_str()).collect();
    codes.sort_unstable();

    assert_eq!(codes, vec!["camp01", "camp02"]);

    let empty = repo
        .list_links_by_campaign(&CampaignId::new("never-ran".to_string()))
        .await
        .expect("list links for a campaign with none");
    assert!(empty.is_empty(), "a campaign with no links lists empty");

    db.cleanup().await;
}

#[tokio::test]
async fn list_links_by_source_content_returns_only_that_content() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_repo = ContentRepository::new(Arc::clone(&db.pool));
    let repo = LinkRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let article = content_repo
        .create(&content_params("article", &source))
        .await
        .expect("create the linking article");
    let unrelated = content_repo
        .create(&content_params("unrelated", &source))
        .await
        .expect("create an unrelated article");

    repo.create_link(&link_params("src001").with_source_content_id(Some(article.id.clone())))
        .await
        .expect("create a link from the article");
    repo.create_link(&link_params("src002").with_source_content_id(Some(unrelated.id.clone())))
        .await
        .expect("create a link from the unrelated article");

    let listed = repo
        .list_links_by_source_content(&article.id)
        .await
        .expect("list links by source content");

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].short_code, "src001");
    assert_eq!(listed[0].source_content_id.as_ref(), Some(&article.id));

    db.cleanup().await;
}
