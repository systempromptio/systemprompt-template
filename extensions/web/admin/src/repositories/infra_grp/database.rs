use sqlx::PgPool;

#[derive(Debug)]
pub struct TableRow {
    pub schema_name: String,
    pub table_name: String,
    pub column_count: i64,
}

#[derive(Debug)]
pub struct IndexRow {
    pub schema: String,
    pub table: String,
    pub index: String,
}

pub async fn list_tables(pool: &PgPool) -> Result<Vec<TableRow>, sqlx::Error> {
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

pub async fn list_indexes(pool: &PgPool) -> Result<Vec<IndexRow>, sqlx::Error> {
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

pub async fn get_db_size(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT pg_size_pretty(pg_database_size(current_database())) AS "size!""#)
        .fetch_one(pool)
        .await
}

pub async fn get_db_name(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT current_database() AS "name!""#)
        .fetch_one(pool)
        .await
}
