mod client_tests;
mod token_tests;
mod webauthn_tests;

use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn setup_test_pool() -> sqlx::PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/systemprompt_test".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}
