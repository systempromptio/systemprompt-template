use serial_test::serial;
use systemprompt_admin::tools::logs::repository::LogsRepository;

use super::super::common::TestDb;

#[tokio::test]
#[serial]
async fn fetch_recent_logs_respects_limit() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = LogsRepository::new(db.db_pool())?;

    let logs = repo.fetch_recent_logs(0, 10, None).await?;

    assert!(logs.len() <= 10);
    Ok(())
}

#[tokio::test]
#[serial]
async fn fetch_recent_logs_handles_pagination() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = LogsRepository::new(db.db_pool())?;

    let page1 = repo.fetch_recent_logs(0, 5, None).await?;
    let page2 = repo.fetch_recent_logs(1, 5, None).await?;

    if !page1.is_empty() && !page2.is_empty() {
        assert_ne!(
            page1[0].id, page2[0].id,
            "Page 2 should have different results"
        );
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn fetch_log_stats_returns_valid_structure() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = LogsRepository::new(db.db_pool())?;

    let stats = repo.fetch_log_stats().await?;

    assert!(stats.total_logs >= 0);
    assert!(stats.error_count >= 0);
    assert!(stats.warn_count >= 0);
    assert!(stats.info_count >= 0);
    assert!(stats.unique_modules >= 0);
    assert!(stats.unique_users >= 0);
    Ok(())
}
