use systemprompt_models::{AuthError, GrantType, PkceMethod, ResponseType};

#[derive(Debug)]
pub struct PkceChallenge {
    pub challenge: String,
    pub method: PkceMethod,
}

#[derive(Debug)]
pub struct CsrfToken(String);

impl CsrfToken {
    pub fn new(state: impl Into<String>) -> Result<Self, AuthError> {
        let state = state.into();

        if state.is_empty() {
            return Err(AuthError::MissingState);
        }

        if !state
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(AuthError::InvalidRequest {
                reason: "State must be alphanumeric".to_string(),
            });
        }

        Ok(Self(state))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug)]
pub struct ValidatedClientRegistration {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<GrantType>,
    pub response_types: Vec<ResponseType>,
}

pub fn required_param(value: Option<&str>, param_name: &str) -> Result<String, AuthError> {
    value
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AuthError::InvalidRequest {
            reason: format!("{param_name} parameter is required"),
        })
        .map(ToString::to_string)
}

pub fn optional_param(value: Option<&str>) -> Option<String> {
    value.filter(|s| !s.is_empty()).map(ToString::to_string)
}

pub fn scope_param(value: Option<&str>) -> Result<Vec<String>, AuthError> {
    let scope_str = required_param(value, "scope")?;

    let scopes: Vec<String> = scope_str
        .split_whitespace()
        .map(ToString::to_string)
        .collect();

    if scopes.is_empty() {
        return Err(AuthError::InvalidScope { scope: scope_str });
    }

    Ok(scopes)
}

pub fn validate_pkce(
    code_challenge: Option<&str>,
    code_challenge_method: Option<&str>,
) -> Result<PkceChallenge, AuthError> {
    let challenge = code_challenge
        .filter(|c| !c.is_empty())
        .ok_or(AuthError::MissingCodeChallenge)?
        .to_string();

    let method_str = code_challenge_method.ok_or_else(|| AuthError::InvalidRequest {
        reason: "code_challenge_method required".to_string(),
    })?;

    let method = method_str.parse::<PkceMethod>()?;

    Ok(PkceChallenge { challenge, method })
}

pub fn get_audit_user(user_id: Option<&str>) -> Result<String, AuthError> {
    user_id
        .filter(|id| !id.is_empty())
        .ok_or_else(|| AuthError::InvalidRequest {
            reason: "Authenticated user required for this operation".to_string(),
        })
        .map(ToString::to_string)
}
