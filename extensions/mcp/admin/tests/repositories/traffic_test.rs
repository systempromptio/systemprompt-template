use serial_test::serial;
use systemprompt_admin::tools::traffic::repository::TrafficRepository;

use super::super::common::TestDb;

#[tokio::test]
#[serial]
async fn get_traffic_summary_returns_valid_data() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = TrafficRepository::new(db.db_pool())?;

    let summary = repo.get_traffic_summary(30).await?;

    assert!(summary.total_sessions >= 0);
    assert!(summary.total_requests >= 0);
    assert!(summary.unique_users >= 0);
    assert!(summary.avg_session_duration_secs >= 0.0);
    assert!(summary.avg_requests_per_session >= 0.0);
    assert!(summary.total_cost_cents >= 0);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_device_breakdown_calculates_percentages() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = TrafficRepository::new(db.db_pool())?;

    let breakdown = repo.get_device_breakdown_with_trends(30).await?;

    let total_percentage: f64 = breakdown.iter().map(|d| d.percentage).sum();
    if !breakdown.is_empty() {
        assert!(
            total_percentage <= 100.1,
            "Total percentage {} should be <= 100",
            total_percentage
        );
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_geographic_breakdown_limits_results() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = TrafficRepository::new(db.db_pool())?;

    let breakdown = repo.get_geographic_breakdown(30).await?;

    assert!(breakdown.len() <= 20);
    for item in &breakdown {
        assert!(!item.country.is_empty());
        assert!(item.sessions >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_browser_breakdown_handles_results() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = TrafficRepository::new(db.db_pool())?;

    let breakdown = repo.get_browser_breakdown(30).await?;

    assert!(breakdown.len() <= 10);
    for item in &breakdown {
        assert!(!item.browser.is_empty());
        assert!(item.sessions >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_os_breakdown_limits_to_10() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = TrafficRepository::new(db.db_pool())?;

    let breakdown = repo.get_os_breakdown(30).await?;

    assert!(breakdown.len() <= 10);
    for item in &breakdown {
        assert!(!item.os.is_empty());
        assert!(item.sessions >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_normalized_referrers_returns_valid_data() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = TrafficRepository::new(db.db_pool())?;

    let referrers = repo.get_normalized_referrers(30).await?;

    assert!(referrers.len() <= 20);
    for referrer in &referrers {
        assert!(referrer.sessions >= 0);
        assert!(referrer.unique_visitors >= 0);
        assert!(referrer.avg_pages_per_session >= 0.0);
        assert!(referrer.avg_duration_sec >= 0.0);
    }
    Ok(())
}
