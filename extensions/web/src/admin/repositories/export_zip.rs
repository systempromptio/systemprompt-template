use std::io::Write;

use zip::write::{SimpleFileOptions, ZipWriter};

use super::export::{PluginBundle, SyncPluginsResponse};
use crate::error::MarketplaceError;

const MAX_ZIP_SIZE: usize = 50 * 1024 * 1024;

pub fn build_plugin_zip(bundle: &PluginBundle) -> Result<Vec<u8>, MarketplaceError> {
    let buf = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(buf));

    let default_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for file in &bundle.files {
        let opts = if file.executable {
            default_opts.unix_permissions(0o755)
        } else {
            default_opts.unix_permissions(0o644)
        };
        zip.start_file(&file.path, opts)
            .map_err(|e| MarketplaceError::Internal(format!("ZIP write error: {e}")))?;
        zip.write_all(file.content.as_bytes())?;
    }

    let cursor = zip
        .finish()
        .map_err(|e| MarketplaceError::Internal(format!("ZIP finish error: {e}")))?;
    let data = cursor.into_inner();

    if data.len() > MAX_ZIP_SIZE {
        return Err(MarketplaceError::BadRequest(format!(
            "Plugin ZIP too large: {} bytes (max {} bytes / 50MB)",
            data.len(),
            MAX_ZIP_SIZE
        )));
    }

    Ok(data)
}

pub fn build_marketplace_zip(response: &SyncPluginsResponse) -> Result<Vec<u8>, MarketplaceError> {
    let buf = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(buf));

    let default_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file(
        &response.marketplace.path,
        default_opts.unix_permissions(0o644),
    )
    .map_err(|e| MarketplaceError::Internal(format!("ZIP write error: {e}")))?;
    zip.write_all(response.marketplace.content.as_bytes())?;

    for bundle in &response.plugins {
        let prefix = format!("plugins/{}", bundle.id);
        for file in &bundle.files {
            let path = format!("{}/{}", prefix, file.path);
            let opts = if file.executable {
                default_opts.unix_permissions(0o755)
            } else {
                default_opts.unix_permissions(0o644)
            };
            zip.start_file(&path, opts)
                .map_err(|e| MarketplaceError::Internal(format!("ZIP write error: {e}")))?;
            zip.write_all(file.content.as_bytes())?;
        }
    }

    let cursor = zip
        .finish()
        .map_err(|e| MarketplaceError::Internal(format!("ZIP finish error: {e}")))?;
    let data = cursor.into_inner();

    if data.len() > MAX_ZIP_SIZE {
        return Err(MarketplaceError::Internal(format!(
            "Marketplace ZIP too large: {} bytes (max {} bytes / 50MB)",
            data.len(),
            MAX_ZIP_SIZE
        )));
    }

    Ok(data)
}

pub struct CoworkExportParams<'a> {
    pub response: &'a SyncPluginsResponse,
    pub username: &'a str,
    pub email: &'a str,
    pub platform_url: &'a str,
    pub user_id: &'a systemprompt::identifiers::UserId,
}

pub fn build_cowork_plugin_zip(params: &CoworkExportParams<'_>) -> Result<Vec<u8>, MarketplaceError> {
    let cowork_token =
        super::plugin_jwt::generate_plugin_token(params.user_id, params.email, "cowork-bundle")?;

    let mut merged_files = collect_merged_files(params.response);
    append_merged_mcp_config(params.response, &mut merged_files);
    append_env_plugin(&cowork_token, params.platform_url, &mut merged_files);
    append_cowork_manifest(params.username, params.email, &mut merged_files);

    let description = format!("{} marketplace hooks", params.username);
    let merged_files = super::cowork_sanitize::sanitize_for_cowork(
        &merged_files,
        params.platform_url,
        &cowork_token,
        &description,
    )?;

    let counts = super::export_validation::compute_bundle_counts(&merged_files);
    let merged_bundle = PluginBundle {
        id: "cowork-bundle".to_string(),
        name: params.username.to_string(),
        description: format!("All plugins from {0} systemprompt.io marketplace", params.username),
        version: "1.0.0".to_string(),
        files: merged_files,
        counts,
    };

    build_plugin_zip(&merged_bundle)
}

fn collect_merged_files(response: &SyncPluginsResponse) -> Vec<super::export::PluginFile> {
    use super::export::PluginFile;

    let mut files: Vec<PluginFile> = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for bundle in &response.plugins {
        for file in &bundle.files {
            match file.path.as_str() {
                ".mcp.json" | "hooks/hooks.json" | ".env.plugin" | ".claude-plugin/plugin.json" => {}
                _ => {
                    if seen_paths.insert(file.path.clone()) {
                        files.push(PluginFile {
                            path: file.path.clone(),
                            content: file.content.clone(),
                            executable: file.executable,
                        });
                    }
                }
            }
        }
    }

    files
}

fn append_merged_mcp_config(
    response: &SyncPluginsResponse,
    files: &mut Vec<super::export::PluginFile>,
) {
    use super::export::{McpConfigFile, McpServerEntry, PluginFile};

    let mut merged_servers = std::collections::HashMap::<String, McpServerEntry>::new();

    for bundle in &response.plugins {
        for file in &bundle.files {
            if file.path == ".mcp.json" {
                if let Ok(mcp_config) = serde_json::from_str::<McpConfigFile>(&file.content) {
                    merged_servers.extend(mcp_config.mcp_servers);
                }
            }
        }
    }

    if merged_servers.is_empty() {
        return;
    }

    let mcp_config = McpConfigFile {
        mcp_servers: merged_servers,
    };
    if let Ok(content) = serde_json::to_string_pretty(&mcp_config) {
        files.push(PluginFile {
            path: ".mcp.json".to_string(),
            content,
            executable: false,
        });
    }
}

fn append_env_plugin(
    token: &str,
    platform_url: &str,
    files: &mut Vec<super::export::PluginFile>,
) {
    files.push(super::export::PluginFile {
        path: ".env.plugin".to_string(),
        content: format!("SYSTEMPROMPT_PLUGIN_TOKEN={token}\nSYSTEMPROMPT_API_URL={platform_url}\n"),
        executable: false,
    });
}

fn append_cowork_manifest(
    username: &str,
    email: &str,
    files: &mut Vec<super::export::PluginFile>,
) {
    use super::export::{ManifestAuthor, PluginManifest};

    let sanitized: String = username
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let slug = sanitized
        .trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let plugin_name = format!("{slug}-marketplace");

    let manifest = PluginManifest {
        name: plugin_name,
        description: format!("All plugins from {username} systemprompt.io marketplace"),
        version: "1.0.0".to_string(),
        author: Some(ManifestAuthor {
            name: username.to_string(),
            email: email.to_string(),
        }),
        hooks: None,
        keywords: Vec::new(),
    };

    if let Ok(content) = serde_json::to_string_pretty(&manifest) {
        files.push(super::export::PluginFile {
            path: ".claude-plugin/plugin.json".to_string(),
            content,
            executable: false,
        });
    }
}

pub fn validate_plugin_name(name: &str) -> Result<(), MarketplaceError> {
    if name.len() > 64 {
        return Err(MarketplaceError::Internal(format!(
            "Plugin name too long: {} chars (max 64)",
            name.len()
        )));
    }
    if name != name.to_lowercase() {
        return Err(MarketplaceError::Internal(format!(
            "Plugin name must be lowercase: {name}"
        )));
    }
    if name.contains(' ') {
        return Err(MarketplaceError::Internal(format!(
            "Plugin name must not contain spaces: {name}"
        )));
    }
    Ok(())
}
