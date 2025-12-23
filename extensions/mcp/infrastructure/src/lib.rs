use anyhow::Result;

pub mod config;
pub mod prompts;
pub mod resources;
pub mod server;
pub mod sync;
pub mod tools;

pub use server::InfrastructureServer;

pub async fn create_database_connection() -> Result<systemprompt_core_database::DbPool> {
    use systemprompt_core_system::AppContext;

    let ctx = AppContext::new().await?;
    Ok(ctx.db_pool().clone())
}
