use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::secrets_grp::{
    secret_crypto, secret_keys, secret_migration,
};
use systemprompt_web_shared::error::MarketplaceError;

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
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(execute_inner(ctx).await?)
    }
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let start = std::time::Instant::now();

    let Ok(master_key) = secret_crypto::load_master_key() else {
        return Ok(JobResult::success().with_stats(0, 0).with_duration(0));
    };

    let db = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
        "Database not available in job context".to_owned(),
    ))?;

    let pool = db.pool().ok_or(MarketplaceError::Internal(
        "PgPool not available from database".to_owned(),
    ))?;

    let actor_user = &ctx.actor().user_id;

    let rows = secret_migration::fetch_unencrypted_secrets(pool.as_ref())
        .await
        .map_err(MarketplaceError::Database)?;

    if rows.is_empty() {
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        return Ok(JobResult::success()
            .with_stats(0, 0)
            .with_duration(duration_ms));
    }

    let (success_count, error_count) = migrate_secrets(&pool, &rows, &master_key, actor_user).await;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    tracing::info!(
        actor = %actor_user.as_str(),
        migrated = success_count,
        errors = error_count,
        duration_ms,
        "Secret migration job completed"
    );

    Ok(JobResult::success()
        .with_stats(success_count, error_count)
        .with_duration(duration_ms))
}

async fn migrate_secrets(
    pool: &std::sync::Arc<sqlx::PgPool>,
    rows: &[secret_migration::UnencryptedSecret],
    master_key: &[u8; 32],
    actor: &UserId,
) -> (u64, u64) {
    let mut success_count = 0u64;
    let mut error_count = 0u64;

    for row in rows {
        let result = encrypt_and_store_secret(pool, row, master_key, actor).await;

        match result {
            Ok(()) => {
                success_count += 1;
                tracing::debug!(id = %row.id, user_id = %row.user_id, var_name = %row.var_name, "Migrated secret");
            },
            Err(e) => {
                error_count += 1;
                tracing::warn!(id = %row.id, user_id = %row.user_id, error = %e, "Failed to migrate secret");
            },
        }
    }

    (success_count, error_count)
}

async fn encrypt_and_store_secret(
    pool: &std::sync::Arc<sqlx::PgPool>,
    row: &secret_migration::UnencryptedSecret,
    master_key: &[u8; 32],
    actor: &UserId,
) -> Result<(), MarketplaceError> {
    let dek = secret_keys::get_or_create_user_dek(pool, &UserId::new(&row.user_id), master_key)
        .await
        .map_err(|e| MarketplaceError::Crypto(format!("DEK error: {e}")))?;

    let nonce = secret_crypto::generate_nonce();
    let encrypted = secret_crypto::encrypt(&dek, &nonce, row.var_value.as_bytes())
        .map_err(|e| MarketplaceError::Crypto(format!("Encryption error: {e}")))?;

    let key_version = secret_migration::fetch_key_version(pool.as_ref(), &row.user_id).await;

    secret_migration::update_encrypted_value(
        pool.as_ref(),
        &row.id,
        &encrypted,
        nonce.as_slice(),
        key_version,
    )
    .await?;

    if let Err(e) =
        secret_migration::insert_migration_audit(pool.as_ref(), &row.user_id, &row.var_name, actor)
            .await
    {
        tracing::warn!(error = %e, user_id = %row.user_id, var_name = %row.var_name, "failed to record secret migration audit");
    }

    Ok(())
}

systemprompt::traits::submit_job!(&SecretMigrationJob);
