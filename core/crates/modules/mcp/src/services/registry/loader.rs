pub use crate::services::deployment::DeploymentService;
pub use crate::services::registry::manager::RegistryService;

// Legacy compatibility - redirect to new service methods
use crate::ServerManifest;
use anyhow::Result;

pub async fn load_registry(_registry_path: &str) -> Result<Vec<ServerManifest>> {
    let servers = RegistryService::list_servers().await?;
    let mut manifests = Vec::new();

    for server_name in servers {
        let manifest = RegistryService::load_manifest(&server_name).await?;
        manifests.push(manifest);
    }

    Ok(manifests)
}

pub async fn reload_registry(registry_path: &str) -> Result<Vec<ServerManifest>> {
    load_registry(registry_path).await
}

pub async fn validate_registry_file(_registry_path: &str) -> Result<()> {
    RegistryService::validate_registry().await
}
