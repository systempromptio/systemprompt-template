use harness_notifications_ui::{NotificationsShell, SlackShell};
use harness_shell::{ShellExtension, ShellExtensions};
use sp_notifications::NotificationsExtension;
use sp_slack::SlackExtension;

#[test]
fn each_shell_half_carries_its_instance_half_id() {
    assert_eq!(NotificationsShell.metadata().id, NotificationsExtension::ID);
    assert_eq!(SlackShell.metadata().id, SlackExtension::ID);
}

#[test]
fn slack_depends_on_notifications() {
    assert_eq!(
        SlackShell.metadata().dependencies,
        vec![NotificationsExtension::ID]
    );
    assert_eq!(NotificationsShell.metadata().dependencies, Vec::new());
    assert!(!NotificationsShell.metadata().required);
    assert!(!SlackShell.metadata().required);
}

#[test]
fn both_extensions_are_discovered_at_link_time() {
    let discovered = ShellExtensions::discover().unwrap();
    let ids: Vec<_> = discovered
        .entries()
        .iter()
        .map(|entry| entry.metadata.id.clone())
        .collect();
    assert!(ids.contains(&NotificationsExtension::ID));
    assert!(ids.contains(&SlackExtension::ID));
}
