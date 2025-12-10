use anyhow::Result;
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation, ProtocolVersion};
use rmcp::transport::streamable_http_client::StreamableHttpClientTransport;
use rmcp::ServiceExt;
use std::time::Duration;
use tokio::time::timeout;

use super::types::{McpConnectionResult, McpProtocolInfo, ValidationResult};

pub async fn validate_connection(
    service_name: &str,
    host: &str,
    port: u16,
) -> Result<McpConnectionResult> {
    let connection_start = std::time::Instant::now();
    let url = format!("http://{host}:{port}/mcp");

    let connection_result = timeout(
        Duration::from_secs(15),
        connect_and_validate(&url, service_name),
    )
    .await;

    let connection_time = connection_start.elapsed().as_millis() as u32;

    match connection_result {
        Ok(Ok((server_info, validation_result))) => Ok(McpConnectionResult {
            service_name: service_name.to_string(),
            success: validation_result.success,
            error_message: validation_result.error_message,
            connection_time_ms: connection_time,
            server_info: Some(server_info),
            tools_count: validation_result.tools_count,
            validation_type: validation_result.validation_type,
        }),
        Ok(Err(e)) => Ok(McpConnectionResult {
            service_name: service_name.to_string(),
            success: false,
            error_message: Some(e.to_string()),
            connection_time_ms: connection_time,
            server_info: None,
            tools_count: 0,
            validation_type: "connection_failed".to_string(),
        }),
        Err(_) => Ok(McpConnectionResult {
            service_name: service_name.to_string(),
            success: false,
            error_message: Some("Connection timeout".to_string()),
            connection_time_ms: connection_time,
            server_info: None,
            tools_count: 0,
            validation_type: "timeout".to_string(),
        }),
    }
}

pub async fn validate_connection_with_auth(
    service_name: &str,
    host: &str,
    port: u16,
    requires_oauth: bool,
) -> Result<McpConnectionResult> {
    if requires_oauth {
        validate_oauth_service(service_name, host, port).await
    } else {
        validate_connection(service_name, host, port).await
    }
}

async fn validate_oauth_service(
    service_name: &str,
    host: &str,
    port: u16,
) -> Result<McpConnectionResult> {
    let connection_start = std::time::Instant::now();

    let port_check = std::net::TcpStream::connect(format!("{host}:{port}"));
    let connection_time = connection_start.elapsed().as_millis() as u32;

    match port_check {
        Ok(_) => Ok(McpConnectionResult {
            service_name: service_name.to_string(),
            success: true,
            error_message: None,
            connection_time_ms: connection_time,
            server_info: Some(McpProtocolInfo {
                server_name: service_name.to_string(),
                version: "unknown".to_string(),
                protocol_version: "unknown".to_string(),
            }),
            tools_count: 0,
            validation_type: "auth_required".to_string(),
        }),
        Err(e) => Ok(McpConnectionResult {
            service_name: service_name.to_string(),
            success: false,
            error_message: Some(format!("Port not responding: {e}")),
            connection_time_ms: connection_time,
            server_info: None,
            tools_count: 0,
            validation_type: "port_unavailable".to_string(),
        }),
    }
}

async fn connect_and_validate(
    url: &str,
    service_name: &str,
) -> Result<(McpProtocolInfo, ValidationResult)> {
    let transport = StreamableHttpClientTransport::from_uri(url);

    let client_info = ClientInfo {
        protocol_version: ProtocolVersion::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: format!("SystemPrompt MCP Validator for {service_name}"),
            title: None,
            version: "1.0.0".to_string(),
            website_url: None,
            icons: None,
        },
    };

    let client = client_info.serve(transport).await?;

    let server_info = {
        let peer_info = client.peer_info().unwrap();
        McpProtocolInfo {
            server_name: if peer_info.server_info.name.is_empty() {
                service_name.to_string()
            } else {
                peer_info.server_info.name.clone()
            },
            version: if peer_info.server_info.version.is_empty() {
                "1.0.0".to_string()
            } else {
                peer_info.server_info.version.clone()
            },
            protocol_version: peer_info.protocol_version.to_string(),
        }
    };

    let validation_result = match client.list_tools(None).await {
        Ok(tools_response) => {
            let tools_count = tools_response.tools.len();

            if tools_count > 0 {
                ValidationResult {
                    success: true,
                    error_message: None,
                    tools_count,
                    validation_type: "mcp_validated".to_string(),
                }
            } else {
                ValidationResult {
                    success: false,
                    error_message: Some(
                        "No tools returned - service may require authentication".to_string(),
                    ),
                    tools_count: 0,
                    validation_type: "no_tools".to_string(),
                }
            }
        },
        Err(e) => ValidationResult {
            success: false,
            error_message: Some(format!("Tools request failed: {e}")),
            tools_count: 0,
            validation_type: "tools_request_failed".to_string(),
        },
    };

    client.cancel().await?;
    Ok((server_info, validation_result))
}

pub fn rewrite_url_for_internal_use(url: &str) -> Result<String> {
    use systemprompt_core_system::Config;

    let config = Config::global();
    let external_url = &config.api_external_url;
    let internal_url = &config.api_server_url;

    if url.starts_with(external_url) {
        Ok(url.replace(external_url, internal_url))
    } else {
        Ok(url.to_string())
    }
}
