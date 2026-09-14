//! Regression coverage for native correlation and malformed hook identity.
use systemprompt::identifiers::ClientSessionId;
use systemprompt_web_admin::types::webhook::validate_session_key;

#[test]
fn native_json_session_is_preserved() {
    let id = "6e781d53-34e7-44c9-9e56-e3c78f224182";
    let json = format!(r#"{{"account_uuid":"account","device_id":"device","session_id":"{id}"}}"#);
    let parsed = ClientSessionId::from_metadata_user_id(&json)
        .expect("valid metadata")
        .expect("present session");
    assert_eq!(parsed.as_str(), id);
    assert_eq!(
        ClientSessionId::from_metadata_user_id(&format!("user_hash_account_account_session_{id}"))
            .expect("valid legacy metadata"),
        Some(parsed)
    );
}

#[test]
fn malformed_explicit_native_identity_is_not_silently_absent() {
    for value in [
        r#"{"session_id":42}"#,
        r#"{"session_id":"bad"}"#,
        r#"{"device_id":"x"}"#,
        "user_session_",
        "{broken",
    ] {
        assert!(
            ClientSessionId::from_metadata_user_id(value).is_err(),
            "{value}"
        );
    }
    assert!(
        ClientSessionId::from_metadata_user_id("ordinary-api-user")
            .expect("unrelated metadata")
            .is_none()
    );
}

#[test]
fn hook_session_identity_rejects_empty_and_unsafe_keys() {
    for value in ["", " ", "a/b", "a\nb"] {
        assert!(validate_session_key(value).is_err());
    }
    assert!(validate_session_key("sess_valid-123:client").is_ok());
    assert!(validate_session_key(&"x".repeat(256)).is_err());
}
