use systemprompt_device::accounts::AccountsSnapshot;
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_device::view::{
    AccountStatus, AccountView, InstanceSummary, SyncCounts, SyncStatus,
};
use systemprompt_harness_views::signin::address::{
    Credential, choices, credential_kind, grouped_fingerprint, instance_target, opening_address,
    settled,
};
use systemprompt_harness_views::signin::flow::FirstSync;
use systemprompt_harness_views::strings::signin as copy;

fn account(id: &str, status: AccountStatus) -> AccountView {
    AccountView {
        id: AccountId::new(id),
        label: id.to_owned(),
        url: format!("https://{id}.example.com:8443"),
        user: None,
        org: None,
        status,
        active: false,
        fingerprint: None,
        pinned: None,
        trusted: false,
    }
}

#[test]
fn addresses_without_a_scheme_get_https_or_http_for_loopback() {
    assert_eq!(
        instance_target("sp.example.com"),
        Some("https://sp.example.com".to_owned())
    );
    assert_eq!(
        instance_target(" localhost:8080 "),
        Some("http://localhost:8080".to_owned())
    );
    assert_eq!(
        instance_target("127.0.0.1:18101"),
        Some("http://127.0.0.1:18101".to_owned())
    );
    assert_eq!(
        instance_target("https://sp.example.com/base"),
        Some("https://sp.example.com/base".to_owned())
    );
    assert_eq!(instance_target("ftp://sp.example.com"), None);
    assert_eq!(instance_target("two words"), None);
    assert_eq!(instance_target("   "), None);
}

#[test]
fn only_a_64_character_lowercase_hex_value_is_a_one_time_code() {
    let code = "0123456789abcdef".repeat(4);
    assert_eq!(credential_kind(&code), Some(Credential::Code));
    assert_eq!(credential_kind(&code.to_uppercase()), Some(Credential::Pat));
    assert_eq!(credential_kind("sp-live-abc"), Some(Credential::Pat));
    assert_eq!(credential_kind("  "), None);
}

#[test]
fn step_two_lists_signed_in_instances_with_the_newest_first() {
    let snapshot = AccountsSnapshot {
        accounts: vec![
            account("alpha", AccountStatus::SignedIn),
            account("beta", AccountStatus::SignedOut),
            account("gamma", AccountStatus::SignedIn),
        ],
        active: Some(AccountId::new("alpha")),
        last_signed_in: Some(AccountId::new("gamma")),
        secure_storage: true,
        ..AccountsSnapshot::default()
    };
    let list = choices(&snapshot);
    let ids: Vec<&str> = list.iter().map(|choice| choice.account.as_str()).collect();
    assert_eq!(ids, vec!["gamma", "alpha"]);
    assert_eq!(list[0].host, "gamma.example.com:8443");
    assert_eq!(
        opening_address(&snapshot, "https://default"),
        "https://alpha.example.com:8443"
    );
    assert_eq!(
        opening_address(&AccountsSnapshot::default(), "https://default"),
        "https://default"
    );
}

#[test]
fn only_a_synced_or_failed_status_of_the_chosen_instance_ends_the_wait() {
    let chosen = AccountId::new("alpha");
    let at = chrono::Utc::now();
    let synced = SyncStatus::Synced {
        account: chosen.clone(),
        at,
        counts: SyncCounts {
            skills: 5,
            ..SyncCounts::default()
        },
    };
    assert_eq!(
        settled(&synced, &chosen),
        Some(FirstSync::Synced { skills: 5 })
    );
    assert_eq!(
        settled(
            &SyncStatus::Syncing {
                account: chosen.clone()
            },
            &chosen
        ),
        None
    );
    assert_eq!(settled(&SyncStatus::Idle, &chosen), None);
    let other = SyncStatus::Synced {
        account: AccountId::new("beta"),
        at,
        counts: SyncCounts::default(),
    };
    assert_eq!(settled(&other, &chosen), None);
}

#[test]
fn fingerprints_are_grouped_for_reading_and_counts_read_naturally() {
    let fingerprint = format!("sha256:{}", "ab".repeat(32));
    let grouped = grouped_fingerprint(&fingerprint);
    assert!(grouped.starts_with("sha256: abababab abababab"));
    assert_eq!(grouped.split(' ').count(), 9);
    let summary = InstanceSummary {
        skills: 1,
        agents: 2,
        mcp: 1,
        role: None,
    };
    assert_eq!(copy::counts(&summary), "1 skill · 2 agents · 1 MCP server");
}
