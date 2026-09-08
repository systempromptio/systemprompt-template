//! Month-scoped usage aggregates behind the two end-of-month reports.
//!
//! Split by audience, and deliberately so. [`internal`] reads provider cost —
//! the operator's supplier bill. [`customer`] reads what a project consumed,
//! and selects no cost column anywhere: the report is sent outside the
//! platform team, so "no internal figures leak" is a property of the SQL
//! rather than a discipline the template has to keep.
//!
//! Every function takes an explicit `from`/`to` pair. The rest of the codebase
//! aggregates over rolling windows anchored at `now()`; a billing report
//! cannot, because the number it prints has to still be the same number next
//! week.

pub mod customer;
pub mod internal;
