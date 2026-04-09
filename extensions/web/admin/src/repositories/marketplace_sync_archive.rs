use systemprompt_web_shared::error::MarketplaceError;
use std::path::Path;

const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024;
pub const MAX_FILE_SIZE: usize = 1024 * 1024;

pub fn validate_upload_size(data: &[u8]) -> Result<(), MarketplaceError> {
    if data.len() > MAX_UPLOAD_SIZE {
        return Err(MarketplaceError::Internal(format!(
            "Upload too large: {} bytes (max {})",
            data.len(),
            MAX_UPLOAD_SIZE
        )));
    }
    Ok(())
}

pub enum ArchiveFormat {
    TarGz,
    Zip,
}

pub fn detect_archive_format(data: &[u8]) -> Result<ArchiveFormat, MarketplaceError> {
    if data.len() < 4 {
        return Err(MarketplaceError::Internal(
            "Upload too small to be a valid archive".to_string(),
        ));
    }
    if data[0] == 0x1f && data[1] == 0x8b {
        Ok(ArchiveFormat::TarGz)
    } else if data[0] == 0x50 && data[1] == 0x4b {
        Ok(ArchiveFormat::Zip)
    } else {
        Err(MarketplaceError::Internal(
            "Unsupported archive format (expected .tar.gz or .zip)".to_string(),
        ))
    }
}

fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), MarketplaceError> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        let path_str = path.to_string_lossy();
        if path_str.contains("..") {
            tracing::warn!(path = %path_str, "Skipping archive entry with path traversal");
            continue;
        }

        let target = dest.join(&*path);
        if !target.starts_with(dest) {
            continue;
        }

        if entry.size() > MAX_FILE_SIZE as u64 {
            tracing::warn!(path = %path_str, size = entry.size(), "Skipping oversized file");
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if entry.header().entry_type().is_file() {
            let mut file = std::fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut file)?;
        }
    }

    Ok(())
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<(), MarketplaceError> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| MarketplaceError::Internal(format!("Failed to open ZIP archive: {e}")))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| MarketplaceError::Internal(format!("Failed to read ZIP entry: {e}")))?;
        let name = file.name().to_string();

        if name.contains("..") {
            tracing::warn!(path = %name, "Skipping zip entry with path traversal");
            continue;
        }

        let target = dest.join(&name);
        if !target.starts_with(dest) {
            continue;
        }

        if file.is_dir() {
            std::fs::create_dir_all(&target)?;
            continue;
        }

        if file.size() > MAX_FILE_SIZE as u64 {
            tracing::warn!(path = %name, size = file.size(), "Skipping oversized file");
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut outfile = std::fs::File::create(&target)?;
        std::io::copy(&mut file, &mut outfile)?;
    }

    Ok(())
}

pub fn extract_archive(data: &[u8]) -> Result<tempfile::TempDir, MarketplaceError> {
    validate_upload_size(data)?;
    let format = detect_archive_format(data)?;
    let tmp = tempfile::TempDir::new()?;

    match format {
        ArchiveFormat::TarGz => extract_tar_gz(data, tmp.path())?,
        ArchiveFormat::Zip => extract_zip(data, tmp.path())?,
    }

    Ok(tmp)
}
