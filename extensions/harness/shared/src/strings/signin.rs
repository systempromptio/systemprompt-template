//! The English text of the sign-in gate.

use sp_protocol::view::{InstanceSummary, SignInFailure};

use crate::views::signin::flow::{Failure, FirstSync, Problem};

pub(crate) const STEP_CONNECT: &str = "Step 1 of 2";
pub(crate) const STEP_CHOOSE: &str = "Step 2 of 2";

pub(crate) const CONNECT_TITLE: &str = "Connect to your organisation";
pub(crate) const CONNECT_BODY: &str = "Sign in to the instance that governs your work. Its skills, agents and MCP servers then arrive on this device by themselves.";
pub(crate) const INSECURE_TITLE: &str = "No secure credential storage";
pub(crate) const INSECURE_BODY: &str = "The operating system's credential store is not available to this app, so a sign-in cannot be kept safe and none can start.";

pub(crate) const ADDRESS_LABEL: &str = "Instance address";
pub(crate) const ADDRESS_PLACEHOLDER: &str = "https://instance.example.com";
pub(crate) const ADDRESS_EMPTY: &str = "Enter the address of your instance.";
pub(crate) const ADDRESS_CHECKING: &str = "Looking for the instance…";

pub(crate) const BROWSER: &str = "Sign in with your browser";
pub(crate) const BROWSER_HINT: &str =
    "Your browser opens the instance's sign-in page. Approve there and this device is linked.";
pub(crate) const BROWSER_WAITING: &str = "Waiting for you to approve in your browser…";
pub(crate) const CANCEL: &str = "Cancel";
pub(crate) const ESCAPE_KEY: &str = "Esc";
pub(crate) const TOKEN_ONLY: &str = "This instance links devices with a personal access token. Create one on the instance, or ask it for a one-time code, and paste it below.";
pub(crate) const KEEP: &str = "Keep me signed in on this device";

pub(crate) const TOKEN_SHOW: &str = "Use a personal access token instead";
pub(crate) const TOKEN_HIDE: &str = "Hide the token field";
pub(crate) const TOKEN_LABEL: &str = "Personal access token or one-time code";
pub(crate) const TOKEN_PLACEHOLDER: &str = "sp-live-… or a 64-character code";
pub(crate) const TOKEN_CONNECT: &str = "Connect";
pub(crate) const CODE_PAGE: &str = "Get a one-time code";
pub(crate) const CODE_PAGE_OPENED: &str =
    "Approve the request in your browser, then paste the code it shows here.";

pub(crate) const CHOOSE_TITLE: &str = "Choose an instance";
pub(crate) const CHOOSE_BODY: &str = "This device works with one instance at a time. You can add or switch instances later in Settings.";
pub(crate) const NO_INSTANCES: &str = "No signed-in instance was found. Go back and sign in again.";
pub(crate) const COUNTS_LOADING: &str = "Reading what this instance offers…";
pub(crate) const FINGERPRINT_LABEL: &str = "Manifest signing key";
pub(crate) const FINGERPRINT_NONE: &str = "This instance did not offer a signing key.";
pub(crate) const FINGERPRINT_NOTE: &str = "For your information. Compare it with the fingerprint your administrator publishes; you can trust the key in Settings.";
pub(crate) const KEY_TRUSTED: &str = "Trusted";
pub(crate) const KEY_NOT_TRUSTED: &str = "Not trusted yet";
pub(crate) const BACK: &str = "Back";
pub(crate) const FINISH: &str = "Finish";

pub(crate) const FINISHING_TITLE: &str = "Finishing setup…";

pub(crate) fn eyebrow(product: &str, step: &str) -> String {
    format!("{product} · {step}")
}

pub(crate) fn found(label: &str, host: &str) -> String {
    if label == host {
        format!("Found an instance at {host}")
    } else {
        format!("Found {label} at {host}")
    }
}

pub(crate) const fn problem(problem: Problem) -> &'static str {
    match problem {
        Problem::Malformed => {
            "That is not an instance address. Use a web address such as https://instance.example.com."
        },
        Problem::Unreachable => {
            "The instance did not answer. Check the address and your connection."
        },
        Problem::Unsupported => "This address answers, but it is not a supported instance.",
    }
}

pub(crate) const fn failure(failure: Failure) -> &'static str {
    match failure {
        Failure::Denied => "The sign-in was declined in the browser.",
        Failure::Expired => "The sign-in took too long and expired. Try again.",
        Failure::MissingToken => "Paste a personal access token or a one-time code first.",
        Failure::Activation => "This instance could not be made active. Try again.",
        Failure::SignIn(kind) => sign_in_failure(kind),
    }
}

const fn sign_in_failure(kind: SignInFailure) -> &'static str {
    match kind {
        SignInFailure::Unreachable => {
            "The instance could not be reached. Check your connection and try again."
        },
        SignInFailure::Refused => {
            "The instance refused the sign-in. Check the token or code and try again."
        },
        SignInFailure::RateLimited => "Too many attempts. Wait a moment, then try again.",
        SignInFailure::Incompatible => "This instance is not supported by this app.",
        SignInFailure::Insecure => {
            "There is no secure storage for credentials on this device, so the sign-in cannot finish."
        },
        SignInFailure::Failed => "The sign-in did not complete. Try again.",
    }
}

fn count(value: u32, one: &str, many: &str) -> String {
    format!("{value} {}", if value == 1 { one } else { many })
}

pub(crate) fn skills(value: u32) -> String {
    count(value, "skill", "skills")
}

pub fn counts(summary: &InstanceSummary) -> String {
    format!(
        "{} · {} · {}",
        skills(summary.skills),
        count(summary.agents, "agent", "agents"),
        count(summary.mcp, "MCP server", "MCP servers")
    )
}

pub(crate) fn role(role: &str) -> String {
    format!("Your role: {role}")
}

pub(crate) fn instance_detail(host: &str, user: Option<&str>) -> String {
    user.map_or_else(|| host.to_owned(), |user| format!("{host} · {user}"))
}

pub(crate) fn finishing(label: &str) -> String {
    format!("Fetching the first sync from {label}.")
}

pub(crate) fn first_sync(result: FirstSync) -> String {
    match result {
        FirstSync::Synced { skills: synced } => {
            format!("Synced {}. Opening your workspace…", skills(synced))
        },
        FirstSync::Failed => {
            "The first sync did not complete; it continues in the background.".to_owned()
        },
        FirstSync::TimedOut => {
            "The first sync is taking a while; it continues in the background.".to_owned()
        },
        FirstSync::Unavailable => "Opening your workspace…".to_owned(),
    }
}
