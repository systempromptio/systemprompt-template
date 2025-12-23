use serial_test::serial;
use systemprompt_admin::tools::dashboard::repository::DashboardRepository;

use super::super::common::TestDb;

#[tokio::test]
#[serial]
async fn get_conversation_metrics_returns_valid_structure() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = DashboardRepository::new(db.db_pool())?;

    let metrics = repo.get_conversation_metrics().await?;

    assert!(metrics.conversations_24h >= 0);
    assert!(metrics.conversations_7d >= 0);
    assert!(metrics.conversations_30d >= 0);
    assert!(metrics.conversations_prev_24h >= 0);
    assert!(metrics.conversations_prev_7d >= 0);
    assert!(metrics.conversations_prev_30d >= 0);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_recent_conversations_respects_limit() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = DashboardRepository::new(db.db_pool())?;

    let conversations = repo.get_recent_conversations(5).await?;

    assert!(conversations.len() <= 5);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_traffic_summary_returns_valid_structure() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = DashboardRepository::new(db.db_pool())?;

    let summary = repo.get_traffic_summary(7).await?;

    assert!(summary.total_sessions >= 0);
    assert!(summary.total_requests >= 0);
    assert!(summary.unique_users >= 0);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_conversation_trends_groups_by_date() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = DashboardRepository::new(db.db_pool())?;

    let trends = repo.get_conversation_trends(7).await?;

    for trend in &trends {
        assert!(!trend.date.is_empty());
        assert!(trend.conversations >= 0);
        assert!(trend.tool_executions >= 0);
        assert!(trend.active_users >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_tool_usage_data_returns_valid_structure() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = DashboardRepository::new(db.db_pool())?;

    let usage = repo.get_tool_usage_data().await?;

    for agent in &usage.agent_data {
        assert!(!agent.agent_name.is_empty());
        assert!(agent.hours_24 >= 0);
        assert!(agent.days_7 >= 0);
        assert!(agent.days_30 >= 0);
    }

    for tool in &usage.tool_data {
        assert!(!tool.tool_name.is_empty());
        assert!(tool.hours_24 >= 0);
        assert!(tool.days_7 >= 0);
        assert!(tool.days_30 >= 0);
    }
    Ok(())
}
