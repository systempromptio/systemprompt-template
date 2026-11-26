//! Config loading is deliberately non-fatal: a broken `services/web/` tree
//! degrades the affected section to "absent" instead of taking the server down.

use systemprompt_web_site::config_loader::{ConfigError, log_and_discard_err};

fn broken() -> Result<Option<String>, ConfigError> {
    Err(ConfigError::Parse {
        config_name: "navigation.yaml".to_owned(),
        message: "unexpected key".to_owned(),
    })
}

#[test]
fn a_successful_load_is_returned() {
    assert_eq!(
        log_and_discard_err(Ok(Some("loaded".to_owned())), "test").as_deref(),
        Some("loaded")
    );
}

#[test]
fn an_absent_config_is_none_without_being_an_error() {
    assert!(log_and_discard_err::<String>(Ok(None), "test").is_none());
}

#[test]
fn a_failed_load_is_discarded_as_none_rather_than_panicking() {
    assert!(log_and_discard_err(broken(), "test").is_none());
}
