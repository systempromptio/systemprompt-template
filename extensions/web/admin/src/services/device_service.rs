use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::bridge_grp::{self, EnrolledDevice, IssuedApiKey};

pub async fn enroll_device(
    pool: &PgPool,
    user_id: &UserId,
    name: &str,
    platform: &str,
    hostname: &str,
    expires_at: Option<DateTime<Utc>>,
) -> AdminResult<EnrolledDevice> {
    let enrolled =
        bridge_grp::enroll_device(pool, user_id, name, platform, hostname, expires_at).await?;
    Ok(enrolled)
}

pub async fn issue_pat(
    pool: &PgPool,
    user_id: &UserId,
    name: &str,
    expires_at: Option<DateTime<Utc>>,
) -> AdminResult<IssuedApiKey> {
    let issued = bridge_grp::issue_api_key(pool, user_id, name, expires_at).await?;
    Ok(issued)
}

pub async fn revoke_pat(pool: &PgPool, user_id: &UserId, id: &str) -> AdminResult<()> {
    let revoked = bridge_grp::revoke_api_key(pool, user_id, id).await?;
    if !revoked {
        return Err(AdminError::NotFound("PAT not found".to_string()));
    }
    Ok(())
}

pub async fn revoke_device_cert(pool: &PgPool, user_id: &UserId, id: &str) -> AdminResult<()> {
    let revoked = bridge_grp::revoke_device_cert(pool, user_id, id).await?;
    if !revoked {
        return Err(AdminError::NotFound("cert not found".to_string()));
    }
    Ok(())
}
