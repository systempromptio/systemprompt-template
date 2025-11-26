use axum::http::HeaderMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMethod {
    AuthorizationHeader,
    McpProxyHeader,
    Cookie,
}

impl fmt::Display for ExtractionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthorizationHeader => write!(f, "Authorization header"),
            Self::McpProxyHeader => write!(f, "MCP proxy header"),
            Self::Cookie => write!(f, "Cookie"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenExtractor {
    fallback_chain: Vec<ExtractionMethod>,
    cookie_name: String,
    mcp_header_name: String,
}

impl TokenExtractor {
    pub fn new(fallback_chain: Vec<ExtractionMethod>) -> Self {
        Self {
            fallback_chain,
            cookie_name: "access_token".to_string(),
            mcp_header_name: "x-mcp-proxy-auth".to_string(),
        }
    }

    pub fn with_cookie_name(mut self, name: String) -> Self {
        self.cookie_name = name;
        self
    }

    pub fn with_mcp_header_name(mut self, name: String) -> Self {
        self.mcp_header_name = name;
        self
    }

    pub fn standard() -> Self {
        Self::new(vec![
            ExtractionMethod::AuthorizationHeader,
            ExtractionMethod::McpProxyHeader,
            ExtractionMethod::Cookie,
        ])
    }

    pub fn browser_only() -> Self {
        Self::new(vec![
            ExtractionMethod::AuthorizationHeader,
            ExtractionMethod::Cookie,
        ])
    }

    pub fn api_only() -> Self {
        Self::new(vec![ExtractionMethod::AuthorizationHeader])
    }

    pub fn chain(&self) -> &[ExtractionMethod] {
        &self.fallback_chain
    }

    pub fn extract(&self, headers: &HeaderMap) -> Result<String, TokenExtractionError> {
        for method in &self.fallback_chain {
            match method {
                ExtractionMethod::AuthorizationHeader => {
                    if let Ok(token) = Self::extract_from_authorization(headers) {
                        return Ok(token);
                    }
                },
                ExtractionMethod::McpProxyHeader => {
                    if let Ok(token) = self.extract_from_mcp_proxy(headers) {
                        return Ok(token);
                    }
                },
                ExtractionMethod::Cookie => {
                    if let Ok(token) = self.extract_from_cookie(headers) {
                        return Ok(token);
                    }
                },
            }
        }

        Err(TokenExtractionError::NoTokenFound)
    }

    pub fn extract_from_authorization(
        headers: &HeaderMap,
    ) -> Result<String, TokenExtractionError> {
        let auth_headers = headers.get_all("authorization");

        if auth_headers.iter().count() == 0 {
            return Err(TokenExtractionError::MissingAuthorizationHeader);
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

        Err(TokenExtractionError::InvalidAuthorizationFormat)
    }

    pub fn extract_from_mcp_proxy(
        &self,
        headers: &HeaderMap,
    ) -> Result<String, TokenExtractionError> {
        let header_value = headers
            .get(&self.mcp_header_name)
            .ok_or(TokenExtractionError::MissingMcpProxyHeader)?;

        let auth_header = header_value
            .to_str()
            .map_err(|_| TokenExtractionError::InvalidMcpProxyFormat)?;

        auth_header
            .strip_prefix("Bearer ")
            .ok_or(TokenExtractionError::InvalidMcpProxyFormat)
            .map(ToString::to_string)
    }

    pub fn extract_from_cookie(&self, headers: &HeaderMap) -> Result<String, TokenExtractionError> {
        let cookie_header = headers
            .get("cookie")
            .ok_or(TokenExtractionError::MissingCookie)?
            .to_str()
            .map_err(|_| TokenExtractionError::InvalidCookieFormat)?;

        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            let cookie_prefix = format!("{}=", self.cookie_name);
            if let Some(value) = cookie.strip_prefix(&cookie_prefix) {
                if !value.is_empty() {
                    return Ok(value.to_string());
                }
            }
        }

        Err(TokenExtractionError::TokenNotFoundInCookie)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenExtractionError {
    NoTokenFound,
    MissingAuthorizationHeader,
    InvalidAuthorizationFormat,
    MissingMcpProxyHeader,
    InvalidMcpProxyFormat,
    MissingCookie,
    InvalidCookieFormat,
    TokenNotFoundInCookie,
}

impl fmt::Display for TokenExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTokenFound => write!(f, "No token found in request"),
            Self::MissingAuthorizationHeader => {
                write!(f, "Missing Authorization header")
            },
            Self::InvalidAuthorizationFormat => {
                write!(
                    f,
                    "Invalid Authorization header format (expected 'Bearer <token>')"
                )
            },
            Self::MissingMcpProxyHeader => {
                write!(f, "Missing MCP proxy authorization header")
            },
            Self::InvalidMcpProxyFormat => {
                write!(
                    f,
                    "Invalid MCP proxy header format (expected 'Bearer <token>')"
                )
            },
            Self::MissingCookie => write!(f, "Missing cookie header"),
            Self::InvalidCookieFormat => write!(f, "Invalid cookie format"),
            Self::TokenNotFoundInCookie => write!(f, "Token not found in cookies"),
        }
    }
}

