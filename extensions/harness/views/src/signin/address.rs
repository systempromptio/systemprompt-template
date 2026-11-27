//! Pure helpers of the sign-in gate: the address the probe and the sign-ins
//! use, what the token field holds, the instances step 2 lists and the sync
//! status that ends the finishing step.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_accounts::AccountsSnapshot;
use sp_protocol::ids::AccountId;
use sp_protocol::view::{AccountStatus, AccountView, InstanceSummary, KeyFingerprint, SyncStatus};
use url::Url;

use super::flow::FirstSync;

const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];
const CODE_LENGTH: usize = 64;
const FINGERPRINT_GROUP: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Credential {
    Pat,
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceChoice {
    pub account: AccountId,
    pub label: String,
    pub host: String,
    pub user: Option<String>,
    pub fingerprint: Option<KeyFingerprint>,
    pub trusted: bool,
    pub summary: Option<InstanceSummary>,
}

pub fn instance_target(input: &str) -> Option<String> {
    let text = input.trim();
    if text.is_empty() || text.chars().any(char::is_whitespace) {
        return None;
    }
    if text.contains("://") {
        let url = Url::parse(text).ok()?;
        let web = matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some_and(|host| !host.is_empty());
        return web.then(|| text.to_owned());
    }
    let assumed = Url::parse(&format!("https://{text}")).ok()?;
    let host = assumed.host_str()?;
    if LOOPBACK_HOSTS.contains(&host) {
        Some(format!("http://{text}"))
    } else {
        Some(format!("https://{text}"))
    }
}

pub fn credential_kind(token: &str) -> Option<Credential> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    let code = token.len() == CODE_LENGTH
        && token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    Some(if code {
        Credential::Code
    } else {
        Credential::Pat
    })
}

pub(crate) fn host_of(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| {
            let host = parsed.host_str()?.to_owned();
            Some(match parsed.port() {
                Some(port) => format!("{host}:{port}"),
                None => host,
            })
        })
        .unwrap_or_else(|| url.to_owned())
}

pub fn opening_address(snapshot: &AccountsSnapshot, default: &str) -> String {
    snapshot
        .active_account()
        .map_or_else(|| default.to_owned(), |account| account.url.clone())
}

pub fn choices(snapshot: &AccountsSnapshot) -> Vec<InstanceChoice> {
    let rank = |id: &AccountId| -> u8 {
        if snapshot.last_signed_in.as_ref() == Some(id) {
            0
        } else if snapshot.active.as_ref() == Some(id) {
            1
        } else {
            2
        }
    };
    let mut list: Vec<InstanceChoice> = snapshot
        .accounts
        .iter()
        .filter(|account| account.status == AccountStatus::SignedIn)
        .map(choice)
        .collect();
    list.sort_by_key(|choice| rank(&choice.account));
    list
}

fn choice(account: &AccountView) -> InstanceChoice {
    InstanceChoice {
        account: account.id.clone(),
        label: account.label.clone(),
        host: host_of(&account.url),
        user: account.user.clone(),
        fingerprint: account.fingerprint.clone(),
        trusted: account.trusted,
        summary: None,
    }
}

pub fn settled(status: &SyncStatus, account: &AccountId) -> Option<FirstSync> {
    match status {
        SyncStatus::Synced {
            account: synced,
            counts,
            ..
        } if synced == account => Some(FirstSync::Synced {
            skills: counts.skills,
        }),
        SyncStatus::Error {
            account: failed, ..
        } if failed == account => Some(FirstSync::Failed),
        _ => None,
    }
}

pub fn grouped_fingerprint(fingerprint: &str) -> String {
    let (prefix, digest) = fingerprint.split_once(':').unwrap_or(("", fingerprint));
    let groups: Vec<String> = digest
        .chars()
        .collect::<Vec<char>>()
        .chunks(FINGERPRINT_GROUP)
        .map(|group| group.iter().collect())
        .collect();
    let joined = groups.join(" ");
    if prefix.is_empty() {
        joined
    } else {
        format!("{prefix}: {joined}")
    }
}
