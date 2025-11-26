use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AuthError {
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token signature")]
    InvalidSignature,

    #[error("Missing required claim: {claim}")]
    MissingClaim { claim: String },

    #[error("Invalid authorization header")]
    InvalidAuthHeader,

    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {reason}")]
    Forbidden { reason: String },

    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Invalid session: {reason}")]
    InvalidSession { reason: String },

    #[error("Session expired")]
    SessionExpired,

    #[error("Cookie not found")]
    CookieNotFound,

    #[error("Invalid cookie format")]
    InvalidCookieFormat,

    #[error("Database error: {reason}")]
    DatabaseError { reason: String },

    #[error("Internal server error: {reason}")]
    InternalError { reason: String },
}

impl AuthError {
    pub fn reason(&self) -> String {
        self.to_string()
    }

    pub const fn status_code(&self) -> u16 {
        match self {
            Self::AuthenticationFailed { .. }
            | Self::InvalidToken
            | Self::TokenExpired
            | Self::InvalidSignature
            | Self::Unauthorized
            | Self::SessionExpired => 401,
            Self::MissingClaim { .. }
            | Self::InvalidAuthHeader
            | Self::InvalidTokenFormat
            | Self::InvalidSession { .. }
            | Self::CookieNotFound
            | Self::InvalidCookieFormat => 400,
            Self::Forbidden { .. } => 403,
            Self::UserNotFound { .. } | Self::SessionNotFound { .. } => 404,
            Self::DatabaseError { .. } | Self::InternalError { .. } => 500,
        }
    }

    pub const fn is_client_error(&self) -> bool {
        self.status_code() < 500
    }

    pub const fn is_server_error(&self) -> bool {
        self.status_code() >= 500
    }

    pub const fn is_auth_error(&self) -> bool {
        matches!(
            self,
            Self::AuthenticationFailed { .. }
                | Self::InvalidToken
                | Self::TokenExpired
                | Self::InvalidSignature
                | Self::InvalidAuthHeader
                | Self::InvalidTokenFormat
                | Self::Unauthorized
                | Self::SessionExpired
        )
    }

    pub const fn is_permission_error(&self) -> bool {
        matches!(self, Self::Forbidden { .. })
    }

    pub const fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::UserNotFound { .. } | Self::SessionNotFound { .. }
        )
    }
}

impl From<String> for AuthError {
    fn from(reason: String) -> Self {
        Self::InternalError { reason }
    }
}

impl From<&str> for AuthError {
    fn from(reason: &str) -> Self {
        Self::InternalError {
            reason: reason.to_string(),
        }
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError {
            reason: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_status_codes() {
        assert_eq!(
            AuthError::InvalidToken.status_code(),
            401,
            "Invalid token should be 401"
        );
        assert_eq!(
            AuthError::Forbidden {
                reason: "admin only".to_string()
            }
            .status_code(),
            403,
            "Forbidden should be 403"
        );
        assert_eq!(
            AuthError::UserNotFound {
                user_id: "123".to_string()
            }
            .status_code(),
            404,
            "Not found should be 404"
        );
        assert_eq!(
            AuthError::DatabaseError {
                reason: "connection failed".to_string()
            }
            .status_code(),
            500,
            "Database error should be 500"
        );
    }

    #[test]
    fn test_error_classification() {
        let token_expired = AuthError::TokenExpired;
        assert!(token_expired.is_auth_error());
        assert!(!token_expired.is_permission_error());
        assert!(!token_expired.is_not_found());

        let forbidden = AuthError::Forbidden {
            reason: "insufficient permissions".to_string(),
        };
        assert!(!forbidden.is_auth_error());
        assert!(forbidden.is_permission_error());

        let not_found = AuthError::UserNotFound {
            user_id: "123".to_string(),
        };
        assert!(not_found.is_not_found());

        let db_error = AuthError::DatabaseError {
            reason: "connection lost".to_string(),
        };
        assert!(db_error.is_server_error());
        assert!(!db_error.is_client_error());
    }

    #[test]
    fn test_from_string() {
        let err: AuthError = "test error".into();
        match err {
            AuthError::InternalError { reason } => {
                assert_eq!(reason, "test error");
            },
            _ => panic!("Expected InternalError"),
        }
    }
}
