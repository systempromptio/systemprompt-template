//! Email + passkey self-registration.
//!
//! `POST /admin/auth/passkey/register` is the domain-gated front door for
//! creating an account without Salesforce: it validates the email against the
//! same `allowed_email_domains` list that gates SSO, provisions the user (org
//! membership and seat limit included), and returns a short-lived setup token.
//! The browser then enrols the passkey through core's public
//! `/api/v1/core/oauth/webauthn/link/{start,finish}` ceremony — the same flow
//! CLI-created users use — and signs in with the fresh credential.
//!
//! Core's own open `/webauthn/register/*` endpoints are disabled via
//! `security.allow_registration: false`; this endpoint is the only
//! self-registration door.
//!
//! That door is **shut by default** (`allow_self_registration`, in the same
//! `salesforce.yaml`). Closed enrolment is the enterprise posture: an
//! allow-listed domain establishes that an address *could* belong to someone
//! who should have access, never that anyone approved them. With the switch
//! off this endpoint refuses every caller, and an invite — which carries its
//! own authorization and so bypasses the domain list — is the only way in.
//! Turning it on restores the domain allowlist as the whole gate.

mod register;

pub(crate) use register::passkey_register;
