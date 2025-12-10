use axum::http::HeaderMap;

#[derive(Debug, Clone)]
pub struct CookieExtractor {
    cookie_name: String,
}

impl Default for CookieExtractor {
    fn default() -> Self {
        Self::new(Self::DEFAULT_COOKIE_NAME)
    }
}

impl CookieExtractor {
    pub const DEFAULT_COOKIE_NAME: &'static str = "access_token";

    pub fn new(cookie_name: impl Into<String>) -> Self {
        Self {
            cookie_name: cookie_name.into(),
        }
    }

    pub fn extract(&self, headers: &HeaderMap) -> Result<String, CookieExtractionError> {
        self.extract_internal(headers)
    }

    pub fn extract_access_token(headers: &HeaderMap) -> Result<String, CookieExtractionError> {
        Self::default().extract(headers)
    }

    fn extract_internal(&self, headers: &HeaderMap) -> Result<String, CookieExtractionError> {
        let cookie_header = headers
            .get("cookie")
            .ok_or(CookieExtractionError::MissingCookie)?
            .to_str()
            .map_err(|_| CookieExtractionError::InvalidCookieFormat)?;

        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            let cookie_prefix = format!("{}=", self.cookie_name);
            if let Some(value) = cookie.strip_prefix(&cookie_prefix) {
                if !value.is_empty() {
                    return Ok(value.to_string());
                }
            }
        }

        Err(CookieExtractionError::TokenNotFoundInCookie)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieExtractionError {
    MissingCookie,
    InvalidCookieFormat,
    TokenNotFoundInCookie,
}

impl std::fmt::Display for CookieExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCookie => write!(f, "Missing cookie header"),
            Self::InvalidCookieFormat => write!(f, "Invalid cookie format"),
            Self::TokenNotFoundInCookie => {
                write!(f, "Access token not found in cookies")
            },
        }
    }
}

impl std::error::Error for CookieExtractionError {}
