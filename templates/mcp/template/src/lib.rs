use anyhow::Result;

pub mod prompts;
pub mod server;
pub mod tools;

pub use server::TemplateServer;

pub async fn create_database_connection() -> Result<systemprompt_core_database::DbPool> {
    use systemprompt_core_system::AppContext;

    let ctx = AppContext::new().await?;
    Ok(ctx.db_pool().clone())
}
