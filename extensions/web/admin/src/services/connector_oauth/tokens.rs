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
    error_description: Option<String>,
    token_type: Option<String>,
}

use systemprompt_web_shared::format::truncate_chars as truncate;

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
    let provider = grant.provider.slug().to_owned();
    let response = request
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(
                target: "connector_oauth",
                provider,
                grant_type = fields.first().map_or("", |(_, v)| v),
                timeout = error.is_timeout(),
                connect = error.is_connect(),
                "token request failed before a response"
            );
            AdminError::Upstream("Connector authorization service unavailable".into())
        })?;
    let tokens = accepted_tokens(&provider, response).await?;
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

async fn accepted_tokens(provider: &str, response: reqwest::Response) -> AdminResult<Tokens> {
    let status = response.status();
    if status.is_server_error() || status.as_u16() == 429 {
        tracing::warn!(
            target: "connector_oauth",
            provider,
            status = status.as_u16(),
            "token endpoint unavailable"
        );
        return Err(AdminError::Upstream(
            "Connector authorization service unavailable".into(),
        ));
    }
    let tokens: Tokens = super::generic_discovery::bounded_json(response).await?;
    // Why: RFC 6749 §5.2 — every error a token endpoint answers with a 4xx
    // (invalid_grant, unauthorized_client, invalid_client, invalid_scope …)
    // is final for this grant; retrying it is a loop, not resilience. Only a
    // 5xx, a 429 or a transport failure is an outage.
    if !status.is_success() || tokens.error.is_some() {
        let code = tokens.error.as_deref().unwrap_or("none");
        tracing::warn!(
            target: "connector_oauth",
            provider,
            status = status.as_u16(),
            oauth_error = code,
            oauth_error_description = tokens.error_description.as_deref().map(|d| truncate(d, 160)),
            "token request rejected; grant retired"
        );
        return Err(AdminError::Unauthorized(format!(
            "Connector grant rejected ({code}); reconnect required"
        )));
    }
    Ok(tokens)
}

fn apply_tokens(grant: &mut Grant, tokens: Tokens) -> AdminResult<()> {
    grant.access_token = tokens
        .access_token
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| AdminError::Upstream("Connector returned no access token".into()))?;
    if let Some(refresh) = tokens.refresh_token {
        grant.refresh_token = Some(refresh);
    }
    // Why: a generic provider may omit expires_in. In that case refresh on the
    // next use instead of guessing how long the upstream session will stay valid.
    let seconds = tokens
        .expires_in
        .filter(|n| *n > 0 && *n <= 31_536_000)
        .or_else(|| matches!(grant.provider, Provider::Generic(_)).then_some(0))
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
