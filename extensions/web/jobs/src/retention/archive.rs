//! Streams one table window to a gzipped JSON Lines file and records it.
//!
//! `SELECT row_to_json(t)::text` streamed a row at a time is the whole export
//! path: each row is written as one JSON object per line, gzipped as it
//! arrives, and a SHA-256 is taken over the compressed file so the manifest
//! can be verified without inflating it. Memory stays flat whatever the table
//! holds.

use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::ManagedTable;
use super::ledger::{ArchiveRecord, record_archive};
use crate::error::JobError;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Window {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct ArchivedFile {
    pub table: &'static str,
    pub file: String,
    pub rows: i64,
    pub bytes: i64,
    pub sha256: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct Manifest {
    pub tier: &'static str,
    pub period: String,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
    pub written_at: DateTime<Utc>,
    pub files: Vec<ArchivedFile>,
}

pub(crate) struct ArchiveTarget<'a> {
    pub pool: &'a PgPool,
    pub exports_root: &'a Path,
    pub tier: &'static str,
    pub period: &'a str,
    pub window: Window,
}

// Why: the manifest is written last, so a directory with a manifest is a
// complete archive.
pub(crate) async fn archive_tables(
    target: &ArchiveTarget<'_>,
    tables: &[ManagedTable],
) -> Result<Manifest, JobError> {
    let dir = target.exports_root.join(target.tier).join(target.period);
    std::fs::create_dir_all(&dir)?;
    let mut files = Vec::with_capacity(tables.len());
    for table in tables {
        let file = archive_one(target, &dir, table).await?;
        record_archive(
            target.pool,
            &ArchiveRecord {
                tier: target.tier,
                period: target.period,
                table: table.name,
                relative_path: &format!("{}/{}/{}", target.tier, target.period, file.file),
                row_count: file.rows,
                byte_count: file.bytes,
                sha256: &file.sha256,
                window_from: target.window.from,
                window_to: target.window.to,
            },
        )
        .await?;
        files.push(file);
    }
    let manifest = Manifest {
        tier: target.tier,
        period: target.period.to_owned(),
        window_from: target.window.from,
        window_to: target.window.to,
        written_at: Utc::now(),
        files,
    };
    let body =
        serde_json::to_vec_pretty(&manifest).map_err(|error| JobError::other(error.to_string()))?;
    std::fs::write(dir.join("manifest.json"), body)?;
    Ok(manifest)
}

async fn archive_one(
    target: &ArchiveTarget<'_>,
    dir: &Path,
    table: &ManagedTable,
) -> Result<ArchivedFile, JobError> {
    let file_name = format!("{}.jsonl.gz", table.name);
    let path: PathBuf = dir.join(&file_name);
    let mut stream = sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(select_statement(
        table,
        target.window,
    )))
    .fetch(target.pool);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut rows: i64 = 0;
    while let Some(row) = stream.next().await {
        let row = row?;
        encoder.write_all(row.as_bytes())?;
        encoder.write_all(b"\n")?;
        rows += 1;
    }
    drop(stream);
    let compressed = encoder.finish()?;
    let sha256 = hex::encode(Sha256::digest(&compressed));
    std::fs::write(&path, &compressed)?;
    Ok(ArchivedFile {
        table: table.name,
        file: file_name,
        rows,
        bytes: i64::try_from(compressed.len()).unwrap_or(i64::MAX),
        sha256,
    })
}

// Why: a streamed `SELECT`, not `COPY … TO STDOUT`. COPY's text format
// escapes backslashes, so a JSON string containing \" arrives as \\" and the
// line is not valid JSON. `fetch` yields one row at a time, so memory stays
// flat, and the text is exactly what `row_to_json` produced.
fn select_statement(table: &ManagedTable, window: Window) -> String {
    format!(
        "SELECT row_to_json(t)::text FROM {name} t \
         WHERE {col} >= '{from}'::timestamptz AND {col} < '{to}'::timestamptz \
         ORDER BY {col}",
        name = table.name,
        col = table.time_column,
        from = window.from.to_rfc3339(),
        to = window.to.to_rfc3339(),
    )
}

// Why: period strings are zero-padded (`2026-W03`, `2026-03`), so lexical
// order is chronological and "before keep_from" is a string compare.
pub(crate) fn prune_periods(
    exports_root: &Path,
    tier: &str,
    keep_from: &str,
) -> Result<u64, JobError> {
    let dir = exports_root.join(tier);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(0);
    };
    let mut removed = 0;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_dir() && name.as_str() < keep_from {
            std::fs::remove_dir_all(entry.path())?;
            removed += 1;
        }
    }
    Ok(removed)
}
