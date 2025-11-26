use anyhow::Result;
use systemprompt_models::repository::ServiceRepository;

pub async fn update_service_status(
    db_pool: &systemprompt_core_database::DbPool,
    name: &str,
    status: &str,
) -> Result<()> {
    let repo = ServiceRepository::new(db_pool.clone());
    repo.update_service_status(name, status).await
}

pub async fn clear_service_pid(
    db_pool: &systemprompt_core_database::DbPool,
    name: &str,
) -> Result<()> {
    let repo = ServiceRepository::new(db_pool.clone());
    repo.clear_service_pid(name).await
}
