//! Execution-scoped HTTP relay for isolated evaluator containers.
//!
//! The relay is the only network path out of an evaluator container. It
//! forwards exactly two POST routes to the platform — the inference gateway
//! and the evaluation fixture MCP — and only when the request carries an
//! execution credential and a session id, so a container can never reach
//! anything a human operator did not scope it to. It listens on
//! `SYSTEMPROMPT_RELAY_BIND` (default `0.0.0.0:8090`) and forwards to
//! `SYSTEMPROMPT_RELAY_UPSTREAM`, which must be a credential-free HTTP(S)
//! origin.

mod error;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use url::Url;

use error::RelayError;

const DEFAULT_BIND: &str = "0.0.0.0:8090";
const BODY_LIMIT: usize = 17 * 1024 * 1024;
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(600);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const GATEWAY_PATH: &str = "/v1/messages";
const FIXTURE_PATH: &str = "/mcp/evaluation_fixture";
const FIXTURE_UPSTREAM_PATH: &str = "/api/v1/mcp/evaluation_fixture/mcp";
const EXECUTION_TOKEN_PREFIX: &str = "Bearer spexec_";
const SESSION_HEADER: &str = "x-session-id";

// Why: an allow-list, not a deny-list — the relay forwards only what the
// gateway contract needs, so a cookie or proxy header set inside the
// container never crosses into the platform.
const FORWARDED_HEADERS: [&str; 6] = [
    "authorization",
    "content-type",
    "accept",
    "anthropic-version",
    "anthropic-beta",
    SESSION_HEADER,
];

#[derive(Clone)]
struct RelayState {
    upstream: Url,
    client: reqwest::Client,
}

fn upstream_from_env() -> Result<Url, RelayError> {
    let raw = std::env::var("SYSTEMPROMPT_RELAY_UPSTREAM")
        .map_err(|source| RelayError::MissingUpstream { source })?;
    let upstream = Url::parse(&raw)?;
    if !matches!(upstream.scheme(), "http" | "https")
        || !upstream.username().is_empty()
        || upstream.password().is_some()
    {
        return Err(RelayError::InsecureUpstream);
    }
    Ok(upstream)
}

#[tokio::main]
async fn main() -> Result<(), RelayError> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let upstream = upstream_from_env()?;
    let bind = std::env::var("SYSTEMPROMPT_RELAY_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_owned());
    let state = Arc::new(RelayState {
        upstream,
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(UPSTREAM_TIMEOUT)
            .build()?,
    });
    let app = Router::new()
        .fallback(any(forward))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "evaluator relay listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn upstream_path(path: &str) -> Option<&'static str> {
    match path {
        GATEWAY_PATH => Some(GATEWAY_PATH),
        FIXTURE_PATH => Some(FIXTURE_UPSTREAM_PATH),
        _ => None,
    }
}

fn credentials_present(headers: &HeaderMap) -> bool {
    let authorized = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with(EXECUTION_TOKEN_PREFIX));
    authorized && headers.contains_key(SESSION_HEADER)
}

async fn forward(
    State(state): State<Arc<RelayState>>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    let path = uri.path();
    let Some(target_path) =
        upstream_path(path).filter(|_| method == Method::POST && uri.query().is_none())
    else {
        tracing::warn!(%method, path, "relay path denied");
        return (StatusCode::FORBIDDEN, "relay path denied").into_response();
    };
    if !credentials_present(&headers) {
        tracing::warn!(path, "relay request without execution credential");
        return (
            StatusCode::UNAUTHORIZED,
            "execution credential and session required",
        )
            .into_response();
    }
    let mut target = state.upstream.clone();
    target.set_path(target_path);
    target.set_query(None);
    let mut request = state.client.request(method, target).body(body);
    for name in FORWARDED_HEADERS {
        if let Some(value) = headers.get(name) {
            request = request.header(name, value);
        }
    }
    match request.send().await {
        Ok(upstream) => relay_response(upstream).await,
        Err(error) => {
            tracing::error!(error = %error, path, "relay upstream unavailable");
            (StatusCode::BAD_GATEWAY, "upstream unavailable").into_response()
        },
    }
}

async fn relay_response(upstream: reqwest::Response) -> Response {
    let status = upstream.status();
    let response_headers = upstream.headers().clone();
    match upstream.bytes().await {
        Ok(bytes) => {
            let mut response = (status, bytes).into_response();
            for (name, value) in &response_headers {
                if !matches!(name.as_str(), "content-length" | "connection") {
                    response.headers_mut().insert(name, value.clone());
                }
            }
            response
        },
        Err(error) => {
            tracing::error!(error = %error, "relay upstream body failed");
            (StatusCode::BAD_GATEWAY, "upstream body failed").into_response()
        },
    }
}
