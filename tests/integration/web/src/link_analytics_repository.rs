//! `LinkAnalyticsRepository` against the real schema: click rows, the
//! counters they maintain, and the rollups read back off them.

use std::sync::Arc;

use systemprompt::identifiers::{CampaignId, LinkId, SessionId};
use systemprompt_web_content::repository::{
    ContentRepository, LinkAnalyticsRepository, LinkRepository,
};
use systemprompt_web_shared::models::TrackClickParams;

use crate::fixtures::{content_params, link_params, source_id};
use crate::tempdb::TempDb;

fn session(id: &str) -> SessionId {
    SessionId::new(id.to_string())
}

#[tokio::test]
async fn track_click_records_the_click_and_bumps_the_total() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));

    let link = links
        .create_link(&link_params("trk001"))
        .await
        .expect("create link");

    analytics
        .track_click(
            &TrackClickParams::new(link.id.clone(), session("sess-a"))
                .with_ip_address(Some("203.0.113.9".to_string()))
                .with_user_agent(Some("integration-test".to_string()))
                .with_referrer_page(Some("/guides/intro".to_string())),
        )
        .await
        .expect("track click");

    let clicks = analytics
        .get_clicks_by_link(&link.id, 10, 0)
        .await
        .expect("read back the click rows");
    assert_eq!(clicks.len(), 1);
    assert_eq!(clicks[0].session_id, session("sess-a"));
    assert_eq!(clicks[0].ip_address.as_deref(), Some("203.0.113.9"));
    assert_eq!(clicks[0].referrer_page.as_deref(), Some("/guides/intro"));

    let refreshed = links
        .get_link_by_short_code("trk001")
        .await
        .expect("re-read the link")
        .expect("the link is still active");
    assert_eq!(
        refreshed.click_count,
        Some(1),
        "track_click bumps click_count in the same transaction as the insert"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn check_session_clicked_link_flips_once_the_session_has_clicked() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));

    let link = links
        .create_link(&link_params("dedupe1"))
        .await
        .expect("create link");
    let sess = session("sess-dedupe");

    let before = analytics
        .check_session_clicked_link(&link.id, &sess)
        .await
        .expect("check before any click");
    assert!(!before, "a session that has not clicked reads false");

    analytics
        .track_click(&TrackClickParams::new(link.id.clone(), sess.clone()))
        .await
        .expect("track the first click");

    let after = analytics
        .check_session_clicked_link(&link.id, &sess)
        .await
        .expect("check after the click");
    assert!(after, "the same session clicking again is no longer unique");

    db.cleanup().await;
}

#[tokio::test]
async fn check_session_clicked_link_is_scoped_to_session_and_link() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));

    let clicked = links
        .create_link(&link_params("scope01"))
        .await
        .expect("create the clicked link");
    let untouched = links
        .create_link(&link_params("scope02"))
        .await
        .expect("create an unclicked link");

    analytics
        .track_click(&TrackClickParams::new(
            clicked.id.clone(),
            session("sess-one"),
        ))
        .await
        .expect("track click");

    let other_session = analytics
        .check_session_clicked_link(&clicked.id, &session("sess-two"))
        .await
        .expect("check a different session");
    assert!(
        !other_session,
        "another session has not clicked this link yet"
    );

    let other_link = analytics
        .check_session_clicked_link(&untouched.id, &session("sess-one"))
        .await
        .expect("check a different link");
    assert!(
        !other_link,
        "the same session has not clicked the other link"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn increment_link_clicks_counts_a_first_click_as_unique() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));

    let link = links
        .create_link(&link_params("inc001"))
        .await
        .expect("create link");

    analytics
        .increment_link_clicks(&link.id, true)
        .await
        .expect("increment for a first click");

    let after = links
        .get_link_by_short_code("inc001")
        .await
        .expect("re-read the link")
        .expect("the link is still active");
    assert_eq!(after.click_count, Some(1));
    assert_eq!(
        after.unique_click_count,
        Some(1),
        "a first click raises both counters"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn increment_link_clicks_counts_a_repeat_click_only_once() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));

    let link = links
        .create_link(&link_params("inc002"))
        .await
        .expect("create link");

    analytics
        .increment_link_clicks(&link.id, true)
        .await
        .expect("increment for a first click");
    analytics
        .increment_link_clicks(&link.id, false)
        .await
        .expect("increment for a repeat click");

    let after = links
        .get_link_by_short_code("inc002")
        .await
        .expect("re-read the link")
        .expect("the link is still active");
    assert_eq!(after.click_count, Some(2));
    assert_eq!(
        after.unique_click_count,
        Some(1),
        "a repeat click raises the total but not the unique count"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn get_link_performance_derives_the_conversion_rate() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));

    let link = links
        .create_link(&link_params("perf001"))
        .await
        .expect("create link");

    let fresh = analytics
        .get_link_performance(&link.id)
        .await
        .expect("read performance")
        .expect("a created link has a performance row");
    assert_eq!(fresh.click_count, 0);
    assert_eq!(
        fresh.conversion_rate,
        Some(0.0),
        "with no clicks the rate is defined as zero rather than a division by zero"
    );

    analytics
        .increment_link_clicks(&link.id, true)
        .await
        .expect("increment clicks");

    let after = analytics
        .get_link_performance(&link.id)
        .await
        .expect("re-read performance")
        .expect("the performance row is still present");
    assert_eq!(after.click_count, 1);
    assert_eq!(after.unique_click_count, 1);

    let missing = analytics
        .get_link_performance(&LinkId::new("no-such-link".to_string()))
        .await
        .expect("read performance for an unknown link");
    assert!(missing.is_none(), "an unknown link reads as None");

    db.cleanup().await;
}

