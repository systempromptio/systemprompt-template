//! OAuth token exchange and refresh, using only bound destinations.
use super::transport::client;
use super::{Grant, Provider};
use crate::error::{AdminError, AdminResult};
use chrono::Utc;
use serde::Deserialize;
#[derive(Deserialize)]
struct Tokens {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    error: Option<String>,
    token_type: Option<String>,
}

async fn request_with_client(
    grant: &mut Grant,
    fields: &[(&str, &str)],
    http: &reqwest::Client,
) -> AdminResult<()> {
    super::generic::validate_grant(grant)?;
    let body = {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.extend_pairs(fields)
            .append_pair("client_id", &grant.client);
        if !grant.client_secret.is_empty()
            && matches!(grant.token_auth_method.as_str(), "" | "client_secret_post")
        {
            form.append_pair("client_secret", &grant.client_secret);
        }
        if grant.provider == Provider::Atlassian || matches!(grant.provider, Provider::Generic(_)) {
            form.append_pair("resource", &grant.provider.endpoint());
        }
        form.finish()
    };
    let mut request = http.post(&grant.token_endpoint);
    if grant.token_auth_method == "client_secret_basic" {
        let encode = |value: &str| {
            url::form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>()
        };
        request = request.basic_auth(encode(&grant.client), Some(encode(&grant.client_secret)));
    }
    let response = request
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|_redacted_error| {
            AdminError::Upstream("Connector authorization service unavailable".into())
        })?;
    let status = response.status();
    if status.is_server_error() || status.as_u16() == 429 {
        return Err(AdminError::Upstream(
            "Connector authorization service unavailable".into(),
        ));
    }
    let tokens: Tokens = super::generic_discovery::bounded_json(response).await?;
    if tokens.error.as_deref() == Some("invalid_grant") || status.as_u16() == 401 {
        return Err(AdminError::Unauthorized(
            "Connector grant rejected; reconnect required".into(),
        ));
    }
    if !status.is_success() || tokens.error.is_some() {
        return Err(AdminError::Upstream(
            "Connector application authorization rejected".into(),
        ));
    }
    if matches!(grant.provider, Provider::Generic(_))
        && !tokens
            .token_type
            .as_deref()
            .is_some_and(|kind| kind.eq_ignore_ascii_case("bearer"))
    {
        return Err(AdminError::Upstream(
            "Connector requires a Bearer access token".into(),
        ));
    }
    apply_tokens(grant, tokens)
}

fn apply_tokens(grant: &mut Grant, tokens: Tokens) -> AdminResult<()> {
    grant.access_token = tokens
        .access_token
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| AdminError::Upstream("Connector returned no access token".into()))?;
    if let Some(refresh) = tokens.refresh_token {
        grant.refresh_token = Some(refresh);
    }
    // Why: Salesforce may omit expires_in. In that case refresh on the next use
    // instead of guessing how long the upstream session will stay valid.
    let seconds = tokens
        .expires_in
        .filter(|n| *n > 0 && *n <= 31_536_000)
        .or_else(|| {
            matches!(
                grant.provider,
                Provider::Salesforce(_) | Provider::Generic(_)
            )
            .then_some(0)
        })
        .ok_or_else(|| AdminError::Upstream("Connector returned no valid token expiry".into()))?;
    grant.expires_at = if seconds == 0 && matches!(grant.provider, Provider::Generic(_)) {
        i64::MAX
    } else {
        Utc::now().timestamp() + seconds
    };
    if grant.refresh_token.is_none() && !matches!(grant.provider, Provider::Generic(_)) {
        return Err(AdminError::Unauthorized(
            "Connector did not grant refresh authorization".into(),
        ));
    }
    Ok(())
}

pub async fn exchange(grant: &mut Grant, code: &str) -> AdminResult<()> {
    let callback = grant.provider.callback()?;
    exchange_with_client(grant, code, &callback, &client()?).await
}

pub async fn exchange_with_client(
    grant: &mut Grant,
    code: &str,
    callback: &str,
    http: &reqwest::Client,
) -> AdminResult<()> {
    let verifier = grant.verifier.clone();
    request_with_client(
        grant,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", callback),
            ("code_verifier", &verifier),
        ],
        http,
    )
    .await?;
    grant.verifier.clear();
    Ok(())
}

pub(super) async fn refresh(grant: &mut Grant) -> AdminResult<()> {
    refresh_with_client(grant, &client()?).await
}

pub async fn refresh_with_client(grant: &mut Grant, http: &reqwest::Client) -> AdminResult<()> {
    if matches!(grant.provider, Provider::Generic(_)) {
        super::generic::validate_refresh(grant, http).await?;
    }
    let token = grant.refresh_token.clone().ok_or_else(|| {
        AdminError::Unauthorized("Connector refresh authorization missing".into())
    })?;
    request_with_client(
        grant,
        &[("grant_type", "refresh_token"), ("refresh_token", &token)],
        http,
    )
    .await
}
