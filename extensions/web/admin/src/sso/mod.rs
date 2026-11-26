//! Browser sign-in providers the web member contributes to core's `SsoSlot`:
//! AD FS ([`adfs`]) and the development login link ([`dev`]), with the
//! directory side effects of a sign-in ([`events`]). Core's SSO driver serves
//! each under `/api/v1/core/sso/{id}` and mints every session.

pub mod adfs;
pub mod dev;
pub mod events;
