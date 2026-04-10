mod crud;
mod export;
mod scan;

pub use crud::{
    create_catalog_hook, delete_catalog_hook, get_catalog_hook, list_catalog_hooks,
    update_catalog_hook,
};
pub use export::{catalog_to_detail, read_hook_template, render_tracking_script};
pub use scan::list_file_hooks;

pub const CATEGORY_SYSTEM: &str = "system";
pub const CATEGORY_CUSTOM: &str = "custom";
pub const DEFAULT_VERSION: &str = "1.0.0";
pub const DEFAULT_MATCHER: &str = "*";

#[derive(Debug, thiserror::Error)]
pub enum HookCatalogError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cannot modify system hooks")]
    SystemHookModification,
}
