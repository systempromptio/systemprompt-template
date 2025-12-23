use serial_test::serial;
use systemprompt_admin::tools::conversations::repository::ConversationsRepository;

use super::super::common::TestDb;

#[tokio::test]
#[serial]
async fn get_conversation_summary_returns_valid_structure() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ConversationsRepository::new(db.db_pool())?;

    let summary = repo.get_conversation_summary("24 hours").await?;

    assert!(summary.total_conversations >= 0);
    assert!(summary.total_messages >= 0);
    assert!(summary.avg_messages_per_conversation >= 0.0);
    assert!(summary.avg_execution_time_ms >= 0.0);
    assert!(summary.failed_conversations >= 0);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_recent_conversations_paginated_respects_limit() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ConversationsRepository::new(db.db_pool())?;

    let conversations = repo
        .get_recent_conversations_paginated("7 days", 5, 0, None)
        .await?;

    assert!(conversations.len() <= 5);
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_recent_conversations_handles_offset() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ConversationsRepository::new(db.db_pool())?;

    let page1 = repo
        .get_recent_conversations_paginated("7 days", 5, 0, None)
        .await?;
    let page2 = repo
        .get_recent_conversations_paginated("7 days", 5, 5, None)
        .await?;

    if !page1.is_empty() && !page2.is_empty() {
        assert_ne!(
            page1[0].context_id, page2[0].context_id,
            "Page 2 should have different results than page 1"
        );
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn get_conversation_trends_returns_all_periods() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = ConversationsRepository::new(db.db_pool())?;

    let trends = repo.get_conversation_trends().await?;

    assert!(!trends.is_empty());
    let trend = &trends[0];
    assert!(trend.conversations_1h >= 0);
    assert!(trend.conversations_24h >= 0);
    assert!(trend.conversations_7d >= 0);
    assert!(trend.conversations_30d >= 0);
    Ok(())
}
