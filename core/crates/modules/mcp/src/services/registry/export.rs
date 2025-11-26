use crate::services::registry::manager::RegistryService;
use crate::ServerManifest;
use anyhow::Result;

pub async fn export_registry_servers() -> Result<Vec<ServerManifest>> {
    let servers = RegistryService::list_servers().await?;
    let mut manifests = Vec::new();

    for server_name in servers {
        let manifest = RegistryService::load_manifest(&server_name).await?;
        manifests.push(manifest);
    }

    Ok(manifests)
}

pub async fn export_enabled_servers() -> Result<Vec<(ServerManifest, u16)>> {
    RegistryService::get_enabled_servers().await
}
