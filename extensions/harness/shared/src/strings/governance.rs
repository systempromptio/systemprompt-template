//! The English text of the governance pill and the Governance settings page.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_protocol::view::{SignInFailure, SignInOutcome, SyncErrorKind};

use crate::views::governance::heartbeat::{HeartbeatLine, Unsupported};
use crate::views::governance::model::{Ago, Fix, KeyTrust, SyncLine};
use crate::views::governance::pill_model::PillStatus;

pub(crate) const PAGE_SUBTITLE: &str =
    "The instances this device knows. The active one supplies models, skills and rules.";
pub(crate) const ADD_INSTANCE: &str = "Add instance";
pub(crate) const NO_INSTANCES_TITLE: &str = "No instances yet";
pub(crate) const NO_INSTANCES_BODY: &str = "Add your organisation's instance to sign in.";
pub(crate) const INSECURE_STORAGE: &str = "Windows secure storage is not available, so no sign-in can start. Sign in to Windows normally and restart the app.";

pub(crate) const BADGE_ACTIVE: &str = "Active";
pub(crate) const BADGE_SIGNED_OUT: &str = "Signed out";
pub(crate) const BADGE_NEW_KEY: &str = "New key";
pub(crate) const BADGE_KEY_CHANGED: &str = "Key changed";

pub(crate) const FIELD_URL: &str = "Address";
pub(crate) const FIELD_USER: &str = "User";
pub(crate) const FIELD_ORGANISATION: &str = "Organisation";
pub(crate) const FIELD_KEY: &str = "Manifest key";
pub(crate) const NOT_KNOWN: &str = "Not known";
pub(crate) const NO_KEY: &str = "No key pinned";

pub(crate) const MAKE_ACTIVE: &str = "Make active";
pub(crate) const SYNC_NOW: &str = "Sync now";
pub(crate) const SIGN_OUT: &str = "Sign out";
pub(crate) const SIGN_IN_AGAIN: &str = "Sign in again";
pub(crate) const REMOVE: &str = "Remove";
pub(crate) const CANCEL: &str = "Cancel";

pub(crate) const SIGN_OUT_TITLE: &str = "Sign out of this instance?";
pub(crate) const SIGN_OUT_BODY: &str = "The device forgets its token for this instance. Its models, skills and rules stop until you sign in again.";
pub(crate) const REMOVE_TITLE: &str = "Remove this instance?";
pub(crate) const REMOVE_BODY: &str =
    "The device forgets this instance, its token and its trusted key. You can add it again later.";

pub(crate) const KEY_NEW_BODY: &str = "This instance signs its manifest with a key this device has not seen. Compare the fingerprint with the one your administrator published before you trust it.";
pub(crate) const KEY_CHANGED_BODY: &str = "This instance now signs with a different key. Sync is stopped until you trust the new key. If you did not expect a change, ask your administrator first.";
pub(crate) const KEY_PINNED: &str = "Pinned";
pub(crate) const KEY_OFFERED: &str = "Offered";
pub(crate) const TRUST_THIS_KEY: &str = "Trust this key";
pub(crate) const TRUST_NEW_KEY: &str = "Trust new key";
pub(crate) const KEY_TRUSTED: &str = "The key is trusted. Sync resumes.";

pub(crate) const HOLDS_TITLE: &str = "What it holds";
pub(crate) const WARNINGS_TITLE: &str = "Left out by this device";

pub(crate) const SYNCED_TITLE: &str = "Synced content";
pub(crate) const SYNCED_EMPTY: &str =
    "Nothing is applied yet. Content appears after the first sync.";
pub(crate) const SKILLS: &str = "Skills";
pub(crate) const MCP_SERVERS: &str = "MCP servers";
pub(crate) const RULES: &str = "Rules";
pub(crate) const NONE_SYNCED: &str = "None";
pub(crate) const MCP_CONNECTED: &str = "Connected";
pub(crate) const MCP_CONNECTING: &str = "Connecting";
pub(crate) const MCP_FAILED: &str = "Failed";
pub(crate) const MCP_DISABLED: &str = "Disabled";
pub(crate) const MCP_NEEDS_AUTH: &str = "Needs authorization";
pub(crate) const MCP_NOT_STARTED: &str = "Not started";

pub(crate) const VERDICT_TITLE: &str = "Tool verdicts";
pub(crate) const VERDICT_LABEL: &str = "Ask the instance before each tool runs";
pub(crate) const VERDICT_BODY: &str = "The instance can deny a tool and tell the agent why. When it does not answer within 3 seconds, the tool runs.";
pub(crate) const VERDICT_NOT_GOVERNED: &str =
    "The active instance does not ask for verdicts, so this has no effect yet.";
pub(crate) const VERDICT_UNAVAILABLE: &str = "Verdicts are not available in this session.";

