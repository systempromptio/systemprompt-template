//! Hosted connector refresh uses provider credentials and fails closed.

use systemprompt_web_admin::connector_oauth::{Grant, Provider, refresh_at};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn grant() -> Grant {
    Grant {
        configuration_binding: String::new(),
        authorization_issuer: String::new(),
        token_auth_method: String::new(),
        user: "fixture-user".into(),
        provider: Provider::Github,
        client: "fixture-app".into(),
        client_secret: "fixture-secret".into(),
        verifier: String::new(),
        access_token: "old".into(),
        refresh_token: Some("fixture-refresh".into()),
        expires_at: 0,
        token_endpoint: String::new(),
        generation: 0,
        session: None,
        auth_method: "oauth".into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".into(),
    }
}

async fn endpoint(body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0; 4096];
            let count = socket.read(&mut buffer).await.unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
            let text = String::from_utf8_lossy(&request);
            if let Some((headers, payload)) = text.split_once("\r\n\r\n") {
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        line.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse().unwrap())
                    })
                    .unwrap();
                if payload.len() >= length {
                    break;
                }
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8(request).unwrap()
    });
    (format!("http://{address}/token"), task)
}

#[test]
fn debug_redacts_connector_credentials() {
    assert_eq!(format!("{:?}", grant()), "Grant { <redacted> }");
}

