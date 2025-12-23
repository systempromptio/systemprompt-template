use serial_test::serial;
use systemprompt_admin::tools::users::repository::UsersRepository;

use super::super::common::TestDb;

#[tokio::test]
#[serial]
async fn list_users_returns_valid_data() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = UsersRepository::new(db.db_pool())?;

    let users = repo.list_users(None).await?;

    for user in &users {
        assert!(!user.id.is_empty());
        assert!(!user.status.is_empty());
        assert!(user.total_sessions >= 0);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn list_users_limits_results() -> anyhow::Result<()> {
    let db = TestDb::new().await?;
    let repo = UsersRepository::new(db.db_pool())?;

    let users = repo.list_users(None).await?;

    assert!(users.len() <= 100);
    Ok(())
}
