use super::AuthorizeQuery;
use crate::repository::OAuthRepository;
use anyhow::Result;

pub async fn validate_authorize_request(
    params: &AuthorizeQuery,
    repo: &OAuthRepository,
) -> Result<String> {
    if params.response_type != "code" {
        return Err(anyhow::anyhow!(
            "Unsupported response_type. Only 'code' is supported"
        ));
    }

    let client = repo
        .find_client(&params.client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid client_id"))?;

    if let Some(redirect_uri) = &params.redirect_uri {
        eprintln!("  Validating redirect_uri: {redirect_uri}");
        eprintln!("  Against registered URIs: {:?}", client.redirect_uris);

        let is_valid = client.redirect_uris.contains(redirect_uri);

        eprintln!("  redirect_uri validation result: {is_valid}");

        if !is_valid {
            eprintln!("  redirect_uri '{redirect_uri}' not found in client registered URIs");
            eprintln!("  Checking for similar URIs (potential encoding issues):");
            for registered_uri in &client.redirect_uris {
                eprintln!("    - Registered: {registered_uri}");
                if registered_uri
                    .starts_with(&redirect_uri[..redirect_uri.len().min(registered_uri.len())])
                {
                    eprintln!("      POTENTIAL MATCH: Similar prefix detected!");
                }
            }
            return Err(anyhow::anyhow!(
                "redirect_uri '{}' not registered for client '{}'",
                redirect_uri,
                params.client_id
            ));
        }
        eprintln!("  redirect_uri validation passed");
    }

    let scope = if let Some(scope_param) = params.scope.as_deref() {
        scope_param.to_string()
    } else {
        if client.scopes.is_empty() {
            return Err(anyhow::anyhow!(
                "Client has no registered scopes and none provided in request"
            ));
        }
        client.scopes.join(" ")
    };

    let requested_scopes = crate::repository::OAuthRepository::parse_scopes(&scope);

    let valid_scopes = repo
        .validate_scopes(&requested_scopes)
        .await
        .map_err(|e| anyhow::anyhow!("Invalid scopes requested: {e}"))?;

    for requested_scope in &valid_scopes {
        if !client.scopes.contains(requested_scope) {
            return Err(anyhow::anyhow!(
                "Scope '{}' not allowed for client '{}'",
                requested_scope,
                params.client_id
            ));
        }
    }

    Ok(scope)
}

pub fn validate_oauth_parameters(params: &AuthorizeQuery) -> Result<(), String> {
    if params.response_type != "code" {
        return Err(format!(
            "Unsupported response_type '{}'. Only 'code' is supported.",
            params.response_type
        ));
    }

    if let Some(response_mode) = &params.response_mode {
        if response_mode != "query" {
            return Err(format!(
                "Unsupported response_mode '{response_mode}'. Only 'query' mode is supported."
            ));
        }
    }

    if let Some(code_challenge) = &params.code_challenge {
        if code_challenge.len() < 43 {
            return Err(
                "code_challenge too short. Must be at least 43 characters for security."
                    .to_string(),
            );
        }
        if code_challenge.len() > 128 {
            return Err("code_challenge too long. Must be at most 128 characters.".to_string());
        }

        let is_valid_base64url = code_challenge
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

        if !is_valid_base64url {
            return Err(
                "code_challenge must be base64url encoded (A-Z, a-z, 0-9, -, _)".to_string(),
            );
        }

        if is_low_entropy_challenge(code_challenge) {
            return Err(
                "code_challenge appears to have insufficient entropy for security".to_string(),
            );
        }

        let method = params.code_challenge_method.as_deref().ok_or_else(|| {
            "code_challenge_method is required when code_challenge is provided".to_string()
        })?;

        match method {
            "S256" => {},
            "plain" => {
                return Err(
                    "PKCE method 'plain' is not allowed. Use 'S256' for security.".to_string(),
                );
            },
            _ => {
                return Err(format!(
                    "Unsupported code_challenge_method '{method}'. Only 'S256' is allowed."
                ));
            },
        }
    } else {
        eprintln!(
            "Authorization request without PKCE from client {}. Consider requiring PKCE for \
             enhanced security.",
            params.client_id
        );
    }

    if let Some(display) = &params.display {
        match display.as_str() {
            "page" | "popup" | "touch" | "wap" => {},
            _ => {
                return Err(format!(
                    "Unsupported display value '{display}'. Supported values: page, popup, touch, \
                     wap."
                ));
            },
        }
    }

    if let Some(prompt) = &params.prompt {
        for prompt_value in prompt.split_whitespace() {
            match prompt_value {
                "none" | "login" | "consent" | "select_account" => {},
                _ => {
                    return Err(format!(
                        "Unsupported prompt value '{prompt_value}'. Supported values: none, \
                         login, consent, select_account."
                    ));
                },
            }
        }
    }

    if let Some(max_age) = params.max_age {
        if max_age < 0 {
            return Err("max_age must be a non-negative integer".to_string());
        }
    }

    Ok(())
}

fn is_low_entropy_challenge(challenge: &str) -> bool {
    if challenge
        .chars()
        .all(|c| c == challenge.chars().next().unwrap())
    {
        return true;
    }

    if challenge.len() >= 6 {
        let pattern_length = 3;
        let pattern = &challenge[..pattern_length];
        let repetitions = challenge.len() / pattern_length;
        if repetitions > 2 && challenge.starts_with(&pattern.repeat(repetitions)) {
            return true;
        }
    }

    let chars: Vec<char> = challenge.chars().collect();
    if chars.len() >= 8 {
        let mut sequential_count = 1;
        for i in 1..chars.len() {
            if let (Some(prev), Some(curr)) = (chars[i - 1].to_digit(36), chars[i].to_digit(36)) {
                if curr == prev + 1 {
                    sequential_count += 1;
                    if sequential_count >= 8 {
                        return true;
                    }
                } else {
                    sequential_count = 1;
                }
            }
        }
    }

    let unique_chars: std::collections::HashSet<char> = challenge.chars().collect();
    let entropy_ratio = unique_chars.len() as f64 / challenge.len() as f64;

    if entropy_ratio < 0.3 {
        return true;
    }

    false
}
