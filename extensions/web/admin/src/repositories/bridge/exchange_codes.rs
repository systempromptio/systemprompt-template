use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::error::Result;

const EXCHANGE_CODE_BYTES: usize = 32;
const EXCHANGE_CODE_TTL_SECONDS: i64 = 120;

#[derive(Debug)]
pub struct IssuedExchangeCode {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn issue_exchange_code(pool: &PgPool, user_id: &UserId) -> Result<IssuedExchangeCode> {
    let code = generate_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + ChronoDuration::seconds(EXCHANGE_CODE_TTL_SECONDS);

    sqlx::query!(
        "INSERT INTO bridge_exchange_codes (code_hash, user_id, expires_at) VALUES ($1, $2, $3)",
        code_hash,
        user_id.as_str(),
        expires_at,
    )
    .execute(pool)
    .await?;

    Ok(IssuedExchangeCode { code, expires_at })
}

fn generate_code() -> String {
    let mut raw = [0u8; EXCHANGE_CODE_BYTES];
    rand::rng().fill_bytes(&mut raw);
    let mut out = String::with_capacity(EXCHANGE_CODE_BYTES * 2);
    for byte in raw {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn hash_code(code: &str) -> String {
    let digest = Sha256::digest(code.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