impl Error for TokenExtractionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, Request};

    #[test]
    fn test_extract_from_authorization_success() {
        let extractor = TokenExtractor::standard();
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test_token_123"),
        );

        let token = extractor.extract_from_authorization(headers).unwrap();
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_extract_from_authorization_missing() {
        let extractor = TokenExtractor::standard();
        let request = Request::builder().body(()).unwrap();
        let headers = request.headers();

        let result = extractor.extract_from_authorization(headers);
        assert_eq!(
            result,
            Err(TokenExtractionError::MissingAuthorizationHeader)
        );
    }

    #[test]
    fn test_extract_from_authorization_invalid_format() {
        let extractor = TokenExtractor::standard();
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Basic dGVzdDp0ZXN0"),
        );

        let result = extractor.extract_from_authorization(headers);
        assert_eq!(
            result,
            Err(TokenExtractionError::InvalidAuthorizationFormat)
        );
    }

    #[test]
    fn test_extract_from_cookie_success() {
        let extractor = TokenExtractor::standard();
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert(
            "cookie",
            HeaderValue::from_static("access_token=cookie_token_456; other=value"),
        );

        let token = extractor.extract_from_cookie(headers).unwrap();
        assert_eq!(token, "cookie_token_456");
    }

    #[test]
    fn test_extract_from_cookie_missing() {
        let extractor = TokenExtractor::standard();
        let request = Request::builder().body(()).unwrap();
        let headers = request.headers();

        let result = extractor.extract_from_cookie(headers);
        assert_eq!(result, Err(TokenExtractionError::MissingCookie));
    }

    #[test]
    fn test_extract_fallback_authorization_to_cookie() {
        let extractor = TokenExtractor::standard();
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert(
            "cookie",
            HeaderValue::from_static("access_token=fallback_token; other=value"),
        );

        let token = extractor.extract(headers).unwrap();
        assert_eq!(token, "fallback_token");
    }

    #[test]
    fn test_extract_fallback_mcp_to_cookie() {
        let extractor = TokenExtractor::standard();
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert(
            "x-mcp-proxy-auth",
            HeaderValue::from_static("Bearer mcp_token"),
        );
        headers.insert(
            "cookie",
            HeaderValue::from_static("access_token=cookie_token; other=value"),
        );

        let token = extractor.extract(headers).unwrap();
        assert_eq!(token, "mcp_token");
    }

    #[test]
    fn test_extract_all_fail() {
        let extractor = TokenExtractor::standard();
        let request = Request::builder().body(()).unwrap();
        let headers = request.headers();

        let result = extractor.extract(headers);
        assert_eq!(result, Err(TokenExtractionError::NoTokenFound));
    }

    #[test]
    fn test_browser_only_chain() {
        let extractor = TokenExtractor::browser_only();
        assert_eq!(
            extractor.chain(),
            &[
                ExtractionMethod::AuthorizationHeader,
                ExtractionMethod::Cookie,
            ]
        );
    }

    #[test]
    fn test_custom_cookie_name() {
        let extractor = TokenExtractor::standard().with_cookie_name("my_token".to_string());
        let mut request = Request::builder().body(()).unwrap();
        let headers = request.headers_mut();
        headers.insert(
            "cookie",
            HeaderValue::from_static("my_token=custom_token; other=value"),
        );

        let token = extractor.extract_from_cookie(headers).unwrap();
        assert_eq!(token, "custom_token");
    }
}
