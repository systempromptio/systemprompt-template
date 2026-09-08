//! Strip the gateway's transcript framing out of a stored message body.
//!
//! A `/v1/messages` turn is persisted as a debug transcript rather than a bare
//! message: the user's prompt is introduced by a `=== USER PROMPT ===` header
//! and the assistant's raw half follows a `=== ASSISTANT ANSWER ===` header.
//! That framing is protocol noise to the person who wrote the prompt, so every
//! user-facing surface cuts it. The SQL preview CTEs in `conversation_rows`
//! and `conversations::unified` do the same in Postgres; this is the Rust
//! half, kept pure so the rule is pinned by a unit test.

const USER_MARKER: &str = "=== USER PROMPT ===";
const ASSISTANT_MARKER: &str = "=== ASSISTANT ANSWER ===";

// Why: everything before the assistant marker, with the user marker removed
// Why: and surrounding whitespace trimmed. A body carrying neither marker is
// Why: returned trimmed but otherwise untouched.
#[must_use]
pub fn strip_gateway_markers(input: &str) -> String {
    let head = input.split(ASSISTANT_MARKER).next().unwrap_or(input);
    head.replace(USER_MARKER, "").trim().to_owned()
}
