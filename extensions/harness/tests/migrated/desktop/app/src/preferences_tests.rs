//! Reading the saved preferences: missing fields fall back to defaults,
//! a bad field, and the same project written two ways.

use std::path::Path;

use systemprompt_harness_views::app::settings::UiSettings;
use systemprompt_harness_views::app::settings_store::decode;

use crate::TestResult;

#[test]
fn an_empty_preferences_document_yields_the_defaults() -> TestResult {
    assert_eq!(decode(b"{}")?, UiSettings::default());
    Ok(())
}

#[test]
fn a_missing_field_takes_its_default_while_present_siblings_are_kept() -> TestResult {
    let settings = decode(br#"{"sidebar_open":false,"composer":{"enter_sends":false}}"#)?;
    assert!(!settings.sidebar_open);
    assert!(!settings.composer.enter_sends);
    assert_eq!(
        settings.composer.delivery,
        UiSettings::default().composer.delivery
    );
    assert!(settings.verdict_enabled);
    Ok(())
}

#[test]
fn unknown_fields_in_the_preferences_do_not_stop_the_read() -> TestResult {
    let settings = decode(br#"{"sidebar_open":false,"later_feature":{"x":1}}"#)?;
    assert!(!settings.sidebar_open);
    Ok(())
}

#[test]
#[ignore = "parity ledger: one field of the wrong type makes the whole preferences file decode to defaults instead of only that field"]
fn an_invalid_field_falls_back_alone_and_valid_siblings_are_kept() -> TestResult {
    let settings = decode(br#"{"sidebar_open":"yes","verdict_enabled":false}"#)?;
    assert!(settings.sidebar_open);
    assert!(!settings.verdict_enabled);
    Ok(())
}

#[test]
#[ignore = "parity ledger: a project path written with a trailing separator is remembered as a second project"]
fn equivalent_spellings_of_a_project_path_are_one_project() {
    let mut settings = UiSettings::default();
    settings.remember_project(Path::new("/work/app"));
    settings.remember_project(Path::new("/work/app/"));
    assert_eq!(settings.recent_projects.len(), 1);
}
