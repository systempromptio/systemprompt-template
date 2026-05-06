//! Legacy alias — the old "decisions" page was unified into the Audit Trail.
//!
//! `governance_decisions_page` now delegates to [`super::governance_audit_page`]
//! so the previous `/admin/governance/decisions` route keeps working. New
//! call-sites should depend on `governance_audit_page` directly.

pub use super::ssr_governance_audit::governance_audit_page as governance_decisions_page;
