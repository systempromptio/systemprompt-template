#![allow(clippy::pedantic)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::explicit_deref_methods)]

pub mod api;
pub mod models;
pub mod queries;
pub mod repository;
pub mod services;
pub mod templates;
pub mod traits;
pub use models::*;
pub use repository::OAuthRepository;
pub use services::validation::jwt::validate_jwt_token;
pub use services::{AnonymousSessionInfo, BrowserRedirectService, SessionCreationService};
pub use traits::{extract_bearer_token, extract_cookie_token, TokenValidator};

// Re-export shared auth types from systemprompt_core
pub use systemprompt_models::auth::{AuthError, AuthenticatedUser, BEARER_PREFIX};
