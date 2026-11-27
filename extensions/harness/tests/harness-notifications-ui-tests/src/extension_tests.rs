use systemprompt_harness::members::notifications::NotificationsExtension;
use systemprompt_harness::members::slack::SlackExtension;
use systemprompt_harness_shared::contract::{ShellExtension, ShellExtensions};
use systemprompt_harness_views::notifications::{NotificationsShell, SlackShell};
use systemprompt_harness_views::registered::REGISTERED;

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
fn both_extensions_are_in_the_registration_list() {
    let discovered = ShellExtensions::from_registrations(&REGISTERED).unwrap();
    let ids: Vec<_> = discovered
        .entries()
        .iter()
        .map(|entry| entry.metadata.id.clone())
        .collect();
    assert!(ids.contains(&NotificationsExtension::ID));
    assert!(ids.contains(&SlackExtension::ID));

    let core: Vec<_> = systemprompt_harness::extension::REGISTERED
        .iter()
        .map(|member| member.id.clone())
        .collect();
    for shell in &REGISTERED {
        assert!(
            core.contains(&shell.id),
            "{} has no member in core",
            shell.id
        );
        assert_eq!((shell.factory)().metadata().id, shell.id);
    }
}
