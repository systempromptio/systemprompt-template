//! Renders every plane's export into one zip.

use std::io::{Cursor, Write};

use sqlx::PgPool;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use super::manifest::{ExportedPlane, build_manifest, render_manifest};
use super::{MANIFEST_FILE, TREE_PREFIX};
use crate::error::{AdminError, AdminResult};
use crate::repositories::sync::registry::planes;

/// A finished export, ready to send as an attachment.
#[derive(Debug, Clone)]
pub struct ExportZip {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub planes: usize,
    pub rows: usize,
}

fn zip_error(e: zip::result::ZipError) -> AdminError {
    AdminError::invalid("archive could not be written", e)
}

// Why: lint-ok: unused-pub — the entry point of the /admin/sync export, whose
// handler lands with the Stage-3 admin port.
pub async fn build_export_zip(pool: &PgPool, actor: &str) -> AdminResult<ExportZip> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let registry = planes();
    let mut exported = Vec::new();
    let mut rows = 0usize;
    for plane in &registry {
        // Why: a plane with no file form has nothing to put in the tree.
        let Some(export) = plane.export(pool).await? else {
            continue;
        };
        writer
            .start_file(format!("{TREE_PREFIX}{}", plane.source_file()), options)
            .map_err(zip_error)?;
        writer
            .write_all(export.body.as_bytes())
            .map_err(|e| AdminError::invalid("archive could not be written", e))?;
        rows += export.row_count;
        exported.push(ExportedPlane {
            plane: plane.as_ref(),
            row_count: export.row_count,
        });
    }
    let manifest = build_manifest(pool, actor, &exported).await?;
    writer
        .start_file(MANIFEST_FILE, options)
        .map_err(zip_error)?;
    writer
        .write_all(render_manifest(&manifest)?.as_bytes())
        .map_err(|e| AdminError::invalid("archive could not be written", e))?;
    let bytes = writer.finish().map_err(zip_error)?.into_inner();
    Ok(ExportZip {
        filename: format!(
            "configuration-v{}-{}.zip",
            manifest.release,
            manifest.exported_at.format("%Y%m%d-%H%M")
        ),
        bytes,
        planes: exported.len(),
        rows,
    })
}
