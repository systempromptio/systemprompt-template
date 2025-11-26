use crate::models::context::AppContext;
use anyhow::Result;

pub mod database;
pub mod permissions;
pub mod schemas;
pub mod seeds;

use database::validate_database;
use permissions::validate_permissions;

pub async fn validate_system(ctx: &AppContext) -> Result<()> {
    validate_database(ctx).await?;
    validate_permissions(ctx).await?;
    Ok(())
}
