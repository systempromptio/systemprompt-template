pub mod api;
pub mod auth;
pub use auth::{AuthMode, AuthValidationService, TokenClaims};
pub mod error;
pub mod middleware;
pub mod models;
pub mod repository;
pub mod services;

// Re-export shared types from systemprompt_models
pub use systemprompt_models::{
    ApiError, ApiResponse, AuthError, AuthenticatedUser, BaseRole, CollectionResponse, ColumnInfo,
    Config, DatabaseInfo, DiscoveryResponse, Link, ModelConfig, OAuthClientConfig,
    OAuthServerConfig, PaginationInfo, QueryResult, RequestContext, ResponseLinks, ResponseMeta,
    SingleResponse, TableInfo, TaskMessage, TaskRecord, BEARER_PREFIX,
};

// Re-export local core types
pub use error::CoreError;
pub use models::{
    get_wellknown_metadata, AppContext, Module, ModuleRuntime, ModuleType, Modules,
    ServiceCategory, WellKnownMetadata, WellKnownRoute,
};

// Re-export domain modules for qualified access
pub use systemprompt_models::ai;
pub use systemprompt_models::oauth;

// Re-export database types
pub use systemprompt_core_database::{Database, DbPool};

// Re-export system services
pub use services::{
    bootstrap::initialize_database, broadcaster::CONTEXT_BROADCASTER, install::install_module,
    shared::BinaryPaths, update::update_module, validation::validate_system, SessionAnalytics,
};

// Re-export BroadcastEvent from shared models
pub use systemprompt_models::execution::BroadcastEvent;

// Re-export middleware
pub use middleware::{ContextMiddleware, HeaderContextExtractor};

// Re-export repository types
// Analytics repositories have been removed - use direct SQL queries for analytics

#[macro_export]
macro_rules! register_module_api {
    // Version with explicit module type
    ($module_name:literal, $category:expr, $router_fn:expr, $auth_required:expr, $module_type:expr) => {
        inventory::submit! {
            $crate::models::modules::ModuleApiRegistration {
                module_name: $module_name,
                category: $category,
                module_type: $module_type,
                router_fn: $router_fn,
                auth_required: $auth_required,
            }
        }
    };
    // Default version (backward compatible) - defaults to Regular
    ($module_name:literal, $category:expr, $router_fn:expr, $auth_required:expr) => {
        inventory::submit! {
            $crate::models::modules::ModuleApiRegistration {
                module_name: $module_name,
                category: $category,
                module_type: $crate::models::modules::ModuleType::Regular,
                router_fn: $router_fn,
                auth_required: $auth_required,
            }
        }
    };
}

#[macro_export]
macro_rules! register_wellknown_route {
    // With metadata
    ($path:literal, $handler:expr, $methods:expr, name: $name:literal, description: $desc:literal) => {
        inventory::submit! {
            $crate::models::modules::WellKnownRoute {
                path: $path,
                handler_fn: $handler,
                methods: $methods,
            }
        }

        inventory::submit! {
            $crate::models::wellknown_metadata::WellKnownMetadata::new($path, $name, $desc)
        }
    };

    // Without metadata (backward compatibility)
    ($path:literal, $handler:expr, $methods:expr) => {
        inventory::submit! {
            $crate::models::modules::WellKnownRoute {
                path: $path,
                handler_fn: $handler,
                methods: $methods,
            }
        }
    };
}

// ProxyRoute has been removed - proxy routes are now handled directly in the API module
// This reduces complexity and removes unnecessary abstraction
