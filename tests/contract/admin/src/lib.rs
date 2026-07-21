//! HTTP contract coverage for the admin plane.
//!
//! Every route in `extensions/web/admin/src/routes/{admin,ssr,ssr_bridge}.rs`
//! is driven end-to-end through the *real* router — assembled exactly as
//! `extensions/web/src/extension_impl.rs` assembles it — under three
//! principals: anonymous, authenticated non-admin, and admin.
//!
//! The suite asserts two properties that matter more than any individual
//! status code:
//!
//! - **No route 500s on a well-formed request.** A handler that swallows a
//!   database failure into a rendered page, or that panics on an unknown id,
//!   fails here.
//! - **The table is exhaustive.** [`route_source`] re-reads the route modules
//!   and asserts every path they mount has a contract entry, so a new route
//!   cannot land uncovered.
//!
//! On top of those it pins the observed status of every (route, principal)
//! pair against a checked-in baseline, so a refactor that shifts a status
//! shows up as a reviewable diff rather than a silent behaviour change.
//!
//! Everything runs against a throwaway database created on the server named by
//! `DATABASE_URL`; the suite self-skips when no server is configured.

#[cfg(test)]
mod app;
#[cfg(test)]
mod baseline;
#[cfg(test)]
mod globals;
#[cfg(test)]
mod principal;
#[cfg(test)]
mod route_source;
#[cfg(test)]
mod status_contract;
#[cfg(test)]
mod tempdb;
