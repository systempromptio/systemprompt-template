use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug)]
struct TableRow {
    schema_name: String,
    table_name: String,
    column_count: i64,
}

#[derive(Debug)]
struct IndexRow {
    schema: String,
    table: String,
    index: String,
}

pub async fn infra_database_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let tables = fetch_tables(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list tables");
        vec![]
    });
    let indexes = fetch_indexes(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list indexes");
        vec![]
    });
    let db_size = fetch_db_size(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch db size");
        "unknown".to_string()
    });
    let db_name = fetch_db_name(&pool)
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let tables_json: Vec<serde_json::Value> = tables
        .iter()
        .map(|t| {
            json!({
                "schema": t.schema_name,
                "name": t.table_name,
                "column_count": t.column_count,
            })
        })
        .collect();
    let indexes_json: Vec<serde_json::Value> = indexes
        .iter()
        .map(|i| {
            json!({
                "schema": i.schema,
                "table": i.table,
                "name": i.index,
            })
        })
        .collect();

    let data = json!({
        "page": "infra-database",
        "title": "Infrastructure — Database",
        "cli_command": "systemprompt infra db tables",
        "db_name": db_name,
        "db_size": db_size,
        "tables": tables_json,
        "table_count": tables_json.len(),
        "has_tables": !tables_json.is_empty(),
        "indexes": indexes_json,
        "index_count": indexes_json.len(),
        "has_indexes": !indexes_json.is_empty(),
    });
    super::render_page(&engine, "infra-database", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_tables(pool: &PgPool) -> Result<Vec<TableRow>, sqlx::Error> {
    sqlx::query_as!(
        TableRow,
        r#"SELECT c.table_schema AS "schema_name!",
                  c.table_name AS "table_name!",
                  COUNT(*)::bigint AS "column_count!"
           FROM information_schema.columns c
           JOIN information_schema.tables t
             ON t.table_schema = c.table_schema AND t.table_name = c.table_name
           WHERE c.table_schema = 'public' AND t.table_type = 'BASE TABLE'
           GROUP BY c.table_schema, c.table_name
           ORDER BY c.table_name"#,
    )
    .fetch_all(pool)
    .await
}

async fn fetch_indexes(pool: &PgPool) -> Result<Vec<IndexRow>, sqlx::Error> {
    sqlx::query_as!(
        IndexRow,
        r#"SELECT schemaname::text AS "schema!",
                  tablename::text AS "table!",
                  indexname::text AS "index!"
           FROM pg_indexes
           WHERE schemaname = 'public'
           ORDER BY tablename, indexname
           LIMIT 200"#,
    )
    .fetch_all(pool)
    .await
}

async fn fetch_db_size(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT pg_size_pretty(pg_database_size(current_database())) AS "size!""#
    )
    .fetch_one(pool)
    .await
}

async fn fetch_db_name(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT current_database() AS "name!""#)
        .fetch_one(pool)
        .await
}
