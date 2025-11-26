use systemprompt_models::AuthError;

pub fn validate_redirect_uri(
    registered_uris: &[String],
    requested_uri: Option<&str>,
) -> Result<String, AuthError> {
    let uri = requested_uri
        .filter(|u| !u.is_empty())
        .ok_or(AuthError::InvalidRedirectUri)?;

    if !registered_uris.contains(&uri.to_string()) {
        return Err(AuthError::InvalidRequest {
            reason: format!("Redirect URI '{uri}' not registered for this client"),
        });
    }

    Ok(uri.to_string())
}
