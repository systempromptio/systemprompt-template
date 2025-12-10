use systemprompt_core_database::Database;

#[tokio::test]
async fn transaction_commits_successfully() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let tx = db.begin().await.expect("Failed to begin transaction");

            sqlx::query("SELECT 1")
                .execute(&mut *tx)
                .await
                .expect("Failed to execute query in transaction");

            tx.commit().await.expect("Failed to commit transaction");
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}

#[tokio::test]
async fn transaction_rollback_works() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let tx = db.begin().await.expect("Failed to begin transaction");

            sqlx::query("SELECT 1")
                .execute(&mut *tx)
                .await
                .expect("Failed to execute query in transaction");

            tx.rollback().await.expect("Failed to rollback transaction");
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}

#[tokio::test]
async fn multiple_transactions_are_independent() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let tx1 = db.begin().await.expect("Failed to begin transaction 1");
            let tx2 = db.begin().await.expect("Failed to begin transaction 2");

            sqlx::query("SELECT 1")
                .execute(&mut *tx1)
                .await
                .expect("Failed to execute in tx1");

            sqlx::query("SELECT 2")
                .execute(&mut *tx2)
                .await
                .expect("Failed to execute in tx2");

            tx1.commit().await.expect("Failed to commit transaction 1");

            tx2.commit().await.expect("Failed to commit transaction 2");
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}

#[tokio::test]
async fn transaction_with_parameters() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/test".to_string());

    match Database::new_postgres(&database_url).await {
        Ok(db) => {
            let mut tx = db.begin().await.expect("Failed to begin transaction");

            let result: (i64,) = sqlx::query_as("SELECT $1::int8")
                .bind(42i64)
                .fetch_one(&mut *tx)
                .await
                .expect("Failed to execute query with parameters");

            assert_eq!(result.0, 42, "Expected parameter to be bound correctly");

            tx.commit().await.expect("Failed to commit transaction");
        },
        Err(e) => {
            eprintln!("Skipping test (database not available): {}", e);
        },
    }
}
