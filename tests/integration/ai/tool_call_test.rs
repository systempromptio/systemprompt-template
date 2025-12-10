use serde_json::json;
use std::sync::Arc;
use systemprompt_core_ai::repository::AIRequestRepository;
use systemprompt_core_database::Database;
use systemprompt_identifiers::{AiToolCallId, UserId};

#[tokio::test]
#[ignore]
async fn tool_call_tracking() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    let db = Database::new_postgres(&database_url)
        .await
        .expect("Failed to connect to database");
    let db_pool = Arc::new(db);

    let repo = AIRequestRepository::new(db_pool);

    let user_id = UserId::new("test_user_456");
    let request = repo
        .create(
            &user_id,
            None,
            None,
            "anthropic",
            "claude-3-sonnet",
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create request");

    let tool_call_id = AiToolCallId::new("call_123");
    let tool_call = repo
        .insert_tool_call(
            request.id,
            &tool_call_id,
            "search",
            json!({"query": "test"}),
        )
        .await
        .expect("Failed to insert tool call");

    assert_eq!(tool_call.status, "pending");
    assert_eq!(tool_call.tool_name, "search");

    let calls = repo
        .get_tool_calls(request.id)
        .await
        .expect("Failed to get tool calls");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool_name, "search");
}