#[tokio::test]
async fn refresh_sends_the_user_grant_and_preserves_rotation() {
    let (url, mock) =
        endpoint(r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":3600}"#).await;
    let mut credential = grant();
    refresh_at(&mut credential, &url).await.unwrap();
    assert_eq!(credential.access_token, "fresh");
    assert_eq!(credential.refresh_token.as_deref(), Some("rotated"));
    let request = mock.await.unwrap();
    for field in [
        "grant_type=refresh_token",
        "refresh_token=fixture-refresh",
        "client_id=fixture-app",
        "client_secret=fixture-secret",
    ] {
        assert!(request.contains(field));
    }
    assert!(!request.to_lowercase().contains("authorization: bearer"));
}

#[tokio::test]
async fn revoked_grants_are_authentication_failures() {
    let (url, mock) = endpoint(r#"{"error":"invalid_grant"}"#).await;
    let error = refresh_at(&mut grant(), &url).await.unwrap_err();
    assert_eq!(error.status(), axum::http::StatusCode::UNAUTHORIZED);
    mock.await.unwrap();
}

#[tokio::test]
async fn empty_access_tokens_cannot_pass_refresh() {
    let (url, mock) = endpoint(r#"{"access_token":"","expires_in":3600}"#).await;
    assert!(refresh_at(&mut grant(), &url).await.is_err());
    mock.await.unwrap();
}

#[test]
fn atlassian_identity_accepts_wrapped_and_text_tool_results() {
    use serde_json::json;
    use systemprompt_web_admin::connector_oauth::payload::atlassian_user;
    let user = json!({"account_id":"account-123","displayName":"Pilot user"});
    for response in [
        json!({"structuredContent":user}),
        json!({"structuredContent":{"result":{"user":user}}}),
        json!({"structuredContent":{},"content":[{"type":"text","text":user.to_string()}]}),
        json!({"content":[{"type":"text","text":json!({"data":{"account":user}}).to_string()}]}),
    ] {
        assert_eq!(
            atlassian_user(&response).unwrap()["account_id"],
            "account-123"
        );
    }
    assert!(atlassian_user(&json!({"structuredContent":{"id":"site-id"}})).is_err());
    assert!(atlassian_user(&json!({"isError":true,"structuredContent":user})).is_err());
    assert!(
        atlassian_user(
            &json!({"structuredContent":user,"content":[{"text":"{\"accountId\":\"other\"}"}]})
        )
        .is_err()
    );
}

#[test]
fn atlassian_sites_accepts_wrapped_array_and_ignores_empty_structured_content() {
    use serde_json::json;
    use systemprompt_web_admin::connector_oauth::payload::atlassian_sites;
    let sites = json!([{"id":"cloud-123","url":"https://astounddigital.atlassian.net"}]);
    for response in [
        json!({"structuredContent":{"result":sites}}),
        json!({"structuredContent":{},"content":[{"text":sites.to_string()}]}),
    ] {
        assert_eq!(atlassian_sites(&response).unwrap()[0]["id"], "cloud-123");
    }
    assert!(atlassian_sites(&json!({"content":[{"text":"not JSON"}]})).is_err());
}

#[test]
fn atlassian_v2_resources_use_cloud_id_without_url() {
    use serde_json::json;
    use systemprompt_web_admin::connector_oauth::payload::atlassian_sites;
    let response = json!({"content":[{"type":"text","text":json!({"data":{"resources":[
        {"cloudId":"cloud-123","products":[{"id":"jira","access":"read"}]}
    ]}}).to_string()}],"isError":false});
    let sites = atlassian_sites(&response).unwrap();
    assert_eq!(sites[0]["id"], "cloud-123");
}

#[test]
fn atlassian_site_lookup_cannot_target_arbitrary_hosts() {
    use systemprompt_web_admin::connector_oauth::site::tenant_metadata_url;
    assert_eq!(
        tenant_metadata_url("https://astounddigital.atlassian.net")
            .unwrap()
            .as_str(),
        "https://astounddigital.atlassian.net/_edge/tenant_info"
    );
    for site in [
        "http://example.atlassian.net",
        "https://evil.test",
        "https://example.atlassian.net.evil.test",
        "https://user:secret@example.atlassian.net",
        "https://example.atlassian.net/jira",
        "https://example.atlassian.net/?token=secret",
    ] {
        assert!(tenant_metadata_url(site).is_err(), "{site}");
    }
}

#[test]
fn configured_provider_ids_preserve_legacy_wire_names() {
    for id in [
        "atlassian",
        "github",
        "salesforce",
        "salesforce-uat",
        "fourth-mcp",
    ] {
        let provider: Provider = serde_json::from_value(serde_json::json!(id)).unwrap();
        assert_eq!(provider.slug(), id);
        assert_eq!(
            serde_json::to_value(provider).unwrap(),
            serde_json::json!(id)
        );
    }
    for invalid in ["", "../github", "https://evil.test", "provider?x=1", "a/b"] {
        assert!(serde_json::from_value::<Provider>(serde_json::json!(invalid)).is_err());
    }
}

#[test]
fn generic_oauth_cannot_send_credentials_outside_approved_origins() {
    use systemprompt_web_admin::connector_oauth::generic::validate_endpoint;
    let resource = "https://mcp.example.test/resource";
    let origins = vec!["https://login.example.test".to_owned()];
    for endpoint in [
        "https://mcp.example.test/.well-known/oauth-protected-resource",
        "https://login.example.test/token",
        "https://login.example.test/authorize?scope=read",
    ] {
        assert!(validate_endpoint(resource, &origins, endpoint).is_ok());
    }
    for endpoint in [
        "http://login.example.test/token",
        "https://evil.test/token",
        "https://login.example.test.evil.test/token",
        "https://secret@login.example.test/token",
        "https://login.example.test:444/token",
        "https://login.example.test/token#fragment",
    ] {
        assert!(
            validate_endpoint(resource, &origins, endpoint).is_err(),
            "{endpoint}"
        );
    }
}

#[tokio::test]
async fn gateway_does_not_allow_a_request_when_its_policy_database_is_unavailable() {
    use systemprompt::extension::{GatewayDenyKind, GatewayGuardRequest, GatewayRequestGuard};
    use systemprompt_web_admin::gateway_entitlement::RouteEntitlementGuard;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://test:test@localhost:1/test")
        .unwrap();
    pool.close().await;
    let request = GatewayGuardRequest {
        user_id: "00000000-0000-0000-0000-000000000001",
        model: "test-model",
        route_id: Some("test-route"),
        provider: "test-provider",
        streaming: false,
    };
    let denial = RouteEntitlementGuard
        .check(&pool, &request)
        .await
        .unwrap_err();
    assert!(matches!(denial.kind, GatewayDenyKind::Unavailable));
    assert_eq!(denial.retry_after_seconds, 5);
    let unresolved = GatewayGuardRequest {
        route_id: None,
        ..request
    };
    assert!(
        RouteEntitlementGuard
            .check(&pool, &unresolved)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn fourth_configured_connector_completes_oauth_and_mcp_verification() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use systemprompt_web_admin::connector_oauth::{exchange_with_client, generic, verify};
    struct Server(std::process::Child);
    impl Drop for Server {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let root = tempfile::tempdir().unwrap();
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/oauth/server.py");
    let mut server = Server(
        Command::new("python3")
            .arg("-u")
            .arg(script)
            .arg(root.path())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut origin = String::new();
    BufReader::new(server.0.stdout.take().unwrap())
        .read_line(&mut origin)
        .unwrap();
    let origin = origin.trim();
    assert!(origin.starts_with("https://localhost:"));
    let yaml = format!(
        "mcp_servers:\n  fourth-mcp:\n    type: external\n    binary: ''\n    package: null\n    port: 5050\n    endpoint: {origin}/mcp\n    enabled: true\n    display_in_web: false\n    oauth: {{required: false, scopes: [user], audience: mcp, client_id: null}}\n    connector: {{adapter: generic, scopes: [tools:read]}}\n"
    );
    let config = root.path().join("config.yaml");
    std::fs::write(&config, yaml).unwrap();
    systemprompt::loader::ServicesBootstrap::init_from_path(&config).unwrap();
    let ca = reqwest::Certificate::from_pem(&std::fs::read(root.path().join("ca.pem")).unwrap())
        .unwrap();
    let http = reqwest::Client::builder()
        .add_root_certificate(ca)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let provider = Provider::try_from("fourth-mcp".to_owned()).unwrap();
    assert!(provider.configured() && provider.requires_auth());
    let callback = "https://dashboard.example.test/api/public/connectors/fourth-mcp/callback";
    let (url, mut grant) = generic::authorize_with_client(
        generic::Consent {
            user: "active-user",
            state: "test-state",
            verifier: "test-verifier".into(),
        },
        provider,
        callback,
        &http,
    )
    .await
    .unwrap();
    let url = reqwest::Url::parse(&url).unwrap();
    let params = url
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(params["state"], "test-state");
    assert_eq!(params["code_challenge_method"], "S256");
    assert!(!url.as_str().contains("test-verifier"));
    exchange_with_client(&mut grant, "test-code", callback, &http)
        .await
        .unwrap();
    assert_eq!(grant.access_token, "access-token");
    assert!(grant.verifier.is_empty());
    systemprompt_web_admin::connector_oauth::refresh_with_client(&mut grant, &http)
        .await
        .unwrap();
    assert_eq!(grant.refresh_token.as_deref(), Some("refresh-token"));
    verify::verify_with_client(&mut grant, &http).await.unwrap();
    assert_eq!(grant.resource_name, format!("{origin}/mcp"));
    assert!(grant.account_id.is_empty());
    grant.configuration_binding = "changed-config".into();
    assert!(generic::validate_grant(&grant).is_err());
}

#[tokio::test]
async fn subject_attribute_failure_is_not_an_empty_membership() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://test:test@localhost:1/test")
        .unwrap();
    pool.close().await;
    let user = systemprompt::identifiers::UserId::new("00000000-0000-0000-0000-000000000001");
    assert!(
        systemprompt_web_admin::authz::subject_attributes_for(&pool, &user)
            .await
            .is_err()
    );
}
