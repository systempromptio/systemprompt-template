use anyhow::Result;
use sqlx::PgPool;

#[derive(Debug)]
pub struct CleanupRepository {
    pool: PgPool,
}

impl CleanupRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn delete_orphaned_logs(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM logs
            WHERE user_id IS NOT NULL
              AND user_id NOT IN (SELECT id FROM users)
            "#
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_orphaned_analytics_events(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM analytics_events
            WHERE session_id IS NOT NULL
              AND session_id NOT IN (SELECT session_id FROM user_sessions)
            "#
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_orphaned_mcp_executions(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM mcp_tool_executions
            WHERE context_id IS NOT NULL
              AND context_id NOT IN (SELECT context_id FROM user_contexts)
            "#
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_old_logs(&self, days: i32) -> Result<u64> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
        let result = sqlx::query!("DELETE FROM logs WHERE timestamp < $1", cutoff)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_expired_oauth_tokens(&self) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM oauth_refresh_tokens WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_expired_oauth_codes(&self) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM oauth_auth_codes WHERE expires_at < NOW() OR used_at IS NOT NULL"
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
