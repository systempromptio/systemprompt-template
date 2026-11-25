//! The connector rows of the access overview: whether each connection needs
//! the user's attention, and how it reads on the page.

use super::access_overview::ConnectionView;
use crate::services::connector_accounts;

pub(super) fn needs_attention(connection: &connector_accounts::Connection) -> bool {
    !connection.configured || connection.status != "connected" || connection.verified_at.is_none()
}

pub(super) fn connection_view(c: &connector_accounts::Connection) -> ConnectionView {
    let (status, tone, next_step) = if c.configured {
        match c.status.as_str() {
            "connected" if c.verified_at.is_some() => (
                "Connected",
                "ok",
                "Connection verified; individual tool requests can still fail.",
            ),
            "temporarily_unavailable" => (
                "Temporarily unavailable",
                "warn",
                "Retry the connection when the provider is available.",
            ),
            "connected" => (
                "Verification required",
                "warn",
                "The user must test this connection in the bridge app.",
            ),
            "not_connected" | "expired" | "revoked" | "reauth_required" | "reconnect_required" => (
                "Sign-in required",
                "warn",
                "The user must sign in to this connector in the bridge app.",
            ),
            _ => (
                "Verification required",
                "warn",
                "The user must review and test this connection in the bridge app.",
            ),
        }
    } else {
        (
            "Not configured",
            "muted",
            "An administrator must configure this connector.",
        )
    };
    ConnectionView {
        name: match c.provider.as_str() {
            "atlassian" => "Atlassian",
            "github" => "GitHub",
            other => other,
        }
        .to_owned(),
        permission: if c.entitled { "Allowed" } else { "Not allowed" },
        status,
        tone,
        next_step: if c.entitled {
            next_step
        } else {
            "Review this person's workspace and connector permissions."
        },
        verified_at: c
            .verified_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(
                || "Not verified".to_owned(),
                |t| super::view::stamp(Some(t.with_timezone(&chrono::Utc))),
            ),
    }
}