#[tokio::test]
async fn get_campaign_performance_rolls_up_the_campaign_links() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));
    let campaign = CampaignId::new("rollup-2026".to_string());

    let first = links
        .create_link(&link_params("roll01").with_campaign_id(Some(campaign.clone())))
        .await
        .expect("create the first campaign link");
    links
        .create_link(&link_params("roll02").with_campaign_id(Some(campaign.clone())))
        .await
        .expect("create the second campaign link");
    links
        .create_link(&link_params("roll03"))
        .await
        .expect("create a link outside the campaign");

    analytics
        .increment_link_clicks(&first.id, true)
        .await
        .expect("increment clicks");
    analytics
        .increment_link_clicks(&first.id, false)
        .await
        .expect("increment clicks again");

    let perf = analytics
        .get_campaign_performance(&campaign)
        .await
        .expect("read campaign performance")
        .expect("a campaign with links has a rollup row");
    assert_eq!(perf.campaign_id, campaign);
    assert_eq!(perf.link_count, 2, "only the campaign's own links count");
    assert_eq!(perf.total_clicks, 2);

    let empty = analytics
        .get_campaign_performance(&CampaignId::new("never-ran".to_string()))
        .await
        .expect("read performance for a campaign with no links");
    assert!(
        empty.is_none(),
        "the rollup groups by campaign, so a campaign with no links has no row"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn get_content_journey_map_ranks_clicked_links_from_content() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_repo = ContentRepository::new(Arc::clone(&db.pool));
    let links = LinkRepository::new(Arc::clone(&db.pool));
    let analytics = LinkAnalyticsRepository::new(Arc::clone(&db.pool));
    let source = source_id();

    let article = content_repo
        .create(&content_params("journey", &source))
        .await
        .expect("create the source article");

    let popular = links
        .create_link(&link_params("jrn001").with_source_content_id(Some(article.id.clone())))
        .await
        .expect("create the popular link");
    let quiet = links
        .create_link(&link_params("jrn002").with_source_content_id(Some(article.id.clone())))
        .await
        .expect("create the link nobody clicks");

    for _ in 0..2 {
        analytics
            .increment_link_clicks(&popular.id, false)
            .await
            .expect("increment clicks on the popular link");
    }

    let map = analytics
        .get_content_journey_map(10, 0)
        .await
        .expect("read the journey map");
    assert_eq!(
        map.len(),
        1,
        "the map only includes links that have been clicked"
    );
    assert_eq!(map[0].source_content_id, article.id);
    assert_eq!(map[0].target_url, popular.target_url);
    assert_eq!(map[0].click_count, 2);

    let journey = analytics
        .get_content_journey(&article.id)
        .await
        .expect("read the per-content journey");
    assert_eq!(
        journey.len(),
        2,
        "the per-content journey counts click rows and keeps unclicked links at zero"
    );
    let quiet_node = journey
        .iter()
        .find(|node| node.target_url == quiet.target_url)
        .expect("the unclicked link still appears");
    assert_eq!(quiet_node.click_count, 0);

    db.cleanup().await;
}
