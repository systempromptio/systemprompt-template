use super::{TokenError, TokenResult};
use crate::repository::OAuthRepository;
use anyhow::Result;

pub fn extract_required_field<'a>(
    field: Option<&'a str>,
    field_name: &str,
) -> TokenResult<&'a str> {
    field.ok_or_else(|| TokenError::InvalidRequest {
        field: field_name.to_string(),
        message: "is required".to_string(),
    })
}

pub async fn validate_client_credentials(
    repo: &OAuthRepository,
    client_id: &str,
    client_secret: Option<&str>,
) -> Result<()> {
    let client = repo
        .find_client(client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    let auth_method = client.token_endpoint_auth_method.as_str();

    match auth_method {
        "none" => Ok(()),
        _ => {
            if let Some(secret) = client_secret {
                use crate::services::verify_client_secret;
                let hash = client
                    .client_secret_hash
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Client has no secret hash configured"))?;
                if !verify_client_secret(secret, hash)? {
                    return Err(anyhow::anyhow!("Invalid client secret"));
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Client secret required"))
            }
        },
    }
}

pub async fn validate_authorization_code(
    repo: &OAuthRepository,
    code: &str,
    client_id: &str,
    redirect_uri: Option<&str>,
    code_verifier: Option<&str>,
) -> Result<(String, String)> {
    let (user_id, scope) = repo
        .validate_authorization_code(code, client_id, redirect_uri, code_verifier)
        .await?;
    Ok((user_id, scope))
}
