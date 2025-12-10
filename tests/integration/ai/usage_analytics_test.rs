use std::sync::Arc;
use systemprompt_core_ai::repository::AIRequestRepository;
use systemprompt_core_database::Database;
use systemprompt_identifiers::UserId;

#[tokio::test]
#[ignore]
async fn usage_analytics_aggregation() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    let db = Database::new_postgres(&database_url)
        .await
        .expect("Failed to connect to database");
    let db_pool = Arc::new(db);

    let repo = AIRequestRepository::new(db_pool);

    let user_id = UserId::new("analytics_user");

    for _ in 0..5 {
        let req = repo
            .create(&user_id, None, None, "openai", "gpt-4", None, None, None)
            .await
            .expect("Failed to create request");

        repo.update_completion(req.id, 100, 50, 50, 0.01, 200)
            .await
            .expect("Failed to update completion");
    }

    let usage = repo
        .get_user_usage(&user_id, None, None)
        .await
        .expect("Failed to get user usage");
    assert_eq!(usage.request_count, 5);
    assert_eq!(usage.total_tokens, 500);

    let provider_usage = repo
        .get_provider_usage(30)
        .await
        .expect("Failed to get provider usage");
    assert!(!provider_usage.is_empty());
}
