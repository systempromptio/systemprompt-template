use serial_test::serial;
use systemprompt_admin::tools::content::repository::ContentRepository;

use super::super::common::TestDb;

#[tokio::test]
#[serial]
async fn get_daily_views_per_content_returns_valid_data() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ContentRepository::new(db.db_pool())?;

    let views = repo.get_daily_views_per_content(30).await?;

    for view in &views {
        assert!(!view.content_id.is_empty());
        assert!(view.daily_views >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_traffic_summary_returns_valid_structure() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ContentRepository::new(db.db_pool())?;

    let summary = repo.get_traffic_summary().await?;

    assert!(summary.traffic_1d >= 0);
    assert!(summary.traffic_7d >= 0);
    assert!(summary.traffic_30d >= 0);
    assert!(summary.prev_traffic_1d >= 0);
    assert!(summary.prev_traffic_7d >= 0);
    assert!(summary.prev_traffic_30d >= 0);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_top_content_by_7d_respects_limit() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ContentRepository::new(db.db_pool())?;

    let content = repo.get_top_content_by_7d(10).await?;

    assert!(content.len() <= 10);
    for item in &content {
        assert!(!item.content_id.is_empty());
        assert!(item.visitors_7d >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_normalized_referrers_returns_valid_data() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ContentRepository::new(db.db_pool())?;

    let referrers = repo.get_normalized_referrers(30).await?;

    assert!(referrers.len() <= 20);
    for referrer in &referrers {
        assert!(referrer.sessions >= 0);
        assert!(referrer.unique_visitors >= 0);
    }
    Ok(())
}
