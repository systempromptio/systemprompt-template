pub mod cimd;
pub mod generation;
pub mod http;
pub mod jwt;
pub mod session_creation;
pub mod validation;
pub mod webauthn;

// Clean exports - organized by purpose
pub use http::BrowserRedirectService;
pub use jwt::AuthService;
pub use session_creation::{AnonymousSessionInfo, SessionCreationService};
pub use webauthn::{JwtTokenValidator, UserCreationService, WebAuthnConfig, WebAuthnService};

// Generation functions
pub use generation::{
    generate_access_token_jti, generate_anonymous_jwt, generate_client_secret, generate_jwt,
    generate_secure_token, hash_client_secret, verify_client_secret, JwtConfig,
};

// Validation functions
pub use validation::{
    validate_any_audience, validate_jwt_token, validate_required_audience, validate_service_access,
};
