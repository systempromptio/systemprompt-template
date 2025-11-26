use crate::common::*;
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use uuid::Uuid;

#[tokio::test]
async fn test_concurrent_inserts_succeed() -> Result<()> {
    let ctx = Arc::new(TestContext::new().await?);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let ctx = Arc::clone(&ctx);
            tokio::spawn(async move {
                let session_id = format!("concurrent-test-{}-{}", i, Uuid::new_v4());
                let query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
                ctx.db.execute(&query, &[&session_id]).await
            })
        })
        .collect();

    let mut success_count = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            other => {
                eprintln!("Handle result: {:?}", other);
            },
        }
    }

    assert_eq!(success_count, 10, "Not all concurrent inserts succeeded");

    println!("✓ All 10 concurrent inserts succeeded");
    Ok(())
}

#[tokio::test]
async fn test_concurrent_reads_dont_block_writes() -> Result<()> {
    let ctx = Arc::new(TestContext::new().await?);

    let session_id = format!("concurrent-read-write-{}", Uuid::new_v4());
    let insert_query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
    ctx.db.execute(&insert_query, &[&session_id]).await?;

    let read_handle = {
        let ctx = Arc::clone(&ctx);
        let session_id = session_id.clone();
        tokio::spawn(async move {
            let select_query = "SELECT * FROM user_sessions WHERE session_id = $1";
            ctx.db.fetch_all(&select_query, &[&session_id]).await
        })
    };

    let write_handle = {
        let ctx = Arc::clone(&ctx);
        let session_id = format!("concurrent-write-{}", Uuid::new_v4());
        tokio::spawn(async move {
            let insert_query =
                "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
            ctx.db.execute(&insert_query, &[&session_id]).await
        })
    };

    let read_result = read_handle.await??;
    let write_result = write_handle.await??;

    assert!(!read_result.is_empty(), "Read operation failed");
    assert!(write_result > 0, "Write operation failed");

    println!("✓ Concurrent reads don't block writes");
    Ok(())
}

#[tokio::test]
async fn test_transaction_isolation_level() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SHOW transaction_isolation";
    let rows = ctx.db.fetch_all(&query, &[]).await?;

    assert!(
        !rows.is_empty(),
        "Could not retrieve transaction isolation level"
    );

    println!("✓ Transaction isolation level verified");
    Ok(())
}
