use harness_ui::theme::font::{MONO_STACK, UI_STACK, pick};

#[test]
fn picks_the_first_installed_family() {
    let installed = vec![String::from("Consolas"), String::from("Segoe UI")];
    assert_eq!(pick(UI_STACK, &installed), Some("Segoe UI"));
    assert_eq!(pick(MONO_STACK, &installed), Some("Consolas"));
}

#[test]
fn system_ui_is_the_last_resort() {
    assert_eq!(pick(UI_STACK, &[]), Some(".SystemUIFont"));
    assert_eq!(pick(MONO_STACK, &[]), None);
}
