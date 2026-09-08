//! Reads and revocations for the device fleet: bridge sessions, personal
//! access tokens, device certificates, and enrolment links still unclaimed.
//!
//! The four are one page because they are one question — "what is holding a
//! credential to this platform, and is it still alive?" — asked of four
//! tables. They are separate modules because nothing joins them: a bridge
//! session is a heartbeat, a token is a secret, a certificate is a key, and a
//! link is a promise. Only the person they belong to is common, so every row
//! type carries a `UserId` and the page renders it as the same link.
//!
//! Liveness is one number, `STALE_AFTER_DAYS`, applied to a heartbeat. It
//! lives here rather than in the handler so the count on the KPI tile and the
//! badge on the row can never disagree about what stale means.

pub mod certs;
pub mod links;
pub mod pats;
pub mod sessions;
pub mod stats;

// Why: a bridge that has not called home in a week is not a bridge anyone is
// using; it is a credential still valid on a machine nobody is watching. Seven
// days is a working week, so a laptop closed on Friday is not reported as
// abandoned on Monday.
pub const STALE_AFTER_DAYS: i64 = 7;
