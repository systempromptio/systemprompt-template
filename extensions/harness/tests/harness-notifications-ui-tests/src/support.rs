use chrono::{DateTime, Utc};
use serde_json::json;
use systemprompt_harness::members::notifications::{Notification, NotificationId};

pub(crate) fn id_of(index: usize) -> NotificationId {
    NotificationId::try_new(format!("00000000-0000-4000-8000-{index:012x}")).unwrap()
}

pub(crate) fn at(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap()
        .with_timezone(&Utc)
}

pub(crate) fn notification(index: usize, read: bool, attention: &str) -> Notification {
    let user = "1ecce4e3-bc87-441d-b374-ea29bab88da8";
    let created = "2026-11-20T10:00:00Z";
    serde_json::from_value(json!({
        "id": id_of(index).as_str(),
        "recipient": user,
        "kind": "document.shared",
        "stream": null,
        "actor": user,
        "actor_is_agent": false,
        "via_agent": null,
        "subjects": [],
        "attention": attention,
        "attention_reason": "default",
        "action": {
            "prompt": "Open it",
            "open_url": null,
            "document_id": null,
            "comment_id": null,
            "kind": "document.shared",
            "actor": user
        },
        "title": format!("Item {index}"),
        "body": "",
        "coalesce_count": 1,
        "first_event_at": created,
        "last_event_at": created,
        "created_at": created,
        "read_at": if read { json!("2026-11-20T11:00:00Z") } else { json!(null) },
        "escalated_from": null,
        "escalation_note": null,
        "held_until": null,
        "delivery_state": "delivered"
    }))
    .unwrap()
}
