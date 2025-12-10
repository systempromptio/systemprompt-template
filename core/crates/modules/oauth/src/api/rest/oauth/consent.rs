#![allow(unused_qualifications)]

use crate::repository::OAuthRepository;
use anyhow::Result;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConsentQuery {
    pub client_id: String,
    pub scope: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConsentResponse {
    pub client_name: String,
    pub scopes: Vec<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConsentError {
    pub error: String,
    pub error_description: Option<String>,
}

pub async fn handle_consent_get(
    Query(params): Query<ConsentQuery>,
    State(ctx): State<systemprompt_core_system::AppContext>,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    match get_consent_info(&repo, &params).await {
        Ok(consent_info) => {
            let consent_form = generate_consent_page(&consent_info, &params);
            Html(consent_form).into_response()
        },
        Err(error) => {
            let error = ConsentError {
                error: "invalid_request".to_string(),
                error_description: Some(error.to_string()),
            };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct ConsentRequest {
    pub client_id: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub decision: String, // "allow" or "deny"
}

pub async fn handle_consent_post(
    State(_ctx): State<systemprompt_core_system::AppContext>,
    Json(decision): Json<ConsentRequest>,
) -> impl IntoResponse {
    match process_consent_decision(decision).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            let error = ConsentError {
                error: "server_error".to_string(),
                error_description: Some(error.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        },
    }
}

async fn get_consent_info(
    repo: &OAuthRepository,
    params: &ConsentQuery,
) -> Result<ConsentResponse> {
    let client = repo
        .find_client(&params.client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    let requested_scopes = match &params.scope {
        Some(scope_str) if !scope_str.is_empty() => scope_str
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect(),
        _ => {
            return Err(anyhow::anyhow!("Scope parameter is required"));
        },
    };

    for scope in &requested_scopes {
        if !client.scopes.contains(scope) {
            return Err(anyhow::anyhow!("Invalid scope: {scope}"));
        }
    }

    Ok(ConsentResponse {
        client_name: client.client_name,
        scopes: requested_scopes,
        state: params.state.clone(),
    })
}

fn generate_consent_page(consent_info: &ConsentResponse, params: &ConsentQuery) -> String {
    let scope_items = generate_scope_items(&consent_info.scopes);
    let scope_value = params.scope.as_deref().unwrap_or_default();
    let state_value = params.state.as_deref().unwrap_or_default();

    let template_vars = ConsentTemplateVars {
        client_name: &consent_info.client_name,
        scope_items: &scope_items,
        client_id: &params.client_id,
        scope: scope_value,
        state: state_value,
    };

    render_consent_template(&template_vars)
}

struct ConsentTemplateVars<'a> {
    client_name: &'a str,
    scope_items: &'a str,
    client_id: &'a str,
    scope: &'a str,
    state: &'a str,
}

fn generate_scope_items(scopes: &[String]) -> String {
    scopes
        .iter()
        .map(|scope| {
            format!(
                "<div class='scope-item'>• Access your {} data</div>",
                scope.replace('_', " ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_consent_template(vars: &ConsentTemplateVars) -> String {
    format!(
        "{}{}{}{}{}{}\n</html>",
        get_html_head(),
        get_body_start(),
        get_consent_header(vars.client_name),
        get_scopes_section(vars.scope_items),
        get_buttons_section(),
        get_javascript_section(vars.client_id, vars.scope, vars.state)
    )
}

const fn get_html_head() -> &'static str {
    r"<!DOCTYPE html>
<html>
<head>
    <title>Grant Access</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .container { max-width: 500px; margin: 0 auto; padding: 20px; border: 1px solid #ddd; border-radius: 8px; }
        .scopes { margin: 20px 0; }
        .scope-item { margin: 10px 0; padding: 10px; background-color: #f5f5f5; border-radius: 4px; }
        .buttons { margin: 20px 0; }
        button { padding: 12px 24px; margin: 5px; border: none; border-radius: 4px; cursor: pointer; }
        .allow { background-color: #4CAF50; color: white; }
        .deny { background-color: #f44336; color: white; }
        .allow:hover { background-color: #45a049; }
        .deny:hover { background-color: #da190b; }
    </style>
</head>"
}

const fn get_body_start() -> &'static str {
    "<body>\n    <div class=\"container\">\n        <h2>Grant Access</h2>"
}

fn get_consent_header(client_name: &str) -> String {
    format!("        <p><strong>{client_name}</strong> is requesting access to your account.</p>")
}

fn get_scopes_section(scope_items: &str) -> String {
    format!(
        "        <div class=\"scopes\">\n            <h3>This application will be able to:</h3>\n            {scope_items}\n        </div>"
    )
}

const fn get_buttons_section() -> &'static str {
    r#"        <div class="buttons">
            <button onclick="makeDecision('allow')" class="allow">Allow</button>
            <button onclick="makeDecision('deny')" class="deny">Cancel</button>
        </div>
    </div>"#
}

fn get_javascript_section(client_id: &str, scope: &str, state: &str) -> String {
    format!(
        r"    <script>
        function makeDecision(decision) {{
            fetch('/oauth/consent', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{
                    client_id: '{client_id}',
                    scope: '{scope}',
                    state: '{state}',
                    decision: decision
                }})
            }}).then(response => {{
                if (response.ok) {{
                    if (decision === 'allow') {{
                        window.location.href = '/api/v1/core/oauth/authorize?response_type=code&client_id={client_id}&scope={scope}&state={state}&consent=granted';
                    }} else {{
                        window.close();
                    }}
                }} else {{
                    alert('Error processing consent decision');
                }}
            }});
        }}
    </script>
</body>"
    )
}

async fn process_consent_decision(decision: ConsentRequest) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "status": "processed",
        "decision": decision.decision,
        "client_id": decision.client_id
    }))
}
