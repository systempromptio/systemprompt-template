use anyhow::{anyhow, Result};
use chrono::Utc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::models::JwtClaims;

pub fn validate_jwt_token(token: &str, jwt_secret: &str) -> Result<JwtClaims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_aud = false; // Don't validate audience

    let token_data = match decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data,
        Err(e) => {
            return Err(anyhow!("JWT validation failed: {e}"));
        },
    };

    // Check expiration
    let now = Utc::now().timestamp();

    if token_data.claims.exp < now {
        return Err(anyhow!("Token has expired"));
    }

    Ok(token_data.claims)
}
