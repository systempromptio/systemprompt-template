use async_trait::async_trait;
use std::sync::Arc;

pub trait AppContext: Send + Sync {
    fn config(&self) -> Arc<dyn ConfigProvider>;
    fn database_handle(&self) -> Arc<dyn DatabaseHandle>;
}

/// Context propagation traits for `RequestContext`
pub trait InjectContextHeaders {
    fn inject_headers(&self, headers: &mut axum::http::HeaderMap);
}

pub trait ContextPropagation {
    fn from_headers(headers: &axum::http::HeaderMap) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn to_headers(&self) -> axum::http::HeaderMap;
}

/// Config provider trait - implemented by config module
pub trait ConfigProvider: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn database_url(&self) -> &str;
    fn system_path(&self) -> &str;
    fn jwt_secret(&self) -> &str;
    fn api_port(&self) -> u16;
}

/// Module registry trait
pub trait ModuleRegistry: Send + Sync {
    fn get_module(&self, name: &str) -> Option<Arc<dyn Module>>;
    fn list_modules(&self) -> Vec<String>;
}

/// Database handle trait - opaque handle to database
/// The actual Database implementation is in the database module
pub trait DatabaseHandle: Send + Sync {
    fn is_connected(&self) -> bool;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Core module trait - minimal interface
#[async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn display_name(&self) -> &str;
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
}

/// API module trait - for modules with REST APIs
#[async_trait]
pub trait ApiModule: Module {
    fn router(&self, ctx: Arc<dyn AppContext>) -> axum::Router;
}
