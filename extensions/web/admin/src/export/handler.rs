//! `GET /admin/export/{dataset}` and `GET /admin/export/{dataset}/preview`.
//!
//! The file endpoint writes the dataset's rows in the requested format; the
//! preview answers with the counts the dialog shows before a download —
//! rows, columns, cells, and whether the cap bit. Both read the same query
//! string the page was showing, plus `format=`, `columns=` and the window
//! parameters the dialog sets, so the file is the table the reader saw.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, OriginalUri, Path, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::format::Format;
use super::model::{Column, DataSet, ExportContext, Table};
use super::{registry, window};
use crate::error::{AdminError, AdminResult};
use crate::types::UserContext;

#[derive(Debug, Default, Deserialize)]
struct ExportQuery {
    format: Option<Format>,
    columns: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportPreview {
    rows: i64,
    columns: usize,
    cells: i64,
    capped: bool,
    cap: i64,
    from: Option<String>,
    to: Option<String>,
    clamped: bool,
}

struct Prepared<'a> {
    dataset: &'static dyn DataSet,
    columns: Vec<Column>,
    ctx: ExportContext<'a>,
}

fn prepare<'a>(
    id: &str,
    uri: &axum::http::Uri,
    pool: &'a PgPool,
    user: &'a UserContext,
) -> AdminResult<Prepared<'a>> {
    let dataset =
        registry::find(id).ok_or_else(|| AdminError::NotFound(format!("No export named {id}")))?;
    if !dataset.allows(user) {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    let mut ctx = ExportContext::new(pool, user, uri.clone());
    ctx.window = window::resolve(dataset.window(), &ctx.query::<window::WindowQuery>()?)?;
    ctx.limit = dataset.cap();
    let query: ExportQuery = ctx.query()?;
    let columns = select_columns(dataset.columns(), query.columns.as_deref());
    Ok(Prepared {
        dataset,
        columns,
        ctx,
    })
}

// Why: the selection is applied in the dataset's own column order, whatever
// order the dialog sent, so two exports of the same table always line up.
fn select_columns(all: &'static [Column], picked: Option<&str>) -> Vec<Column> {
    let picked: Vec<&str> = picked
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .collect();
    let chosen: Vec<Column> = if picked.is_empty() {
        all.iter().copied().filter(|c| c.default).collect()
    } else {
        all.iter()
            .copied()
            .filter(|c| picked.contains(&c.key))
            .collect()
    };
    if chosen.is_empty() {
        all.to_vec()
    } else {
        chosen
    }
}

fn project(all: &'static [Column], chosen: &[Column], table: Table) -> Table {
    let keep: Vec<usize> = all
        .iter()
        .enumerate()
        .filter(|(_, c)| chosen.iter().any(|k| k.key == c.key))
        .map(|(i, _)| i)
        .collect();
    let rows = table
        .rows
        .into_iter()
        .map(|row| keep.iter().filter_map(|&i| row.get(i).cloned()).collect())
        .collect();
    Table {
        rows,
        total: table.total,
    }
}

pub(crate) async fn export_file(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    file_response(&id, &uri, &pool, &user).await
}

// Why: the file half without the extractors, so the per-page CSV URLs that
// predate this surface (`export::legacy`) serve the same bytes.
pub(crate) async fn file_response(
    id: &str,
    uri: &axum::http::Uri,
    pool: &PgPool,
    user: &UserContext,
) -> AdminResult<Response> {
    let prepared = prepare(id, uri, pool, user)?;
    let format = prepared
        .ctx
        .query::<ExportQuery>()?
        .format
        .unwrap_or(Format::Csv);
    let table = prepared.dataset.load(&prepared.ctx).await?;
    let table = project(prepared.dataset.columns(), &prepared.columns, table);
    let body = format.write(&prepared.columns, &table);
    Ok(format.into_response(&filename(&prepared, format), body))
}

pub(crate) async fn export_preview(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Json<ExportPreview>> {
    let mut prepared = prepare(&id, &uri, &pool, &user)?;
    // Why: one row is enough — every loader reports the filter's total beside
    // whatever slice it returns, and a preview must not pay for the file. A
    // loader that cannot count without loading ignores the limit instead.
    prepared.ctx.limit = 1;
    let table = prepared.dataset.load(&prepared.ctx).await?;
    let cap = prepared.dataset.cap();
    let rows = table.total.min(cap);
    let columns = prepared.columns.len();
    Ok(Json(ExportPreview {
        rows,
        columns,
        cells: rows * i64::try_from(columns).unwrap_or(0),
        capped: table.total > cap,
        cap,
        from: prepared.ctx.window.map(|w| w.from.to_rfc3339()),
        to: prepared.ctx.window.map(|w| w.to.to_rfc3339()),
        clamped: prepared.ctx.window.is_some_and(|w| w.clamped),
    }))
}

// Why: named for the table and its window, so a folder of exports stays
// legible — `requests-20260910-20260917.csv` rather than a row of
// `export.csv`s.
fn filename(prepared: &Prepared<'_>, format: Format) -> String {
    let id = prepared.dataset.id();
    let ext = format.extension();
    prepared.ctx.window.map_or_else(
        || format!("{id}-{}.{ext}", chrono::Utc::now().format("%Y%m%dT%H%M%SZ")),
        |w| {
            format!(
                "{id}-{}-{}.{ext}",
                w.from.format("%Y%m%d"),
                w.to.format("%Y%m%d")
            )
        },
    )
}
