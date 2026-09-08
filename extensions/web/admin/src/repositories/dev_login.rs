//! One-shot developer login codes behind `GET /admin/auth/dev/login`.
//!
//! Same shape as the bridge exchange codes — 32 random bytes, SHA-256 at
//! rest, a ten-minute window — but in a table of their own: a bridge
//! device-link code must never be redeemable as a browser session, nor the
//! reverse. Consumption is one atomic `UPDATE`; a code stays redeemable for a
//! short grace after its first use because address-bar prefetchers and link
//! previews fetch a URL before the person does, and a strictly single-use
//! link then locks the real browser out.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

const DEV_LOGIN_CODE_BYTES: usize = 32;
pub const DEV_LOGIN_CODE_TTL_SECONDS: i64 = 600;
pub const DEV_LOGIN_REDEEM_GRACE_SECONDS: i64 = 60;

#[derive(Debug)]
pub struct IssuedDevLoginCode {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

// Why: the account a redeemed code signs in, in the shape the session mint
// needs. Roles are read at redeem time, not issue time, so a role change
// between the two is honoured.
#[derive(Debug, Clone)]
pub struct DevLoginUser {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

pub fn hash_dev_login_code(code: &str) -> String {
    hex::encode(Sha256::digest(code.as_bytes()))
}

// Why: the CLI takes "a username"; on this instance that is the e-mail, but
// the `name` column is matched too so the short form works where it differs.
pub async fn find_active_user_id_by_login(
    pool: &PgPool,
    login: &str,
) -> Result<Option<UserId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT id AS "id: UserId" FROM users
           WHERE (LOWER(email) = LOWER($1) OR name = $1) AND status = 'active'
           ORDER BY (LOWER(email) = LOWER($1)) DESC
           LIMIT 1"#,
        login,
    )
    .fetch_optional(pool)
    .await
}

pub async fn insert_dev_login_code(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<IssuedDevLoginCode, sqlx::Error> {
    let mut raw = [0u8; DEV_LOGIN_CODE_BYTES];
    rand::rng().fill_bytes(&mut raw);
    let code = hex::encode(raw);
    let expires_at = Utc::now() + ChronoDuration::seconds(DEV_LOGIN_CODE_TTL_SECONDS);

    sqlx::query!(
        "INSERT INTO dev_login_codes (code_hash, user_id, expires_at) VALUES ($1, $2, $3)",
        hash_dev_login_code(&code),
        user_id.as_str(),
        expires_at,
    )
    .execute(pool)
    .await?;

    Ok(IssuedDevLoginCode { code, expires_at })
}

pub async fn consume_dev_login_code(
    pool: &PgPool,
    code: &str,
) -> Result<Option<DevLoginUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"UPDATE dev_login_codes c SET consumed_at = COALESCE(c.consumed_at, NOW())
           FROM users u
           WHERE c.code_hash = $1
             AND (c.consumed_at IS NULL
                  OR c.consumed_at > NOW() - make_interval(secs => $2))
             AND c.expires_at > NOW()
             AND u.id = c.user_id
             AND u.status = 'active'
           RETURNING u.id AS "user_id: UserId",
                     u.email AS "email!",
                     COALESCE(u.display_name, u.name) AS "display_name!",
                     u.roles AS "roles!: Vec<String>""#,
        hash_dev_login_code(code),
        DEV_LOGIN_REDEEM_GRACE_SECONDS as f64,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| DevLoginUser {
        user_id: r.user_id,
        email: r.email,
        display_name: r.display_name,
        roles: r.roles,
    }))
}
