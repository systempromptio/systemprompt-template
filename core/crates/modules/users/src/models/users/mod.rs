pub mod api;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt_identifiers::{SessionId, UserId};

pub use api::{CreateUserRequest, ListUsersQuery, UpdateUserRequest};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    #[sqlx(try_from = "String")]
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub email_verified: Option<bool>,
    pub roles: Vec<String>,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub is_scanner: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserActivity {
    #[sqlx(try_from = "String")]
    pub user_id: UserId,
    pub last_active: Option<DateTime<Utc>>,
    pub session_count: i64,
    pub task_count: i64,
    pub message_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserWithSessions {
    #[sqlx(try_from = "String")]
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub status: Option<String>,
    pub roles: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub active_sessions: i64,
    pub last_session_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    #[sqlx(try_from = "String")]
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub email_verified: Option<bool>,
    pub roles: Vec<String>,
    pub avatar_url: Option<String>,
    pub is_bot: bool,
    pub is_scanner: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            email: row.email,
            full_name: row.full_name,
            display_name: row.display_name,
            status: row.status,
            email_verified: row.email_verified,
            roles: row.roles,
            avatar_url: row.avatar_url,
            is_bot: row.is_bot,
            is_scanner: row.is_scanner,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserActivityRow {
    #[sqlx(try_from = "String")]
    pub user_id: UserId,
    pub last_active: Option<DateTime<Utc>>,
    pub session_count: i64,
    pub task_count: i64,
    pub message_count: i64,
}

impl From<UserActivityRow> for UserActivity {
    fn from(row: UserActivityRow) -> Self {
        Self {
            user_id: row.user_id,
            last_active: row.last_active,
            session_count: row.session_count,
            task_count: row.task_count,
            message_count: row.message_count,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserWithSessionsRow {
    #[sqlx(try_from = "String")]
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub status: Option<String>,
    pub roles: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub active_sessions: i64,
    pub last_session_at: Option<DateTime<Utc>>,
}

impl From<UserWithSessionsRow> for UserWithSessions {
    fn from(row: UserWithSessionsRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            email: row.email,
            full_name: row.full_name,
            status: row.status,
            roles: row.roles,
            created_at: row.created_at,
            active_sessions: row.active_sessions,
            last_session_at: row.last_session_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserSessionRow {
    #[sqlx(try_from = "String")]
    pub session_id: SessionId,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl From<UserSessionRow> for UserSession {
    fn from(row: UserSessionRow) -> Self {
        Self {
            session_id: row.session_id,
            user_id: row.user_id.map(UserId::new),
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            device_type: row.device_type,
            started_at: row.started_at,
            last_activity_at: row.last_activity_at,
            ended_at: row.ended_at,
        }
    }
}

impl User {
    pub fn is_active(&self) -> bool {
        self.status.as_deref() == Some("active")
    }

    pub fn is_admin(&self) -> bool {
        self.roles.contains(&"admin".to_string())
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}
