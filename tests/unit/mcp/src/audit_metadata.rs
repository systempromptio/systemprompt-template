//! `AuditMetadata` is serialized straight into `user_activity.metadata`, so
//! its JSON shape is a storage contract: `tool_name` and `server` always
//! present, and `reason` omitted entirely on the success path rather than
//! written as an explicit null (which would make "no reason" and "reason was
//! null" indistinguishable to queries over the JSONB column).

use systemprompt_mcp_shared::AuditMetadata;

#[test]
fn reason_is_omitted_when_absent() {
    let metadata = AuditMetadata {
        tool_name: "search_project_context".to_owned(),
        server: "knowledge-bank".to_owned(),
        reason: None,
    };
    let value = serde_json::to_value(&metadata).expect("serializes");
    let object = value.as_object().expect("serializes to an object");
    assert_eq!(
        object.get("tool_name").and_then(|v| v.as_str()),
        Some("search_project_context")
    );
    assert_eq!(
        object.get("server").and_then(|v| v.as_str()),
        Some("knowledge-bank")
    );
    assert!(
        !object.contains_key("reason"),
        "reason must be absent, not null"
    );
    assert_eq!(object.len(), 2);
}

#[test]
fn reason_is_written_verbatim_when_present() {
    let metadata = AuditMetadata {
        tool_name: "upload_document".to_owned(),
        server: "knowledge-bank".to_owned(),
        reason: Some("requires the admin role".to_owned()),
    };
    let value = serde_json::to_value(&metadata).expect("serializes");
    let object = value.as_object().expect("serializes to an object");
    assert_eq!(
        object.get("reason").and_then(|v| v.as_str()),
        Some("requires the admin role")
    );
    assert_eq!(object.len(), 3);
}
