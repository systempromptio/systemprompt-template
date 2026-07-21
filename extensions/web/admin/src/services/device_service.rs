//! Device enrolment and personal access token lifecycle.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::bridge::{self, EnrollDeviceParams, EnrolledDevice, IssuedApiKey};

pub(crate) struct EnrollDeviceInput<'a> {
    pub name: &'a str,
    pub platform: &'a str,
    pub hostname: &'a str,
    pub expires_at: Option<DateTime<Utc>>,
}

pub(crate) async fn enroll_device(
    pool: &PgPool,
    user_id: &UserId,
    req: EnrollDeviceInput<'_>,
) -> AdminResult<EnrolledDevice> {
    let enrolled = bridge::enroll_device(
        pool,
        user_id,
        EnrollDeviceParams {
            name: req.name,
            platform: req.platform,
            hostname: req.hostname,
            expires_at: req.expires_at,
        },
    )
    .await?;
    Ok(enrolled)
}

pub(crate) async fn issue_pat(
    pool: &PgPool,
    user_id: &UserId,
    name: &str,
    expires_at: Option<DateTime<Utc>>,
) -> AdminResult<IssuedApiKey> {
    let issued = bridge::issue_api_key(pool, user_id, name, expires_at).await?;
    Ok(issued)
}

pub(crate) async fn revoke_pat(pool: &PgPool, user_id: &UserId, id: &str) -> AdminResult<()> {
    let revoked = bridge::revoke_api_key(pool, user_id, id).await?;
    if !revoked {
        return Err(AdminError::NotFound("PAT not found".to_owned()));
    }
    Ok(())
}

pub(crate) async fn revoke_device_cert(
    pool: &PgPool,
    user_id: &UserId,
    id: &str,
) -> AdminResult<()> {
    let revoked = bridge::revoke_device_cert(pool, user_id, id).await?;
    if !revoked {
        return Err(AdminError::NotFound("cert not found".to_owned()));
    }
    Ok(())
}
