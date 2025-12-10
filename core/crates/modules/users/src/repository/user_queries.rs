use anyhow::Result;
use systemprompt_identifiers::UserId;

use crate::models::{
    User, UserActivity, UserActivityRow, UserRow, UserWithSessions, UserWithSessionsRow,
};

use super::UserRepository;

impl UserRepository {
    pub async fn find_by_id(&self, id: &UserId) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE id = $1 AND status != 'deleted'
            "#,
            id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE email = $1 AND status != 'deleted'
            "#,
            email
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<User>> {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }

        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE name = $1 AND status != 'deleted'
            "#,
            name
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_by_role(&self, role: &str) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE $1 = ANY(roles) AND status != 'deleted'
            ORDER BY created_at DESC
            "#,
            role
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(User::from).collect())
    }

    pub async fn find_first_user(&self) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE status != 'deleted'
            ORDER BY created_at ASC
            LIMIT 1
            "#
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_first_admin(&self) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE 'admin' = ANY(roles) AND status != 'deleted'
            ORDER BY created_at ASC
            LIMIT 1
            "#
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn get_authenticated_user(&self, user_id: &UserId) -> Result<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE id = $1 AND status = 'active'
            "#,
            user_id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn get_with_sessions(&self, user_id: &UserId) -> Result<Option<UserWithSessions>> {
        let row = sqlx::query_as!(
            UserWithSessionsRow,
            r#"
            SELECT
                u.id, u.name, u.email, u.full_name, u.status, u.roles, u.created_at,
                COUNT(s.session_id) FILTER (WHERE s.ended_at IS NULL) as "active_sessions!",
                MAX(s.last_activity_at) as last_session_at
            FROM users u
            LEFT JOIN user_sessions s ON s.user_id = u.id
            WHERE u.id = $1 AND u.status != 'deleted'
            GROUP BY u.id
            "#,
            user_id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(UserWithSessions::from))
    }

    pub async fn get_activity(&self, user_id: &UserId) -> Result<UserActivity> {
        let row = sqlx::query_as!(
            UserActivityRow,
            r#"
            SELECT
                u.id as user_id,
                MAX(s.last_activity_at) as last_active,
                COUNT(DISTINCT s.session_id) as "session_count!",
                COUNT(DISTINCT t.task_id) as "task_count!",
                0::bigint as "message_count!"
            FROM users u
            LEFT JOIN user_sessions s ON s.user_id = u.id
            LEFT JOIN agent_tasks t ON t.user_id = u.id
            WHERE u.id = $1
            GROUP BY u.id
            "#,
            user_id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(UserActivity::from(row))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(User::from).collect())
    }

    pub async fn list_all(&self) -> Result<Vec<User>> {
        let rows = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(User::from).collect())
    }

    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<User>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, full_name, display_name, status, email_verified,
                   roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            FROM users
            WHERE status != 'deleted'
              AND (name ILIKE $1 OR email ILIKE $1 OR full_name ILIKE $1)
            ORDER BY
                CASE WHEN name ILIKE $1 THEN 0 ELSE 1 END,
                created_at DESC
            LIMIT $2
            "#,
            pattern,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(User::from).collect())
    }

    pub async fn count(&self) -> Result<i64> {
        let result = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM users WHERE status != 'deleted'"#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    pub async fn is_temporary_anonymous(&self, id: &UserId) -> Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT 'anonymous' = ANY(roles) as "is_anonymous!"
            FROM users
            WHERE id = $1
            "#,
            id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<User>> {
        self.find_by_id(&UserId::new(id.to_string())).await
    }
}
