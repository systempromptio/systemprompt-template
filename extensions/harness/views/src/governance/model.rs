//! The Governance page as plain values derived from the accounts snapshot, the
//! sync status and the heartbeat.
//!
//! The page only renders them, and every status and action an instance offers
//! is tested without a window.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use chrono::{DateTime, Utc};
use harness_ui::Tone;
use sp_accounts::AccountsSnapshot;
use sp_protocol::ids::AccountId;
use sp_protocol::view::{
    AccountStatus, AccountView, KeyFingerprint, SyncCounts, SyncErrorKind, SyncStatus,
};

use crate::strings::governance as text;

const FINGERPRINT_HEAD: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ago {
    JustNow,
    Minutes(u32),
    Hours(u32),
    Days(u32),
}

pub fn ago(at: DateTime<Utc>, now: DateTime<Utc>) -> Ago {
    let seconds = (now - at).num_seconds().max(0);
    let whole = |value: i64| u32::try_from(value).unwrap_or(u32::MAX);
    match seconds {
        0..60 => Ago::JustNow,
        60..3_600 => Ago::Minutes(whole(seconds / 60)),
        3_600..86_400 => Ago::Hours(whole(seconds / 3_600)),
        _ => Ago::Days(whole(seconds / 86_400)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncLine {
    SignedOut,
    Inactive,
    NoSync,
    Waiting,
    Syncing,
    Synced { ago: Ago },
    Failed { kind: SyncErrorKind, ago: Ago },
}

impl SyncLine {
    pub(crate) const fn tone(&self) -> Tone {
        match self {
            Self::SignedOut => Tone::Warning,
            Self::Inactive | Self::NoSync | Self::Waiting => Tone::Neutral,
            Self::Syncing => Tone::Accent,
            Self::Synced { .. } => Tone::Success,
            Self::Failed { .. } => Tone::Danger,
        }
    }

    pub(crate) const fn live(&self) -> bool {
        matches!(self, Self::Syncing)
    }

    pub const fn failure(&self) -> Option<SyncErrorKind> {
        match self {
            Self::Failed { kind, .. } => Some(*kind),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyTrust {
    pub offered: KeyFingerprint,
    pub pinned: Option<KeyFingerprint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceAction {
    Trust,
    SyncNow,
    MakeActive,
    SignIn,
    SignOut,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fix {
    SyncNow,
    SignIn,
    ReviewKey,
    Wait,
    AskAdministrator,
}

pub const fn fix(kind: SyncErrorKind) -> Fix {
    match kind {
        SyncErrorKind::Unreachable | SyncErrorKind::RateLimited => Fix::Wait,
        SyncErrorKind::Refused => Fix::SignIn,
        SyncErrorKind::KeyChanged => Fix::ReviewKey,
        SyncErrorKind::Invalid => Fix::AskAdministrator,
        SyncErrorKind::Insecure | SyncErrorKind::Service => Fix::SyncNow,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastSynced {
    pub(crate) account: AccountId,
    pub(crate) at: DateTime<Utc>,
    pub(crate) counts: SyncCounts,
}

pub fn remember_synced(previous: Option<LastSynced>, status: &SyncStatus) -> Option<LastSynced> {
    match status {
        SyncStatus::Synced {
            account,
            at,
            counts,
        } => Some(LastSynced {
            account: account.clone(),
            at: *at,
            counts: counts.clone(),
        }),
        _ => previous,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceModel {
    pub(crate) id: AccountId,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) user: Option<String>,
    pub(crate) organisation: Option<String>,
    pub(crate) active: bool,
    pub(crate) signed_in: bool,
    pub sync: SyncLine,
    pub holds: Option<SyncCounts>,
    pub(crate) fingerprint: Option<String>,
    pub key: Option<KeyTrust>,
    pub actions: Vec<InstanceAction>,
}

#[derive(Debug, Clone, Copy)]
pub struct Inputs<'a> {
    pub accounts: &'a AccountsSnapshot,
    pub sync: Option<&'a SyncStatus>,
    pub last_synced: Option<&'a LastSynced>,
    pub now: DateTime<Utc>,
}

pub fn instances(inputs: Inputs<'_>) -> Vec<InstanceModel> {
    inputs
        .accounts
        .accounts
        .iter()
        .map(|account| instance(account, inputs))
        .collect()
}

fn instance(account: &AccountView, inputs: Inputs<'_>) -> InstanceModel {
    let signed_in = account.status == AccountStatus::SignedIn;
    let key = inputs
        .accounts
        .offers
        .iter()
        .find(|pending| pending.account == account.id)
        .map(|pending| KeyTrust {
            offered: pending.offer.offered.clone(),
            pinned: pending.offer.pinned.clone(),
        });
    let sync = sync_line(account, signed_in, inputs);
    InstanceModel {
        id: account.id.clone(),
        name: display_name(account, &inputs.accounts.accounts),
        url: account.url.clone(),
        user: account.user.clone().filter(|user| !user.trim().is_empty()),
        organisation: account.org.clone().filter(|org| !org.trim().is_empty()),
        active: account.active,
        signed_in,
        sync,
        holds: holds(account, inputs),
        fingerprint: account.pinned.as_ref().map(short_fingerprint),
        actions: actions(account, signed_in, key.is_some(), inputs.sync.is_some()),
        key,
    }
}

fn sync_line(account: &AccountView, signed_in: bool, inputs: Inputs<'_>) -> SyncLine {
    if !signed_in {
        return SyncLine::SignedOut;
    }
    if !account.active {
        return SyncLine::Inactive;
    }
    let Some(status) = inputs.sync else {
        return SyncLine::NoSync;
    };
    if status.account() != Some(&account.id) {
        return SyncLine::Waiting;
    }
    match status {
        SyncStatus::Idle => SyncLine::Waiting,
        SyncStatus::Syncing { .. } => SyncLine::Syncing,
        SyncStatus::Synced { at, .. } => SyncLine::Synced {
            ago: ago(*at, inputs.now),
        },
        SyncStatus::Error { kind, at, .. } => SyncLine::Failed {
            kind: *kind,
            ago: ago(*at, inputs.now),
        },
    }
}

fn holds(account: &AccountView, inputs: Inputs<'_>) -> Option<SyncCounts> {
    if !account.active || account.status != AccountStatus::SignedIn {
        return None;
    }
    if let Some(SyncStatus::Synced {
        account: synced,
        counts,
        ..
    }) = inputs.sync
        && synced == &account.id
    {
        return Some(counts.clone());
    }
    inputs
        .last_synced
        .filter(|last| last.account == account.id)
        .map(|last| last.counts.clone())
}

fn actions(
    account: &AccountView,
    signed_in: bool,
    key_pending: bool,
    sync_running: bool,
) -> Vec<InstanceAction> {
    let mut actions = Vec::new();
    if !signed_in {
        actions.extend([InstanceAction::SignIn, InstanceAction::Remove]);
        return actions;
    }
    if key_pending {
        actions.push(InstanceAction::Trust);
    }
    if !account.active {
        actions.push(InstanceAction::MakeActive);
    } else if sync_running && !key_pending {
        actions.push(InstanceAction::SyncNow);
    }
    actions.extend([InstanceAction::SignOut, InstanceAction::Remove]);
    actions
}

pub fn display_name(account: &AccountView, all: &[AccountView]) -> String {
    let label = account.label.trim();
    if label.is_empty() {
        return host(&account.url);
    }
    let collides = all
        .iter()
        .any(|other| other.id != account.id && other.label.trim().eq_ignore_ascii_case(label));
    if collides {
        text::name_with_host(label, &host(&account.url))
    } else {
        label.to_owned()
    }
}

pub(crate) fn host(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}

pub fn short_fingerprint(fingerprint: &KeyFingerprint) -> String {
    let full = fingerprint.as_str();
    let (algorithm, hash) = full.split_once(':').unwrap_or(("", full));
    if hash.chars().count() <= FINGERPRINT_HEAD {
        return full.to_owned();
    }
    let head: String = hash.chars().take(FINGERPRINT_HEAD).collect();
    if algorithm.is_empty() {
        format!("{head}…")
    } else {
        format!("{algorithm}:{head}…")
    }
}
