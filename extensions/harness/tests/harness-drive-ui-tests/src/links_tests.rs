use std::path::Path;

use harness_drive_ui::model::{DriveTarget, local_path, web_link};
use sp_protocol::ids::AccountId;

use crate::fixtures::id;

#[test]
fn links_and_local_paths() {
    assert_eq!(
        web_link("https://sp.example.com/", None),
        "https://sp.example.com/drive"
    );
    assert_eq!(
        web_link("https://sp.example.com", Some(&id(7))),
        format!("https://sp.example.com/drive/{}", id(7))
    );
    let root = Path::new("/home/u/Drive");
    assert_eq!(
        local_path(root, "Reports/q3.md"),
        Some(root.join("Reports").join("q3.md"))
    );
    for bad in ["", "/etc/passwd", "../x", "a//b", "a\\b", "C:/x", "a/./b"] {
        assert_eq!(local_path(root, bad), None, "{bad:?} was accepted");
    }
}

#[test]
fn a_target_changes_with_the_account_or_the_instance() {
    let first = AccountId::new("account-one");
    let second = AccountId::new("account-two");
    let here = DriveTarget::new(Some(first.clone()), "https://a.example.com/");
    assert_eq!(
        here,
        DriveTarget::new(Some(first.clone()), "https://a.example.com/")
    );
    assert_ne!(
        here,
        DriveTarget::new(Some(second), "https://a.example.com/")
    );
    assert_ne!(
        here,
        DriveTarget::new(Some(first.clone()), "https://b.example.com/")
    );
    assert_eq!(here.account(), Some(&first));
    assert_eq!(here.instance_base(), "https://a.example.com/");
}
