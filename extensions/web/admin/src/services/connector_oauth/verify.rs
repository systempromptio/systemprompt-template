//! Read-only provider identity and MCP protocol verification before connection
//! use.

use super::transport::client;
use super::{Grant, Provider, config};
use crate::error::{AdminError, AdminResult};
use serde_json::{Value, json};

async fn body(response: reqwest::Response) -> AdminResult<Value> {
    let status = response.status();
    if status.as_u16() == 401 {
        return Err(AdminError::Unauthorized("Provider grant rejected".into()));
    }
    if status.as_u16() == 403 {
        return Err(AdminError::Forbidden(
            "Provider permissions do not allow this operation".into(),
        ));
    }
    if !status.is_success() {
        return Err(AdminError::Upstream(
            "Provider temporarily unavailable".into(),
        ));
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_redacted_error| AdminError::Upstream("Provider response interrupted".into()))?
    {
        if bytes.len() + chunk.len() > 1_048_576 {
            return Err(AdminError::Upstream(
                "Provider response exceeds verification limit".into(),
            ));
        }
        bytes.extend_from_slice(&chunk);
        if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
            return Ok(value);
        }
        // Why: Streamable HTTP can answer a request on an SSE stream that remains
        // open. Return when its JSON-RPC response arrives, not at stream EOF.
        if let Ok(text) = std::str::from_utf8(&bytes) {
            for frame in text.split("\n\n") {
                let data = frame
                    .lines()
                    .filter_map(|l| l.strip_prefix("data:"))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Ok(value) = serde_json::from_str::<Value>(&data)
                    && value.get("id").is_some()
                {
                    return Ok(value);
                }
            }
        }
    }
    Err(AdminError::Upstream(
        "Provider returned an invalid response".into(),
    ))
}

async fn rpc(grant: &Grant, session: &mut Option<String>, payload: Value) -> AdminResult<Value> {
    let mut request = client()?
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

fn tool_error(grant: &Grant, payload: &Value, value: &Value) -> AdminError {
    let operation = payload
        .pointer("/params/name")
        .and_then(Value::as_str)
        .or_else(|| payload.get("method").and_then(Value::as_str))
        .unwrap_or("request");
    let mut detail = value
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .pointer("/result/content/0/text")
                .and_then(Value::as_str)
        })
        .unwrap_or("Provider returned a tool error")
        .to_owned();
    for secret in [
        &grant.access_token,
        grant.refresh_token.as_deref().unwrap_or(""),
        &grant.client_secret,
        &grant.verifier,
    ] {
        if !secret.is_empty() {
            detail = detail.replace(secret, "[redacted]");
        }
    }
    let detail: String = detail
        .chars()
        .filter(|c| !c.is_control())
        .take(600)
        .collect();
    AdminError::Unavailable(format!(
        "{} MCP {operation}: {detail}",
        grant.provider.slug()
    ))
}

async fn identity(grant: &mut Grant, session: &mut Option<String>) -> AdminResult<()> {
    let info = match grant.provider {
        Provider::Atlassian => super::payload::atlassian_user(
            &rpc(
                grant,
                session,
                json!({"jsonrpc":"2.0", "id":3,
            "method":"tools/call", "params":{"name":"atlassianUserInfo", "arguments":{}}}),
            )
            .await?,
        )?,
        Provider::Github | Provider::Salesforce => {
            let url = if grant.provider == Provider::Github {
                "https://api.github.com/user".into()
            } else {
                format!("{}/services/oauth2/userinfo", config::salesforce_domain()?)
            };
            let response = client()?
                .get(url)
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
    let id = match grant.provider {
        Provider::Atlassian => info.get("account_id").or_else(|| info.get("accountId")),
        Provider::Github => info.get("id"),
        Provider::Salesforce => info.get("user_id"),
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
    if grant.provider == Provider::Salesforce {
        grant.resource_id = info
            .get("organization_id")
            .and_then(Value::as_str)
            .ok_or_else(|| AdminError::Upstream("Salesforce returned no organization ID".into()))?
            .into();
        grant.resource_name = config::salesforce_domain()?;
    }
    if grant.provider == Provider::Atlassian {
        let sites = super::payload::atlassian_sites(&rpc(grant, session, json!({"jsonrpc":"2.0", "id":4,
            "method":"tools/call", "params":{"name":"getAccessibleAtlassianResources", "arguments":{}}})).await?)?;
        super::site::select(grant, &sites).await?;
    }
    Ok(())
}

pub async fn verify(grant: &mut Grant) -> AdminResult<()> {
    let mut session = None;
    let result = async {
        rpc(
            grant,
            &mut session,
            json!({"jsonrpc":"2.0", "id":1, "method":"initialize",
            "params":{"protocolVersion":"2025-03-26", "capabilities":{},
                "clientInfo":{"name":"Systemprompt connection verification", "version":"1.0"}}}),
        )
        .await?;
        rpc(
            grant,
            &mut session,
            json!({"jsonrpc":"2.0", "method":"notifications/initialized"}),
        )
        .await?;
        let tools = rpc(
            grant,
            &mut session,
            json!({"jsonrpc":"2.0", "id":2, "method":"tools/list"}),
        )
        .await?;
        if tools
            .get("tools")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            return Err(AdminError::Upstream(
                "MCP server has no accessible tools".into(),
            ));
        }
        identity(grant, &mut session).await
    }
    .await;
    // Why: The probe uses a private session, never a user's active Claude session.
    if let Some(session) = session {
        let _cleanup_result = client()?
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
