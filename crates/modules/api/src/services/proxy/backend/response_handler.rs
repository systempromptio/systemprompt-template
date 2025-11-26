use axum::{body::Body, http::StatusCode, response::Response};
use futures_util::TryStreamExt;

#[derive(Debug, Clone, Copy)]
pub struct ResponseHandler;

impl ResponseHandler {
    pub async fn build_response(response: reqwest::Response) -> Result<Response<Body>, StatusCode> {
        let status_code = response.status().as_u16();
        let axum_status =
            StatusCode::from_u16(status_code).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response_headers = response.headers().clone();

        // Create streaming body with better error handling
        let stream = response.bytes_stream().map_err(|e| {
            eprintln!("ERROR: Stream error: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        });
        let body = Body::from_stream(stream);

        let mut axum_response = Response::builder().status(axum_status);

        for (key, value) in response_headers.iter() {
            let key_str = key.as_str();
            if let Ok(value_str) = value.to_str() {
                if Self::should_preserve_header(key_str) {
                    axum_response = axum_response.header(key_str, value_str);
                }
            }
        }

        // Ensure connection keep-alive headers for streaming
        axum_response = axum_response
            .header("connection", "keep-alive")
            .header("cache-control", "no-cache");

        axum_response
            .body(body)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn should_preserve_header(key: &str) -> bool {
        match key.to_lowercase().as_str() {
            "content-type" | "content-length" | "content-encoding" | "transfer-encoding" => true,
            "connection" | "keep-alive" | "cache-control" => true,
            "access-control-allow-origin"
            | "access-control-allow-methods"
            | "access-control-allow-headers" => true,
            "set-cookie" | "cookie" => true,
            "x-request-id" | "x-correlation-id" => true,
            header if header.starts_with("x-mcp-") => true,
            "x-accel-buffering" => true,
            "host" | "authorization" | "proxy-authorization" | "upgrade" | "te" => false,
            _ => true,
        }
    }

    pub fn handle_request_error(e: reqwest::Error) -> StatusCode {
        eprintln!(
            "ERROR: Failed to connect to backend: {}. Is the MCP server running?",
            e
        );
        StatusCode::BAD_GATEWAY
    }
}
