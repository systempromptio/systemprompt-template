use axum::http::{HeaderMap, StatusCode};

#[derive(Debug, Copy, Clone)]
pub struct TokenExtractor;

impl TokenExtractor {
    pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
        Self::extract_from_authorization(headers)
            .or_else(|_| Self::extract_from_mcp_proxy(headers))
            .or_else(|_| Self::extract_from_cookie(headers))
    }

    fn extract_from_authorization(headers: &HeaderMap) -> Result<String, StatusCode> {
        let auth_headers = headers.get_all("authorization");

        if auth_headers.iter().count() == 0 {
            return Err(StatusCode::UNAUTHORIZED);
        }

        for auth_value in &auth_headers {
            let Ok(auth_header) = auth_value.to_str() else {
                continue;
            };

            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                if !token.trim().is_empty() {
                    return Ok(token.to_string());
                }
            }
        }

        Err(StatusCode::UNAUTHORIZED)
    }

    fn extract_from_mcp_proxy(headers: &HeaderMap) -> Result<String, StatusCode> {
        headers
            .get("x-mcp-proxy-auth")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::BAD_REQUEST)
            .map(ToString::to_string)
    }

    fn extract_from_cookie(headers: &HeaderMap) -> Result<String, StatusCode> {
        let cookie_header = headers
            .get("cookie")
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix("access_token=") {
                return Ok(value.to_string());
            }
        }

        Err(StatusCode::UNAUTHORIZED)
    }
}
