use super::{ClientRepository, ClientSummary, ClientUsageSummary};
use anyhow::Result;
use chrono::Utc;

impl ClientRepository {
    pub async fn cleanup_inactive(&self) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM oauth_clients WHERE is_active = false")
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn cleanup_old_test(&self, days_old: u32) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(days_old));
        let result = sqlx::query!(
            "DELETE FROM oauth_clients WHERE client_id LIKE $1 AND created_at < $2",
            "test_%",
            cutoff
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn deactivate_old_test(&self, days_old: u32) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(days_old));
        let now = Utc::now();
        let result = sqlx::query!(
            "UPDATE oauth_clients SET is_active = false, updated_at = $1
             WHERE client_id LIKE $2 AND created_at < $3",
            now,
            "test_%",
            cutoff
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_unused(&self, never_used_before: i64) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::seconds(never_used_before);
        let result = sqlx::query!(
            "DELETE FROM oauth_clients WHERE last_used_at IS NULL AND created_at < $1",
            cutoff
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_stale(&self, last_used_before: i64) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::seconds(last_used_before);
        let result = sqlx::query!("DELETE FROM oauth_clients WHERE last_used_at < $1", cutoff)
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_inactive(&self) -> Result<Vec<ClientSummary>> {
        let rows = sqlx::query_as!(
            ClientSummary,
            "SELECT client_id, client_name, created_at, updated_at FROM oauth_clients
             WHERE is_active = false ORDER BY updated_at DESC"
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_old(&self, older_than_timestamp: i64) -> Result<Vec<ClientSummary>> {
        let cutoff = chrono::DateTime::<Utc>::from_timestamp(older_than_timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

        let rows = sqlx::query_as!(
            ClientSummary,
            "SELECT client_id, client_name, created_at, updated_at FROM oauth_clients
             WHERE created_at < $1 ORDER BY created_at DESC",
            cutoff
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_unused(&self, never_used_before: i64) -> Result<Vec<ClientUsageSummary>> {
        let cutoff = Utc::now() - chrono::Duration::seconds(never_used_before);
        let rows = sqlx::query_as!(
            ClientUsageSummary,
            "SELECT client_id, client_name, created_at, updated_at, last_used_at FROM \
             oauth_clients
             WHERE last_used_at IS NULL AND created_at < $1 ORDER BY created_at DESC",
            cutoff
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn list_stale(&self, last_used_before: i64) -> Result<Vec<ClientUsageSummary>> {
        let cutoff = Utc::now() - chrono::Duration::seconds(last_used_before);
        let rows = sqlx::query_as!(
            ClientUsageSummary,
            "SELECT client_id, client_name, created_at, updated_at, last_used_at FROM \
             oauth_clients
             WHERE last_used_at < $1 ORDER BY last_used_at DESC",
            cutoff
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn update_last_used(&self, client_id: &str, timestamp: i64) -> Result<()> {
        let dt = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

        sqlx::query!(
            "UPDATE oauth_clients SET last_used_at = $1 WHERE client_id = $2",
            dt,
            client_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
