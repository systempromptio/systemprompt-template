use anyhow::Result;
use systemprompt_identifiers::UserId;

use crate::models::{UserSession, UserSessionRow};

use super::UserRepository;

impl UserRepository {
    pub async fn list_sessions(&self, user_id: &UserId) -> Result<Vec<UserSession>> {
        let rows = sqlx::query_as!(
            UserSessionRow,
            r#"
            SELECT session_id, user_id, ip_address, user_agent, device_type,
                   started_at, last_activity_at, ended_at
            FROM user_sessions
            WHERE user_id = $1
            ORDER BY last_activity_at DESC
            "#,
            user_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(UserSession::from).collect())
    }

    pub async fn list_active_sessions(&self, user_id: &UserId) -> Result<Vec<UserSession>> {
        let rows = sqlx::query_as!(
            UserSessionRow,
            r#"
            SELECT session_id, user_id, ip_address, user_agent, device_type,
                   started_at, last_activity_at, ended_at
            FROM user_sessions
            WHERE user_id = $1 AND ended_at IS NULL
            ORDER BY last_activity_at DESC
            "#,
            user_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(UserSession::from).collect())
    }

    pub async fn list_recent_sessions(
        &self,
        user_id: &UserId,
        limit: i64,
    ) -> Result<Vec<UserSession>> {
        let rows = sqlx::query_as!(
            UserSessionRow,
            r#"
            SELECT session_id, user_id, ip_address, user_agent, device_type,
                   started_at, last_activity_at, ended_at
            FROM user_sessions
            WHERE user_id = $1
            ORDER BY last_activity_at DESC
            LIMIT $2
            "#,
            user_id.as_str(),
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(UserSession::from).collect())
    }
}
