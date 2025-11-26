pub mod analytics;
pub mod auth;
pub mod bot_detector;
pub mod cors;
pub mod jwt_extractor;
pub mod jwt_metrics;
pub mod rate_limit;
pub mod redirect;
pub mod session;
pub mod trailing_slash;

#[cfg(test)]
mod tests;

pub use analytics::*;
pub use auth::*;
pub use bot_detector::*;
pub use cors::*;
pub use jwt_extractor::*;
pub use jwt_metrics::*;
pub use rate_limit::*;
pub use redirect::*;
pub use session::*;
pub use trailing_slash::*;

// Re-export middleware from core to avoid circular dependency
pub use systemprompt_core_system::{ContextMiddleware, HeaderContextExtractor};
