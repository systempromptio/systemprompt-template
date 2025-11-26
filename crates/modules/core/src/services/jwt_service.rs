use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use systemprompt_identifiers::{JwtToken, UserId};

#[derive(Debug, Serialize, Deserialize)]
struct AdminTokenClaims {
    sub: String,
    exp: i64,
    iat: i64,
    roles: Vec<String>,
    aud: Vec<String>,
    session_id: String,
}

#[derive(Copy, Clone, Debug)]
pub struct JwtService;

impl JwtService {
    pub fn generate_admin_token(
        user_id: &UserId,
        jwt_secret: &str,
        duration: Duration,
    ) -> Result<JwtToken> {
        let now = Utc::now();
        let expiry = now + duration;

        let claims = AdminTokenClaims {
            sub: user_id.as_str().to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            roles: vec!["admin".to_string()],
            aud: vec!["a2a".to_string(), "api".to_string(), "mcp".to_string()],
            session_id: "system".to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )?;

        Ok(JwtToken::new(token))
    }
}
