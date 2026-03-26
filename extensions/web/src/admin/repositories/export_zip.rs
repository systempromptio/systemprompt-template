use std::io::Write;

use zip::write::{SimpleFileOptions, ZipWriter};

use super::export::{PluginBundle, SyncPluginsResponse};

const MAX_ZIP_SIZE: usize = 50 * 1024 * 1024;

pub fn build_plugin_zip(bundle: &PluginBundle) -> Result<Vec<u8>, anyhow::Error> {
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
        zip.start_file(&file.path, opts)?;
        zip.write_all(file.content.as_bytes())?;
    }

    let cursor = zip.finish()?;
    let data = cursor.into_inner();

    if data.len() > MAX_ZIP_SIZE {
        anyhow::bail!(
            "Plugin ZIP too large: {} bytes (max {} bytes / 50MB)",
            data.len(),
            MAX_ZIP_SIZE
        );
    }

    Ok(data)
}

pub fn build_marketplace_zip(response: &SyncPluginsResponse) -> Result<Vec<u8>, anyhow::Error> {
    let buf = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(buf));

    let default_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file(
        &response.marketplace.path,
        default_opts.unix_permissions(0o644),
    )?;
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
            zip.start_file(&path, opts)?;
            zip.write_all(file.content.as_bytes())?;
        }
    }

    let cursor = zip.finish()?;
    let data = cursor.into_inner();

    if data.len() > MAX_ZIP_SIZE {
        anyhow::bail!(
            "Marketplace ZIP too large: {} bytes (max {} bytes / 50MB)",
            data.len(),
            MAX_ZIP_SIZE
        );
    }

    Ok(data)
}

pub fn validate_plugin_name(name: &str) -> Result<(), anyhow::Error> {
    if name.len() > 64 {
        anyhow::bail!("Plugin name too long: {} chars (max 64)", name.len());
    }
    if name != name.to_lowercase() {
        anyhow::bail!("Plugin name must be lowercase: {name}");
    }
    if name.contains(' ') {
        anyhow::bail!("Plugin name must not contain spaces: {name}");
    }
    Ok(())
}
