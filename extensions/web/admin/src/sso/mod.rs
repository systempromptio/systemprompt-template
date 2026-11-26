//! The browser sign-in provider the web member contributes to core's
//! `SsoSlot`: AD FS ([`adfs`]), with the directory side effects of a sign-in
//! ([`events`]). Core's SSO driver serves it under `/api/v1/core/sso/adfs`
//! and mints every session.

pub mod adfs;
pub mod events;
