use anyhow::Result;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

pub struct CleanupRepository {
    db_pool: DbPool,
}

impl CleanupRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn delete_orphaned_logs(&self) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteOrphanedLogs.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[]).await
    }

    pub async fn delete_orphaned_analytics_events(&self) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteOrphanedAnalyticsEvents.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[]).await
    }

    pub async fn delete_orphaned_mcp_executions(&self) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteOrphanedMcpExecutions.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[]).await
    }

    pub async fn delete_old_logs(&self) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteOldLogs.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[]).await
    }

    pub async fn delete_expired_oauth_codes(&self) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteExpiredOAuthCodes.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[]).await
    }

    pub async fn delete_expired_oauth_tokens(&self) -> Result<u64> {
        let query = DatabaseQueryEnum::DeleteExpiredOAuthTokens.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[]).await
    }
}
