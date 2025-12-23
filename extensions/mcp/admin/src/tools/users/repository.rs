use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::User;

pub struct UsersRepository {
    pool: Arc<PgPool>,
}

impl UsersRepository {
    pub fn new(db: DbPool) -> Result<Self> {
        let pool = db.pool_arc()?;
        Ok(Self { pool })
    }

    pub async fn list_users(&self, user_id: Option<&str>) -> Result<Vec<User>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                u.id,
                u.name,
                u.email,
                u.display_name,
                u.status,
                u.roles as "roles: Vec<String>",
                u.created_at::text as created_at,
                COALESCE((SELECT COUNT(*) FROM user_sessions s WHERE s.user_id = u.id), 0) as total_sessions
            FROM users u
            WHERE ($1::text IS NULL OR u.id = $1)
              AND u.status != 'deleted'
            ORDER BY u.created_at DESC
            LIMIT 100
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.id,
                name: r.name,
                email: r.email,
                display_name: r.display_name,
                status: r.status,
                roles: r.roles,
                total_sessions: r.total_sessions.unwrap_or(0),
                created_at: r.created_at.unwrap_or_default(),
            })
            .collect())
    }
}