pub(crate) const HEARTBEAT_TITLE: &str = "Heartbeat";
pub(crate) const HEARTBEAT_PAUSED: &str = "Paused until an instance is signed in.";
pub(crate) const HEARTBEAT_ONLINE: &str = "Online. The instance sees this device every 30 seconds.";
pub(crate) const HEARTBEAT_FAILING: &str = "Not reaching the instance. It keeps trying.";
pub(crate) const HEARTBEAT_UNAVAILABLE: &str = "The heartbeat is not running in this session.";

pub(crate) const PILL_SIGNED_OUT: &str = "Not signed in";
pub(crate) const PILL_OPEN_SETTINGS: &str = "Open Governance settings";
pub(crate) const PILL_SWITCH_TO: &str = "Switch instance";

pub(crate) const CONNECT_TITLE: &str = "Add instance";
pub(crate) const CONNECT_URL_LABEL: &str = "Instance address";
pub(crate) const CONNECT_URL_PLACEHOLDER: &str = "https://instance.example.com";
pub(crate) const CONNECT_TOKEN_LABEL: &str = "Personal access token";
pub(crate) const CONNECT_TOKEN_HINT: &str =
    "Only if your administrator gave you one. Leave empty to use the browser.";
pub(crate) const CONNECT_KEEP: &str = "Keep me signed in on this device";
pub(crate) const CONNECT_BROWSER: &str = "Sign in with browser";
pub(crate) const CONNECT_TOKEN: &str = "Sign in with token";
pub(crate) const CONNECT_WAITING: &str =
    "Finish signing in in your browser. This window updates by itself.";
pub(crate) const CONNECT_WORKING: &str = "Signing in…";
pub(crate) const CONNECT_URL_MISSING: &str = "Enter the instance's address.";
pub(crate) const CONNECT_DONE: &str = "Signed in. Make the instance active to use it.";

pub(crate) const ACTIVATE_FAILED: &str = "The instance could not be made active.";
pub(crate) const SIGN_OUT_FAILED: &str = "The instance could not be signed out.";
pub(crate) const REMOVE_FAILED: &str = "The instance could not be removed.";
pub(crate) const TRUST_FAILED: &str =
    "The key could not be trusted. It may have changed again; check the fingerprint.";
pub(crate) const SYNC_UNAVAILABLE: &str = "Sync is not running in this session.";

pub(crate) fn sync_line(line: &SyncLine) -> String {
    match line {
        SyncLine::SignedOut => "Signed out".to_owned(),
        SyncLine::Inactive => "Not active".to_owned(),
        SyncLine::NoSync => "Sync is not running".to_owned(),
        SyncLine::Waiting => "Waiting to sync".to_owned(),
        SyncLine::Syncing => "Syncing…".to_owned(),
        SyncLine::Synced { ago } => format!("Synced {}", ago_text(*ago)),
        SyncLine::Failed { kind, ago } => format!("{} · {}", sync_error(*kind), ago_text(*ago)),
    }
}

pub(crate) fn ago_text(ago: Ago) -> String {
    match ago {
        Ago::JustNow => "just now".to_owned(),
        Ago::Minutes(1) => "1 minute ago".to_owned(),
        Ago::Minutes(count) => format!("{count} minutes ago"),
        Ago::Hours(1) => "1 hour ago".to_owned(),
        Ago::Hours(count) => format!("{count} hours ago"),
        Ago::Days(1) => "yesterday".to_owned(),
        Ago::Days(count) => format!("{count} days ago"),
    }
}

pub(crate) const fn sync_error(kind: SyncErrorKind) -> &'static str {
    match kind {
        SyncErrorKind::Unreachable => "Instance unreachable",
        SyncErrorKind::RateLimited => "Instance is busy",
        SyncErrorKind::Refused => "Sign-in refused",
        SyncErrorKind::KeyChanged => "Signing key changed",
        SyncErrorKind::Invalid => "Manifest rejected",
        SyncErrorKind::Insecure => "Secure storage locked",
        SyncErrorKind::Service => "Could not apply the content",
    }
}

pub(crate) const fn sync_error_hint(kind: SyncErrorKind) -> &'static str {
    match kind {
        SyncErrorKind::Unreachable => "Check your connection or VPN. Sync retries by itself.",
        SyncErrorKind::RateLimited => "The instance asked to slow down. Sync retries shortly.",
        SyncErrorKind::Refused => {
            "The instance no longer accepts this device's token. Sign in again."
        },
        SyncErrorKind::KeyChanged => "Review the new key below and trust it to resume sync.",
        SyncErrorKind::Invalid => {
            "The manifest did not verify or was not newer. Ask your administrator to publish again."
        },
        SyncErrorKind::Insecure => {
            "Windows could not open the stored token. Unlock your session, then sync again."
        },
        SyncErrorKind::Service => {
            "This device could not write the synced files. Sync again to retry."
        },
    }
}

pub(crate) const fn fix_label(fix: Fix) -> &'static str {
    match fix {
        Fix::SyncNow => SYNC_NOW,
        Fix::SignIn => SIGN_IN_AGAIN,
        Fix::ReviewKey => "Review key",
        Fix::Wait => "Retry now",
        Fix::AskAdministrator => PILL_OPEN_SETTINGS,
    }
}

