use anyhow::Result;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::admin::repositories::{secret_crypto, secret_keys};

#[derive(Debug, Clone, Copy, Default)]
pub struct SecretMigrationJob;

#[async_trait::async_trait]
impl Job for SecretMigrationJob {
    fn name(&self) -> &'static str {
        "secret_migration"
    }

    fn description(&self) -> &'static str {
        "Encrypts existing plaintext secrets in plugin_env_vars"
    }

    fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let start = std::time::Instant::now();

        let Ok(master_key) = secret_crypto::load_master_key() else {
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(0));
        };

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in job context"))?;

        let pool = db
            .pool()
            .ok_or_else(|| anyhow::anyhow!("PgPool not available from database"))?;

        let rows = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT id, user_id, var_name, var_value FROM plugin_env_vars \
             WHERE is_secret = true AND (encrypted_value IS NULL OR key_version = 0) \
             AND var_value != '' LIMIT 100",
        )
        .fetch_all(pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query unencrypted secrets: {e}"))?;

        if rows.is_empty() {
            let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(duration_ms));
        }

        let mut success_count = 0u64;
        let mut error_count = 0u64;

        for (id, user_id, var_name, var_value) in &rows {
            let result: Result<()> = async {
                let dek = secret_keys::get_or_create_user_dek(&pool, user_id, &master_key)
                    .await
                    .map_err(|e| anyhow::anyhow!("DEK error: {e}"))?;

                let nonce = secret_crypto::generate_nonce();
                let encrypted = secret_crypto::encrypt(&dek, &nonce, var_value.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Encryption error: {e}"))?;

                let key_version: i32 = sqlx::query_scalar(
                    "SELECT key_version FROM user_encryption_keys WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(pool.as_ref())
                .await
                .unwrap_or(1);

                sqlx::query(
                    "UPDATE plugin_env_vars SET encrypted_value = $1, value_nonce = $2, \
                     key_version = $3, var_value = '', updated_at = NOW() WHERE id = $4",
                )
                .bind(&encrypted)
                .bind(nonce.as_slice())
                .bind(key_version)
                .bind(id)
                .execute(pool.as_ref())
                .await
                .map_err(|e| anyhow::anyhow!("Update error: {e}"))?;

                let audit_id = uuid::Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
                     VALUES ($1, $2, '', $3, 'updated', 'system')",
                )
                .bind(&audit_id)
                .bind(user_id)
                .bind(var_name)
                .execute(pool.as_ref())
                .await;

                Ok(())
            }
            .await;

            match result {
                Ok(()) => {
                    success_count += 1;
                    tracing::debug!(id = %id, user_id = %user_id, var_name = %var_name, "Migrated secret");
                }
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(id = %id, user_id = %user_id, error = %e, "Failed to migrate secret");
                }
            }
        }

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            migrated = success_count,
            errors = error_count,
            duration_ms,
            "Secret migration job completed"
        );

        Ok(JobResult::success()
            .with_stats(success_count, error_count)
            .with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&SecretMigrationJob);
