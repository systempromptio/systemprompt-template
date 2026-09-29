//! Server-rendered governance pages: the decisions, the approvals queue, the
//! secrets trail and the quota ceilings.
//!
//! Split from `ssr.rs` on size alone; the sidebar group and the handlers are
//! unchanged. The two `.csv` URLs are the pre-export-dialog links, kept
//! mounted so a bookmarked download still answers.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

// Why: sidebar group 4. These read a posture rather than listing an entity.
// The three are one group because they are the three things a policy can do to
// a call — decide it, hold it for a person, or record a credential it touched —
// and an operator tuning one reads the other two.
pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/governance", get(handlers::ssr::governance_page))
        .route(
            "/governance/warnings.csv",
            get(crate::export::legacy::governance_csv),
        )
        .route(
            "/governance/decisions/{decision_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route("/governance/approvals", get(handlers::ssr::approvals_page))
        .route(
            "/governance/secrets",
            get(handlers::ssr::secrets_audit_page),
        )
        .route(
            "/governance/secrets.csv",
            get(crate::export::legacy::secrets_csv),
        )
        // Why: usage against ceiling per subject — the read side of the quota
        // windows the gateway policy editor writes.
        .route(
            "/governance/quotas",
            get(handlers::ssr::governance_quotas_page),
        )
}
