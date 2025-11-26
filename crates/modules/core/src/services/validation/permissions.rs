use crate::models::context::AppContext;
use anyhow::Result;

pub async fn validate_permissions(_ctx: &AppContext) -> Result<()> {
    Ok(())
}