pub(crate) const fn key_badge(key: &KeyTrust) -> &'static str {
    if key.pinned.is_some() {
        BADGE_KEY_CHANGED
    } else {
        BADGE_NEW_KEY
    }
}

pub(crate) const fn key_body(key: &KeyTrust) -> &'static str {
    if key.pinned.is_some() {
        KEY_CHANGED_BODY
    } else {
        KEY_NEW_BODY
    }
}

pub(crate) const fn trust_label(key: &KeyTrust) -> &'static str {
    if key.pinned.is_some() {
        TRUST_NEW_KEY
    } else {
        TRUST_THIS_KEY
    }
}

pub(crate) fn heartbeat(line: &HeartbeatLine) -> String {
    match line {
        HeartbeatLine::Unavailable => HEARTBEAT_UNAVAILABLE.to_owned(),
        HeartbeatLine::Paused => HEARTBEAT_PAUSED.to_owned(),
        HeartbeatLine::Online => HEARTBEAT_ONLINE.to_owned(),
        HeartbeatLine::Failing => HEARTBEAT_FAILING.to_owned(),
        HeartbeatLine::Unsupported { minimum, reason } => match reason {
            Unsupported::TooOld => format!("This version is too old for the instance. It needs {minimum} or newer; update the app."),
            Unsupported::TooNew => "This version is newer than the instance allows. Ask your administrator to update the instance.".to_owned(),
            Unsupported::Refused => format!("The instance does not accept this version (minimum {minimum}). Update the app."),
        },
    }
}

pub(crate) fn counts(skills: u32, agents: u32, mcp: u32, rules: u32) -> String {
    format!(
        "{} · {} · {} · {}",
        plural(skills, "skill", "skills"),
        plural(agents, "agent", "agents"),
        plural(mcp, "MCP server", "MCP servers"),
        plural(rules, "rule", "rules"),
    )
}

pub(crate) fn rules_line(rules: u32) -> String {
    match rules {
        0 => {
            "No rules. Rules reach every session through the managed part of AGENTS.md.".to_owned()
        },
        count => format!(
            "{} reach every session through the managed part of AGENTS.md.",
            plural(count, "rule", "rules")
        ),
    }
}

pub(crate) fn synced_from(name: &str) -> String {
    format!("{SYNCED_TITLE} from {name}")
}

pub(crate) fn switched(name: &str) -> String {
    format!("{name} is now active.")
}

pub(crate) fn mcp_connected(tools: usize) -> String {
    format!(
        "{MCP_CONNECTED} · {}",
        plural(u32::try_from(tools).unwrap_or(u32::MAX), "tool", "tools")
    )
}

pub(crate) const fn sign_in_outcome(outcome: SignInOutcome) -> &'static str {
    match outcome {
        SignInOutcome::SignedIn => CONNECT_DONE,
        SignInOutcome::Cancelled => "Sign-in was cancelled.",
        SignInOutcome::Denied => "Sign-in was denied on the instance's page.",
        SignInOutcome::Expired => "Sign-in took too long. Try again.",
        SignInOutcome::Failed(SignInFailure::Unreachable) => {
            "The instance could not be reached. Check the address."
        },
        SignInOutcome::Failed(SignInFailure::Refused) => "The instance refused the token or code.",
        SignInOutcome::Failed(SignInFailure::RateLimited) => {
            "Too many attempts. Wait a moment and try again."
        },
        SignInOutcome::Failed(SignInFailure::Incompatible) => {
            "That address is not a supported instance."
        },
        SignInOutcome::Failed(SignInFailure::Insecure) => INSECURE_STORAGE,
        SignInOutcome::Failed(SignInFailure::Failed) => "Sign-in failed. Try again.",
    }
}

pub(crate) fn pill_status(status: &PillStatus) -> String {
    match status {
        PillStatus::SignedOut => PILL_SIGNED_OUT.to_owned(),
        PillStatus::KeyPending { rotated: true } => BADGE_KEY_CHANGED.to_owned(),
        PillStatus::KeyPending { rotated: false } => BADGE_NEW_KEY.to_owned(),
        PillStatus::Failed(kind) => sync_error(*kind).to_owned(),
        PillStatus::Syncing => sync_line(&SyncLine::Syncing),
        PillStatus::Synced(ago) => sync_line(&SyncLine::Synced { ago: *ago }),
        PillStatus::Waiting => sync_line(&SyncLine::Waiting),
        PillStatus::NoSync => sync_line(&SyncLine::NoSync),
    }
}

pub(crate) fn name_with_host(label: &str, host: &str) -> String {
    format!("{label} ({host})")
}

fn plural(count: u32, one: &str, many: &str) -> String {
    if count == 1 {
        format!("1 {one}")
    } else {
        format!("{count} {many}")
    }
}
