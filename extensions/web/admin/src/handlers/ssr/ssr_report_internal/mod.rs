//! `/admin/reports/internal.csv` — the month-end provider cost export.
//!
//! The *page* this module once served is gone: the Cost tab of
//! `/admin/analytics` shows provider cost over any window, and exports it
//! through `/admin/analytics/cost.csv`. This month-scoped CSV stays because
//! the finance hand-off fetches it on a schedule, and moving that URL would
//! break a process outside this repo.

pub(crate) mod csv;
