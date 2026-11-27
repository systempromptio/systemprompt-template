use chrono::{DateTime, Utc};
use harness_desktop::views::governance::model::Fix;
use harness_desktop::views::governance::pill_model::{PillStatus, SwitchTarget, pill};
use harness_ui::{IconName, Tone};
use sp_accounts::{AccountsSnapshot, PendingKeyOffer};
use sp_protocol::ids::AccountId;
use sp_protocol::view::{
    AccountStatus, AccountView, KeyFingerprint, KeyOffer, SyncCounts, SyncErrorKind, SyncStatus,
};

fn at(seconds: i64) -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + chrono::TimeDelta::seconds(seconds)
}

fn account(id: &str, label: &str, active: bool, status: AccountStatus) -> AccountView {
    AccountView {
        id: AccountId::new(id),
        label: label.to_owned(),
        url: format!("https://{id}.example.com"),
        user: Some("ada@example.com".to_owned()),
        org: Some("Example".to_owned()),
        status,
        active,
        fingerprint: None,
        pinned: None,
        trusted: true,
    }
}

fn snapshot(accounts: Vec<AccountView>) -> AccountsSnapshot {
    let active = accounts
        .iter()
        .find(|account| account.active)
        .map(|account| account.id.clone());
    AccountsSnapshot {
        accounts,
        active,
        secure_storage: true,
        ..AccountsSnapshot::default()
    }
}

fn offer(id: &str, pinned: Option<&str>) -> PendingKeyOffer {
    PendingKeyOffer {
        account: AccountId::new(id),
        offer: KeyOffer {
            pinned: pinned.map(KeyFingerprint::new),
            offered: KeyFingerprint::new("sha256:offered"),
        },
    }
}

#[test]
fn without_a_signed_in_active_instance_the_pill_asks_to_sign_in() {
    let accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedOut)]);
    let model = pill(&accounts, None, at(0), "Governance");
    assert_eq!(model.label, "Governance");
    assert_eq!(model.status, PillStatus::SignedOut);
    assert!(model.status.needs_attention());
    assert!(!model.can_sync);
}

#[test]
fn sync_states_choose_icon_and_tone() {
    let accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    let syncing = SyncStatus::Syncing {
        account: AccountId::new("a1"),
    };
    let model = pill(&accounts, Some(&syncing), at(0), "Governance");
    assert_eq!(model.label, "Alpha");
    assert_eq!(model.status, PillStatus::Syncing);
    assert!(model.status.pulse());
    let failed = SyncStatus::Error {
        account: AccountId::new("a1"),
        kind: SyncErrorKind::Unreachable,
        at: at(0),
    };
    let model = pill(&accounts, Some(&failed), at(0), "Governance");
    assert_eq!(model.status.tone(), Tone::Danger);
    assert_eq!(model.status.fix(), Some(Fix::Wait));
    assert!(model.can_sync);
}

#[test]
fn a_pending_key_outranks_the_sync_status() {
    let mut accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    accounts.offers = vec![offer("a1", None)];
    let synced = SyncStatus::Synced {
        account: AccountId::new("a1"),
        at: at(0),
        counts: SyncCounts::default(),
    };
    let model = pill(&accounts, Some(&synced), at(30), "Governance");
    assert_eq!(model.status, PillStatus::KeyPending { rotated: false });
    assert_eq!(model.status.icon(), IconName::Key);
    assert_eq!(model.status.fix(), Some(Fix::ReviewKey));
    assert!(!model.can_sync);
}

#[test]
fn only_other_signed_in_instances_are_offered_for_switching() {
    let accounts = snapshot(vec![
        account("a1", "Alpha", true, AccountStatus::SignedIn),
        account("b2", "Beta", false, AccountStatus::SignedIn),
        account("c3", "Gamma", false, AccountStatus::SignedOut),
    ]);
    let model = pill(&accounts, None, at(0), "Governance");
    assert_eq!(model.status, PillStatus::NoSync);
    assert_eq!(
        model.others,
        vec![SwitchTarget {
            id: AccountId::new("b2"),
            name: "Beta".to_owned()
        }]
    );
}
