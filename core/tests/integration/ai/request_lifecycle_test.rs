use std::sync::Arc;
use systemprompt_core_ai::repository::AIRequestRepository;
use systemprompt_core_database::Database;
use systemprompt_identifiers::UserId;

#[tokio::test]
#[ignore]
async fn ai_request_lifecycle() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    let db = Database::new_postgres(&database_url)
        .await
        .expect("Failed to connect to database");
    let db_pool = Arc::new(db);

    let repo = AIRequestRepository::new(db_pool);

    let user_id = UserId::new("test_user_123");

    let request = repo
        .create(
            &user_id,
            None,
            None,
            "anthropic",
            "claude-3-sonnet",
            Some("completion"),
            Some(0.7),
            Some(1000),
        )
        .await
        .expect("Failed to create request");

    assert_eq!(request.status, "pending");
    assert_eq!(request.provider, "anthropic");
    assert_eq!(request.model, "claude-3-sonnet");

    let msg = repo
        .insert_message(request.id, "user", "Hello", 0)
        .await
        .expect("Failed to insert message");
    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello");
    assert_eq!(msg.sequence_number, 0);

    let completed = repo
        .update_completion(request.id, 150, 50, 100, 0.005, 500)
        .await
        .expect("Failed to update completion");

    assert_eq!(completed.status, "completed");
    assert_eq!(completed.tokens_used, Some(150));
    assert_eq!(completed.input_tokens, Some(50));
    assert_eq!(completed.output_tokens, Some(100));
}
