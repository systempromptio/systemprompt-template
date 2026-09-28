//! The validity window on a rule row: `access_control_rule_validity`, the
//! side table that bounds a core-owned `access_control_rules` row in time.
//!
//! Core's resolver never reads this table, so a window binds through the
//! hourly expiry sweep ([`delete_expired_rules`]) rather than at decision
//! time. Everything that writes a rule — the console upsert, the
//! `rules.yaml` sync — records its window here in the same transaction.

use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool};

// Why: `None` removes the window, so a rule edited back to open-ended does not
// keep an old expiry the sweep would still act on.
pub async fn set_rule_validity<'e, E: PgExecutor<'e>>(
    exec: E,
    rule_id: &str,
    valid_until: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    match valid_until {
        Some(until) => {
            sqlx::query!(
                "INSERT INTO access_control_rule_validity (rule_id, valid_until)
                 VALUES ($1, $2)
                 ON CONFLICT (rule_id) DO UPDATE
                    SET valid_until = EXCLUDED.valid_until, updated_at = NOW()",
                rule_id,
                until
            )
            .execute(exec)
            .await?;
        },
        None => {
            sqlx::query!(
                "DELETE FROM access_control_rule_validity WHERE rule_id = $1",
                rule_id
            )
            .execute(exec)
            .await?;
        },
    }
    Ok(())
}

// Why: the sweep's half. Deleting the rule cascades the validity row away,
// and `updated_at` on the survivors is untouched — core's parent-chain cache
// keys on COUNT + MAX(updated_at), and the count moving is enough.
pub async fn delete_expired_rules(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM access_control_rules r
          USING access_control_rule_validity v
          WHERE v.rule_id = r.id AND v.valid_until <= CURRENT_TIMESTAMP"
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
