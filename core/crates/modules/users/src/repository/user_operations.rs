use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use systemprompt_identifiers::UserId;

use crate::models::{User, UserRow};

use super::UserRepository;

impl UserRepository {
    pub async fn create(
        &self,
        name: &str,
        email: &str,
        full_name: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<User> {
        if name.trim().is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }

        if name.len() < 3 || name.len() > 50 {
            return Err(anyhow!(
                "Username must be between 3 and 50 characters (got {})",
                name.len()
            ));
        }

        if email.trim().is_empty() {
            return Err(anyhow!("Email cannot be empty"));
        }

        if !email.contains('@') {
            return Err(anyhow!("Invalid email format: must contain '@' symbol"));
        }

        if let Some(fn_) = &full_name {
            if fn_.trim().is_empty() {
                return Err(anyhow!("Full name cannot be empty if provided"));
            }
        }

        let now = Utc::now();
        let id = UserId::new(uuid::Uuid::new_v4().to_string());

        let display_name_val = display_name.or(full_name);

        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (
                id, name, email, full_name, display_name,
                status, email_verified, roles, is_bot,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 'active', false, ARRAY['user']::TEXT[], false, $6, $6)
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            id.as_str(),
            name,
            email,
            full_name,
            display_name_val,
            now
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn create_anonymous(&self, fingerprint: &str) -> Result<User> {
        let user_id = uuid::Uuid::new_v4();
        let id = UserId::new(user_id.to_string());
        let name = format!("anonymous_{}", &user_id.to_string()[..8]);
        let email = format!("{}@anonymous.local", fingerprint);
        let now = Utc::now();

        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (
                id, name, email, status, email_verified, roles,
                is_bot, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'active', false, ARRAY['anonymous']::TEXT[], false, $4, $4)
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            id.as_str(),
            name,
            email,
            now
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_email(&self, id: &UserId, email: &str) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET email = $1, email_verified = false, updated_at = $2
            WHERE id = $3
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            email,
            Utc::now(),
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_full_name(&self, id: &UserId, full_name: &str) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET full_name = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            full_name,
            Utc::now(),
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_status(&self, id: &UserId, status: &str) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET status = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            status,
            Utc::now(),
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_email_verified(&self, id: &UserId, verified: bool) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET email_verified = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            verified,
            Utc::now(),
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn update_all_fields(
        &self,
        id: &UserId,
        email: &str,
        full_name: Option<&str>,
        display_name: Option<&str>,
        status: &str,
    ) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET email = $1, full_name = $2, display_name = $3, status = $4, updated_at = $5
            WHERE id = $6
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            email,
            full_name,
            display_name,
            status,
            Utc::now(),
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn assign_roles(&self, id: &UserId, roles: &[String]) -> Result<User> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET roles = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, name, email, full_name, display_name, status, email_verified,
                      roles, avatar_url, is_bot, is_scanner, created_at, updated_at
            "#,
            roles,
            Utc::now(),
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(User::from(row))
    }

    pub async fn delete(&self, id: &UserId) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET status = 'deleted', updated_at = $1
            WHERE id = $2
            "#,
            Utc::now(),
            id.as_str()
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_anonymous(&self, id: &UserId) -> Result<()> {
        sqlx::query!(
            r#"DELETE FROM users WHERE id = $1 AND 'anonymous' = ANY(roles)"#,
            id.as_str()
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn cleanup_old_anonymous(&self, days: i32) -> Result<u64> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE 'anonymous' = ANY(roles)
              AND created_at < $1
              AND id NOT IN (
                  SELECT DISTINCT user_id FROM user_sessions WHERE ended_at IS NULL
              )
            "#,
            cutoff
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn create_user(&self, email: &str, name: &str) -> Result<User> {
        self.create(name, email, None, None).await
    }

    pub async fn create_anonymous_user(&self) -> Result<User> {
        let fingerprint = uuid::Uuid::new_v4().to_string();
        self.create_anonymous(&fingerprint).await
    }
}
