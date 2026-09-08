//! `/admin/reports/customer.csv` — the sendable month-end usage export.
//!
//! The *page* this module once served is gone: the Cost tab of
//! `/admin/analytics` answers the same question against any window rather than
//! only a calendar month. The month-scoped CSV stays because the finance
//! hand-off fetches it on a schedule, and moving that URL would break a
//! process outside this repo.
//!
//! It selects no cost column anywhere. The report leaves the platform team, so
//! "no internal figure leaks" is a property of the SQL rather than a
//! discipline a renderer has to keep.

pub(crate) mod csv;
