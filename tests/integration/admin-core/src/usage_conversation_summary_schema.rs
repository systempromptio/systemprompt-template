//! Pin the production installer's structural → migration → index ordering.

use std::sync::Arc;

use systemprompt::ExtensionRegistry;
use systemprompt::database::{Database, install_extension_schemas};

use crate::tempdb::TempDb;

#[tokio::test]
async fn existing_transcripts_gain_search_before_the_declarative_index() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let fresh_index: bool =
        sqlx::query_scalar("SELECT to_regclass('idx_session_transcripts_fts') IS NOT NULL")
            .fetch_one(&*db.pool)
            .await
            .expect("fresh declarative install creates the search index");
    assert!(fresh_index);

    // Model a populated pre-059 database. Do not execute migration SQL here:
    // the production installer must discover and run it before its index phase.
    sqlx::query("ALTER TABLE session_transcripts DROP COLUMN search_tsv CASCADE")
        .execute(&*db.pool)
        .await
        .expect("restore pre-search transcript shape");
    sqlx::query("DELETE FROM extension_migrations WHERE extension_id = 'web' AND version = 59")
        .execute(&*db.pool)
        .await
        .expect("mark only migration 059 pending");
    sqlx::query(
        "INSERT INTO session_transcripts (id, user_id, session_id, transcript)
         VALUES ('upgrade-preservation', 'upgrade-owner', 'upgrade-session',
                 '[{\"text\":\"migration preservation\"}]'::jsonb)",
    )
    .execute(&*db.pool)
    .await
    .expect("seed an existing transcript");

    let database = Database::from_pools(Arc::clone(&db.pool), Some(Arc::clone(&db.pool)));
    let registry = ExtensionRegistry::discover().expect("discover the production extensions");
    install_extension_schemas(&registry, database.write())
        .await
        .expect("upgrade runs pending migration before dependent indexes");

    let restored: bool = sqlx::query_scalar(
        "SELECT to_regclass('idx_session_transcripts_fts') IS NOT NULL
         AND EXISTS (SELECT 1 FROM extension_migrations
                     WHERE extension_id = 'web' AND version = 59)
         AND EXISTS (SELECT 1 FROM session_transcripts
                     WHERE id = 'upgrade-preservation'
                       AND transcript = '[{\"text\":\"migration preservation\"}]'::jsonb
                       AND search_tsv @@ plainto_tsquery('english', 'preservation'))",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("query upgraded transcript and index");
    assert!(
        restored,
        "upgrade preserves the transcript and builds searchable text"
    );
}
