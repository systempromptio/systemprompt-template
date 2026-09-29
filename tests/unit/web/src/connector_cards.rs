//! The connector card for the session-attested control plane: it renders as a
//! first-class MCP server with a live, green state, never as something the
//! person was "signed into automatically".

use systemprompt_web_admin::test_support::{Connection, connector_card};

fn control_plane(entitled: bool) -> Connection {
    Connection {
        provider: "systemprompt".into(),
        display_name: "systemprompt".into(),
        requires_auth: true,
        session_attested: true,
        configured: true,
        entitled,
        status: if entitled {
            "connected"
        } else {
            "not_connected"
        }
        .into(),
        auth_method: Some("session".into()),
        account_id: None,
        account_name: None,
        resource_id: None,
        resource_name: None,
        error_code: None,
        verified_at: entitled.then(|| chrono::Utc::now().to_rfc3339()),
        actions: if entitled {
            vec!["test".into()]
        } else {
            Vec::new()
        },
    }
}

#[test]
fn an_admin_sees_the_control_plane_as_a_connected_server() {
    let card = connector_card(&control_plane(true));
    assert_eq!(card.monogram, "Sp");
    assert_eq!(card.kind, "Control plane");
    assert_eq!(card.status_label, "Connected");
    assert_eq!(card.tone, "ok");
    assert!(card.can_test, "its verification is a live check");
    assert!(card.note.is_none());
    assert!(
        !card.blurb.to_lowercase().contains("automatically") && !card.blurb.contains("Built in"),
        "the card no longer claims an automatic sign-in: {}",
        card.blurb
    );
}

#[test]
fn a_plain_user_is_told_the_control_plane_is_not_open_to_them() {
    let card = connector_card(&control_plane(false));
    assert_ne!(card.status_label, "Built in");
    assert_eq!(card.tone, "muted");
    assert!(!card.can_test);
    assert_eq!(card.note.as_deref(), Some("Not open to your account"));
}
