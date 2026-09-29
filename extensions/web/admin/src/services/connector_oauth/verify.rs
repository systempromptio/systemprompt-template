//! Read-only provider identity and MCP protocol verification before connection
//! use.

use super::report::VerificationReport;
use super::response::{body, tool_error};
use super::transport::client;
use super::{Grant, Provider};
use crate::error::{AdminError, AdminResult};
use serde_json::{Value, json};
use std::time::Instant;

async fn rpc(
    http: &reqwest::Client,
    grant: &Grant,
    session: &mut Option<String>,
    payload: Value,
) -> AdminResult<Value> {
    let mut request = http
        .post(grant.provider.endpoint())
        .header(
            "Authorization",
            format!("{} {}", grant.authorization_scheme, grant.access_token),
        )
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-03-26")
        .json(&payload);
    if let Some(id) = session.as_ref() {
        request = request.header("Mcp-Session-Id", id);
    }
    let response = request
        .send()
        .await
        .map_err(|_redacted_error| AdminError::Upstream("MCP verification unavailable".into()))?;
    if let Some(id) = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
    {
        *session = Some(id.into());
    }
    if payload.get("id").is_none() {
        return if response.status().is_success() {
            Ok(Value::Null)
        } else {
            Err(AdminError::Upstream(
                "MCP initialization notification rejected".into(),
            ))
        };
    }
    let value = body(response).await?;
    if value.get("error").is_some()
        || value.pointer("/result/isError").and_then(Value::as_bool) == Some(true)
    {
        return Err(tool_error(grant, &payload, &value));
    }
    value
        .get("result")
        .cloned()
        .ok_or_else(|| AdminError::Upstream("MCP response has no result".into()))
}

async fn identity(
    http: &reqwest::Client,
    grant: &mut Grant,
    session: &mut Option<String>,
) -> AdminResult<()> {
    if matches!(grant.provider, Provider::Generic(_)) {
        grant.resource_name = grant.provider.endpoint();
        return super::userinfo::generic_identity(http, grant).await;
    }
    let info = match &grant.provider {
        Provider::Generic(_) => return Ok(()),
        Provider::Atlassian => super::payload::atlassian_user(
            &rpc(
                http,
                grant,
                session,
                json!({"jsonrpc":"2.0", "id":3,
            "method":"tools/call", "params":{"name":"atlassianUserInfo", "arguments":{}}}),
            )
            .await?,
        )?,
        Provider::Github => {
            let response = http
                .get("https://api.github.com/user")
                .bearer_auth(&grant.access_token)
                .header("User-Agent", "Systemprompt")
                .send()
                .await
                .map_err(|_redacted_error| {
                    AdminError::Upstream("Provider identity unavailable".into())
                })?;
            body(response).await?
        },
    };
    let id = match &grant.provider {
        Provider::Atlassian => info.get("account_id").or_else(|| info.get("accountId")),
        Provider::Github => info.get("id"),
        Provider::Generic(_) => None,
    };
    grant.account_id = id
        .and_then(|v| {
            v.as_str()
                .map(str::to_owned)
                .or_else(|| v.as_u64().map(|n| n.to_string()))
        })
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AdminError::Upstream("Provider returned no verified account ID".into()))?;
    info.get("displayName")
        .or_else(|| info.get("login"))
        .or_else(|| info.get("preferred_username"))
        .and_then(Value::as_str)
        .unwrap_or(&grant.account_id)
        .clone_into(&mut grant.account_name);
    if grant.provider == Provider::Atlassian {
        let sites = super::payload::atlassian_sites(&rpc(http, grant, session, json!({"jsonrpc":"2.0", "id":4,
            "method":"tools/call", "params":{"name":"getAccessibleAtlassianResources", "arguments":{}}})).await?)?;
        super::site::select(grant, &sites).await?;
    }
    Ok(())
}

pub async fn verify(grant: &mut Grant) -> AdminResult<()> {
    verify_with_client(grant, &client()?).await
}

pub async fn verify_with_client(grant: &mut Grant, http: &reqwest::Client) -> AdminResult<()> {
    let mut report = VerificationReport::for_provider(grant.provider.slug());
    verify_reporting(grant, http, &mut report).await
}

pub async fn verify_reporting(
    grant: &mut Grant,
    http: &reqwest::Client,
    report: &mut VerificationReport,
) -> AdminResult<()> {
    let mut session = None;
    let result = handshake(grant, http, &mut session, report).await;
    // Why: The probe uses a private session, never a user's active Claude session.
    if let Some(session) = session {
        let _cleanup_result = http
            .delete(grant.provider.endpoint())
            .header("Mcp-Session-Id", session)
            .header(
                "Authorization",
                format!("{} {}", grant.authorization_scheme, grant.access_token),
            )
            .send()
            .await;
    }
    result
}

async fn handshake(
    grant: &mut Grant,
    http: &reqwest::Client,
    session: &mut Option<String>,
    report: &mut VerificationReport,
) -> AdminResult<()> {
    let started = Instant::now();
    let outcome = async {
        rpc(
            http,
            grant,
            session,
            json!({"jsonrpc":"2.0", "id":1, "method":"initialize",
            "params":{"protocolVersion":"2025-03-26", "capabilities":{},
                "clientInfo":{"name":"Systemprompt connection verification", "version":"1.0"}}}),
        )
        .await?;
        rpc(
            http,
            grant,
            session,
            json!({"jsonrpc":"2.0", "method":"notifications/initialized"}),
        )
        .await?;
        Ok(format!(
            "MCP session open at {}",
            host(&grant.provider.endpoint())
        ))
    }
    .await;
    report.record("initialize", started, &outcome);
    outcome?;

    let started = Instant::now();
    let outcome = async {
        let tools = rpc(
            http,
            grant,
            session,
            json!({"jsonrpc":"2.0", "id":2, "method":"tools/list"}),
        )
        .await?;
        let count = tools
            .get("tools")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        if count == 0 {
            return Err(AdminError::Upstream(
                "MCP server has no accessible tools".into(),
            ));
        }
        Ok(format!("{count} tools available"))
    }
    .await;
    report.record("tools", started, &outcome);
    outcome?;

    let started = Instant::now();
    let outcome = async {
        identity(http, grant, session).await?;
        Ok(
            match (
                grant.account_name.is_empty(),
                grant.resource_name.is_empty(),
            ) {
                (false, false) => {
                    format!("{} · {}", grant.account_name, host(&grant.resource_name))
                },
                (false, true) => grant.account_name.clone(),
                _ => "Identity confirmed".to_owned(),
            },
        )
    }
    .await;
    report.record("identity", started, &outcome);
    outcome.map(|_| ())
}

fn host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}
