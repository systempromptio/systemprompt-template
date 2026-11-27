use chrono::{DateTime, Utc};
use systemprompt_device::accounts::{AccountsSnapshot, PendingKeyOffer};
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_device::view::{
    AccountStatus, AccountView, KeyFingerprint, KeyOffer, SyncCounts, SyncErrorKind, SyncStatus,
};
use systemprompt_harness_views::governance::model::{
    Ago, Fix, Inputs, InstanceAction, LastSynced, SyncLine, ago, display_name, fix, instances,
    remember_synced, short_fingerprint,
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

fn counts() -> SyncCounts {
    SyncCounts {
        skills: 3,
        agents: 1,
        mcp: 2,
        rules: 4,
        warnings: vec!["left out".to_owned()],
    }
}

fn inputs<'a>(
    accounts: &'a AccountsSnapshot,
    sync: Option<&'a SyncStatus>,
    last: Option<&'a LastSynced>,
) -> Inputs<'a> {
    Inputs {
        accounts,
        sync,
        last_synced: last,
        now: at(10_000),
    }
}

#[test]
fn ago_buckets_by_minute_hour_and_day() {
    assert_eq!(ago(at(100), at(130)), Ago::JustNow);
    assert_eq!(ago(at(0), at(150)), Ago::Minutes(2));
    assert_eq!(ago(at(0), at(7_300)), Ago::Hours(2));
    assert_eq!(ago(at(0), at(200_000)), Ago::Days(2));
    assert_eq!(ago(at(500), at(100)), Ago::JustNow);
}

#[test]
fn the_active_instance_reports_its_sync_and_holdings() {
    let accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    let status = SyncStatus::Synced {
        account: AccountId::new("a1"),
        at: at(9_880),
        counts: counts(),
    };
    let rows = instances(inputs(&accounts, Some(&status), None));
    assert_eq!(
        rows[0].sync,
        SyncLine::Synced {
            ago: Ago::Minutes(2)
        }
    );
    assert_eq!(rows[0].holds, Some(counts()));
    assert_eq!(
        rows[0].actions,
        vec![
            InstanceAction::SyncNow,
            InstanceAction::SignOut,
            InstanceAction::Remove
        ]
    );
}

#[test]
fn holdings_survive_a_later_sync_attempt() {
    let accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    let synced = SyncStatus::Synced {
        account: AccountId::new("a1"),
        at: at(1),
        counts: counts(),
    };
    let last = remember_synced(None, &synced);
    let syncing = SyncStatus::Syncing {
        account: AccountId::new("a1"),
    };
    let last = remember_synced(last, &syncing);
    let rows = instances(inputs(&accounts, Some(&syncing), last.as_ref()));
    assert_eq!(rows[0].sync, SyncLine::Syncing);
    assert_eq!(rows[0].holds, Some(counts()));
}

#[test]
fn errors_carry_their_kind_and_a_fix() {
    let accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    let status = SyncStatus::Error {
        account: AccountId::new("a1"),
        kind: SyncErrorKind::Refused,
        at: at(10_000),
    };
    let rows = instances(inputs(&accounts, Some(&status), None));
    assert_eq!(rows[0].sync.failure(), Some(SyncErrorKind::Refused));
    assert_eq!(fix(SyncErrorKind::Refused), Fix::SignIn);
    assert_eq!(fix(SyncErrorKind::KeyChanged), Fix::ReviewKey);
    assert_eq!(fix(SyncErrorKind::Unreachable), Fix::Wait);
}

#[test]
fn signed_out_and_inactive_instances_offer_their_own_actions() {
    let accounts = snapshot(vec![
        account("a1", "Alpha", true, AccountStatus::SignedIn),
        account("b2", "Beta", false, AccountStatus::SignedIn),
        account("c3", "Gamma", false, AccountStatus::SignedOut),
    ]);
    let rows = instances(inputs(&accounts, None, None));
    assert_eq!(rows[0].sync, SyncLine::NoSync);
    assert_eq!(
        rows[0].actions,
        vec![InstanceAction::SignOut, InstanceAction::Remove]
    );
    assert_eq!(rows[1].sync, SyncLine::Inactive);
    assert_eq!(rows[1].actions[0], InstanceAction::MakeActive);
    assert_eq!(rows[2].sync, SyncLine::SignedOut);
    assert_eq!(
        rows[2].actions,
        vec![InstanceAction::SignIn, InstanceAction::Remove]
    );
    assert_eq!(rows[2].holds, None);
}

#[test]
fn a_pending_key_comes_first_and_stops_sync_now() {
    let mut accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    accounts.offers = vec![offer("a1", Some("sha256:pinned"))];
    let status = SyncStatus::Idle;
    let rows = instances(inputs(&accounts, Some(&status), None));
    let key = rows[0].key.clone();
    assert_eq!(
        key.as_ref().map(|key| key.offered.as_str()),
        Some("sha256:offered")
    );
    assert_eq!(
        key.and_then(|key| key.pinned)
            .map(|pinned| pinned.as_str().to_owned()),
        Some("sha256:pinned".to_owned())
    );
    assert_eq!(
        rows[0].actions,
        vec![
            InstanceAction::Trust,
            InstanceAction::SignOut,
            InstanceAction::Remove
        ]
    );
}

#[test]
fn another_accounts_status_reads_as_waiting() {
    let accounts = snapshot(vec![account("a1", "Alpha", true, AccountStatus::SignedIn)]);
    let status = SyncStatus::Syncing {
        account: AccountId::new("zz"),
    };
    assert_eq!(
        instances(inputs(&accounts, Some(&status), None))[0].sync,
        SyncLine::Waiting
    );
}

#[test]
fn duplicate_labels_gain_their_host() {
    let all = vec![
        account("a1", "Acme", true, AccountStatus::SignedIn),
        account("b2", "acme", false, AccountStatus::SignedIn),
    ];
    assert_eq!(display_name(&all[0], &all), "Acme (a1.example.com)");
    let single = vec![account("a1", "Acme", true, AccountStatus::SignedIn)];
    assert_eq!(display_name(&single[0], &single), "Acme");
    let unnamed = vec![account("a1", "  ", true, AccountStatus::SignedIn)];
    assert_eq!(display_name(&unnamed[0], &unnamed), "a1.example.com");
}

#[test]
fn fingerprints_shorten_to_their_head() {
    let long = KeyFingerprint::new(format!("sha256:{}", "ab".repeat(32)));
    assert_eq!(
        short_fingerprint(&long),
        format!("sha256:{}…", "ab".repeat(8))
    );
    assert_eq!(
        short_fingerprint(&KeyFingerprint::new("sha256:abc")),
        "sha256:abc"
    );
}
