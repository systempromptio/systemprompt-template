//! Display formatting shared by the admin entity pages.
//!
//! The pure value formatters live in `systemprompt_web_shared::format` (so
//! the test workspace can exercise them); this module re-exports them for
//! the ssr pages and keeps the admin-specific helpers.
//!
//! Ids, costs, token counts and durations are rendered in a handful of places
//! across the entity list and detail pages; the rules live here so a cost reads
//! the same on the sessions list as it does on the session it links to.

use serde::Serialize;

pub(crate) use systemprompt_web_shared::format::{
    format_cost, format_duration_ms, short_id, short_num,
};

pub(crate) fn format_token_total(total: i64) -> String {
    if total <= 0 {
        return "—".to_owned();
    }
    short_num(total)
}

// Why: A timestamp in the viewer's local zone, to the second.
pub(crate) fn local_time(t: chrono::DateTime<chrono::Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

// Why: How long a first-to-last window lasted, or an em dash when either end is
// missing — a hook session can arrive with no timestamp at all.
pub(crate) fn format_span(
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (start, end) {
        (Some(a), Some(b)) => format_duration_ms((b - a).num_milliseconds().max(0)),
        _ => "\u{2014}".to_owned(),
    }
}

// Why: "3d ago" answers "is this still in use" at a glance; the exact stamp
// stays on the cell's `title`. Months are thirty days, which is the roster's
// idle threshold, so "1mo ago" and the idle-30d chip agree.
pub(crate) fn relative_time(t: chrono::DateTime<chrono::Utc>) -> String {
    let delta = chrono::Utc::now().timestamp() - t.timestamp();
    match delta {
        d if d < 60 => "just now".to_owned(),
        d if d < 3_600 => format!("{}m ago", d / 60),
        d if d < 86_400 => format!("{}h ago", d / 3_600),
        d if d < 2_592_000 => format!("{}d ago", d / 86_400),
        d => format!("{}mo ago", d / 2_592_000),
    }
}

// Why: `ai_requests.client_kind` is a closed set the core enum owns; a value
// this build does not know (a newer core) is shown as it was stored rather
// than mislabelled, so the chip never lies.
pub(crate) fn client_label(client_kind: &str) -> String {
    systemprompt::models::origin::ClientKind::parse(client_kind)
        .map_or_else(|_| client_kind.to_owned(), |kind| kind.label().to_owned())
}

// Why: one client badge — the label names the client, the tone and title say
// how strongly `ai_requests.client_attestation` evidences it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ClientChipView {
    pub label: String,
    pub tone: &'static str,
    pub title: String,
}

// Why: the tier decides the tone, never the client — a host-token Claude Code
// and a user-agent Claude Code read as the same client at different
// confidence, which is exactly the distinction the column records.
pub(crate) fn client_chip(client_kind: &str, attestation: &str) -> ClientChipView {
    use systemprompt::models::origin::ClientAttestation;
    let tone = match ClientAttestation::parse(attestation) {
        Ok(ClientAttestation::HostToken) => "accent",
        Ok(ClientAttestation::Declared) => "info",
        Ok(_) | Err(_) => "muted",
    };
    ClientChipView {
        label: client_label(client_kind),
        tone,
        title: attestation_title(attestation),
    }
}

pub(crate) fn attestation_title(attestation: &str) -> String {
    systemprompt::models::origin::ClientAttestation::parse(attestation).map_or_else(
        |_| format!("Attestation: {attestation}"),
        |tier| tier.label().to_owned(),
    )
}
