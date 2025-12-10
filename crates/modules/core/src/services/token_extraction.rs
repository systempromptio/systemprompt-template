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

    pub fn extract_from_authorization(headers: &HeaderMap) -> Result<String, TokenExtractionError> {
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
