use harness_desktop::views::settings::shortcuts::{APP_KEYS, key_caps};

#[test]
fn modifiers_and_keys_become_caps() {
    let primary = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };
    assert_eq!(
        key_caps("secondary-shift-p"),
        vec![primary.to_owned(), "Shift".to_owned(), "P".to_owned()]
    );
    assert_eq!(
        key_caps("secondary-,"),
        vec![primary.to_owned(), ",".to_owned()]
    );
    assert_eq!(key_caps("escape"), vec!["Esc".to_owned()]);
}

#[test]
fn a_minus_key_is_kept() {
    assert_eq!(key_caps("ctrl--"), vec!["Ctrl".to_owned(), "-".to_owned()]);
}

#[test]
fn every_app_binding_has_caps() {
    for (_, binding) in APP_KEYS {
        assert!(!key_caps(binding).is_empty(), "{binding}");
    }
}
