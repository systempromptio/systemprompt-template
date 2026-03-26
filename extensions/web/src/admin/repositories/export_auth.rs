use std::path::Path;

use systemprompt::models::auth::{AuthenticatedUser, JwtAudience, Permission};
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::services::{
    generate_access_token_jti, generate_jwt, JwtConfig, JwtSigningParams,
};

use super::super::types::PlatformPluginConfig;
use super::super::types::PlatformPluginConfigFile;

pub fn generate_plugin_token(
    user_id: &str,
    username: &str,
    email: &str,
) -> Result<String, anyhow::Error> {
    use systemprompt::identifiers::SessionId;

    let jwt_secret = SecretsBootstrap::jwt_secret()?;
    let jwt_issuer = &Config::get()?.jwt_issuer;

    let id = uuid::Uuid::parse_str(user_id)
        .map_err(|e| anyhow::anyhow!("Invalid user ID '{user_id}': {e}"))?;

    let user = AuthenticatedUser {
        id,
        username: username.to_string(),
        email: email.to_string(),
        permissions: vec![Permission::User],
        roles: vec!["user".to_string()],
    };

    let config = JwtConfig {
        permissions: vec![Permission::User],
        audience: vec![JwtAudience::Resource("hook".to_string())],
        expires_in_hours: Some(8760),
        resource: None,
    };

    let jti = generate_access_token_jti();
    let session_id = SessionId::generate();
    let signing = JwtSigningParams {
        secret: jwt_secret,
        issuer: jwt_issuer,
    };

    generate_jwt(&user, config, jti, &session_id, &signing)
}

pub fn load_plugin_configs_by_ids(
    plugins_path: &Path,
    authorized_ids: &std::collections::HashSet<String>,
) -> Result<Vec<(String, PlatformPluginConfig)>, anyhow::Error> {
    let all_plugins = load_all_plugin_configs(plugins_path)?;

    let mut authorized: Vec<(String, PlatformPluginConfig)> = all_plugins
        .iter()
        .filter(|(dir_name, plugin)| plugin.base.enabled && authorized_ids.contains(dir_name))
        .cloned()
        .collect();

    let mut seen_ids: std::collections::HashSet<String> =
        authorized.iter().map(|(_, p)| p.base.id.clone()).collect();
    let mut i = 0;
    while i < authorized.len() {
        let deps = authorized[i].1.depends.clone();
        for dep_id in &deps {
            if seen_ids.contains(dep_id) {
                continue;
            }
            if let Some(dep) = all_plugins.iter().find(|(_, p)| p.base.id == *dep_id) {
                if !dep.1.base.enabled {
                    anyhow::bail!(
                        "Plugin '{}' depends on '{}' which is disabled",
                        authorized[i].1.base.id,
                        dep_id
                    );
                }
                seen_ids.insert(dep_id.clone());
                authorized.push(dep.clone());
            } else {
                anyhow::bail!(
                    "Plugin '{}' depends on '{}' which was not found",
                    authorized[i].1.base.id,
                    dep_id
                );
            }
        }
        i += 1;
    }

    authorized.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(authorized)
}

fn load_all_plugin_configs(
    plugins_path: &Path,
) -> Result<Vec<(String, PlatformPluginConfig)>, anyhow::Error> {
    use anyhow::Context;

    let mut plugins = Vec::new();
    if !plugins_path.exists() {
        return Ok(plugins);
    }

    let mut entries: Vec<_> = std::fs::read_dir(plugins_path)?
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                tracing::warn!("Failed to read plugins directory entry: {}", err);
                None
            }
        })
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
        let plugin_file: PlatformPluginConfigFile = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let dir_name = entry.file_name().to_string_lossy().to_string();
        plugins.push((dir_name, plugin_file.plugin));
    }

    Ok(plugins)
}
