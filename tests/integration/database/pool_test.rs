use systemprompt_core_database::Database;

#[tokio::test]
async fn pool_access_returns_functional_pool() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let pool = db.pool_arc();
            assert!(pool.is_ok(), "Failed to get pool");

            let pool = pool.unwrap();
            let result: (i64,) = sqlx::query_as("SELECT 1")
                .fetch_one(pool.as_ref())
                .await
                .expect("Failed to execute query");

            assert_eq!(result.0, 1, "Expected query result to be 1");
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}

#[tokio::test]
async fn pool_can_execute_multiple_queries() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let pool = db.pool_arc().expect("Failed to get pool");

            for i in 0..5 {
                let result: (i64,) = sqlx::query_as("SELECT $1::int8")
                    .bind(i)
                    .fetch_one(pool.as_ref())
                    .await
                    .expect("Failed to execute query");

                assert_eq!(result.0, i, "Expected query result to match input");
            }
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}

#[tokio::test]
async fn pool_arc_is_cloneable() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let pool = db.pool_arc().expect("Failed to get pool");
            let pool_clone = pool.clone();

            let result1: (i64,) = sqlx::query_as("SELECT 1")
                .fetch_one(pool.as_ref())
                .await
                .expect("Failed to execute query with original pool");

            let result2: (i64,) = sqlx::query_as("SELECT 2")
                .fetch_one(pool_clone.as_ref())
                .await
                .expect("Failed to execute query with cloned pool");

            assert_eq!(result1.0, 1);
            assert_eq!(result2.0, 2);
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}
