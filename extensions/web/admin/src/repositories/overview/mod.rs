//! The console landing page's data: one health-and-money read of the whole
//! instance.
//!
//! Every function here is instance-wide and range-bound. The overview answers
//! "is anything on fire" before a reader has chosen a page, so it deliberately
//! does not take a container scope — the AI-activity pages own that question —
//! and it never lists more than a handful of rows per widget.
//!
//! Each widget is its own `Result`. A page that blanks because one aggregate
//! failed is worse than a page that says which number it could not read, so
//! nothing here is folded into a single fallible load.

pub mod kpis;
pub mod liveness;
pub mod queues;
pub mod scopes;
pub mod series;
