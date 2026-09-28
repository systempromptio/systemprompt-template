//! One export surface for every table the console renders.
//!
//! A page declares a [`DataSet`](model::DataSet) — its columns and the
//! repository read behind them — and registers it in [`registry`]. The
//! handler serves any dataset in any [`Format`](format::Format) with any
//! column selection over any window the dataset's contract allows, and the
//! preview endpoint tells the dialog how many rows and cells a download will
//! hold before it is asked for. `datasets/` holds one file per table.

pub(crate) mod datasets;
pub(crate) mod format;
pub(crate) mod handler;
pub(crate) mod legacy;
pub(crate) mod model;
pub(crate) mod registry;
pub(crate) mod view;
pub(crate) mod window;

pub(crate) use view::ExportView;
