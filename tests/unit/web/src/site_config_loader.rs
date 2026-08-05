//! Config loading is deliberately non-fatal: a broken `services/web/` tree
//! degrades the affected section to "absent" instead of taking the server down,
//! and the result is memoised so a failing load is not retried per request.

use std::sync::OnceLock;

use systemprompt_web_site::config_loader::{ConfigError, log_and_discard_err};

fn ok_value() -> Result<Option<String>, ConfigError> {
    Ok(Some("loaded".to_owned()))
}

fn absent() -> Result<Option<String>, ConfigError> {
    Ok(None)
}

fn broken() -> Result<Option<String>, ConfigError> {
    Err(ConfigError::Parse {
        config_name: "navigation.yaml".to_owned(),
        message: "unexpected key".to_owned(),
    })
}

#[test]
fn a_successful_load_is_returned_and_then_memoised() {
    static LOCK: OnceLock<Result<Option<String>, String>> = OnceLock::new();

    assert_eq!(
        log_and_discard_err(&LOCK, ok_value, "test").as_deref(),
        Some("loaded")
    );
    assert_eq!(
        log_and_discard_err(&LOCK, broken, "test").as_deref(),
        Some("loaded"),
        "the memoised value must win over a second initialiser"
    );
}

#[test]
fn an_absent_config_is_none_without_being_an_error() {
    static LOCK: OnceLock<Result<Option<String>, String>> = OnceLock::new();

    assert!(log_and_discard_err(&LOCK, absent, "test").is_none());
}

#[test]
fn a_failed_load_is_discarded_as_none_rather_than_panicking() {
    static LOCK: OnceLock<Result<Option<String>, String>> = OnceLock::new();

    assert!(log_and_discard_err(&LOCK, broken, "test").is_none());
    assert!(
        log_and_discard_err(&LOCK, ok_value, "test").is_none(),
        "a failure is cached too; the section stays absent for the process"
    );
}
