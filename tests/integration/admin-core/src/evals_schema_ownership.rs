//! Regression for the 0.29.0 eval-table ownership move: core's
//! `systemprompt-evaluation` extension owns the eval spine and the web
//! extension only declares it via `cross_extension_tables`. A duplicate
//! `SchemaDefinition` on either side fails installation with
//! `DuplicateTableOwner`, so `TempDb::create()` succeeding — it installs the
//! full real schema — is itself the assertion; the checks below pin the
//! resulting shape.

use crate::tempdb::TempDb;

const EVAL_TABLES: [&str; 6] = [
    "eval_runs",
    "eval_cases",
    "eval_results",
    "eval_pairs",
    "eval_judge_calls",
    "eval_rubrics",
];

#[tokio::test]
async fn install_creates_every_eval_table_exactly_once() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    for table in EVAL_TABLES {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(db.pool.as_ref())
            .await
            .expect("regclass lookup");
        assert!(exists, "{table} must exist after a clean install");
    }
    db.cleanup().await;
}

#[tokio::test]
async fn replay_results_are_not_blocked_by_run_request_uniqueness() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    // The unique (run_id, ai_request_id) guard applies to first-pass rows
    // only; a replay row for the same request carries replay_of_result_id
    // and must insert cleanly.
    let indexdef: String = sqlx::query_scalar(
        "SELECT indexdef FROM pg_indexes WHERE indexname = 'idx_eval_results_run_request'",
    )
    .fetch_one(db.pool.as_ref())
    .await
    .expect("unique index exists");
    assert!(
        indexdef.contains("replay_of_result_id IS NULL"),
        "uniqueness must exempt replay rows: {indexdef}"
    );
    db.cleanup().await;
}
