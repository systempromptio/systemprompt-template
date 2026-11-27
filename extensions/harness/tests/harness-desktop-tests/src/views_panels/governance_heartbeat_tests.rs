use harness_desktop::views::governance::heartbeat::{HeartbeatLine, Unsupported, heartbeat_line};
use harness_ui::Tone;
use sp_protocol::wire::heartbeat::WindowStatus;
use sp_sync::HeartbeatState;

#[test]
fn heartbeat_states_map_to_lines() {
    assert_eq!(heartbeat_line(None), HeartbeatLine::Unavailable);
    assert_eq!(
        heartbeat_line(Some(&HeartbeatState::Online)).tone(),
        Tone::Success
    );
    let old = HeartbeatState::Unsupported {
        min_bridge_version: "0.90.0".to_owned(),
        window: Some(WindowStatus::Below),
    };
    assert_eq!(
        heartbeat_line(Some(&old)),
        HeartbeatLine::Unsupported {
            minimum: "0.90.0".to_owned(),
            reason: Unsupported::TooOld
        }
    );
    let refused = HeartbeatState::Unsupported {
        min_bridge_version: "1.0.0".to_owned(),
        window: None,
    };
    assert!(matches!(
        heartbeat_line(Some(&refused)),
        HeartbeatLine::Unsupported {
            reason: Unsupported::Refused,
            ..
        }
    ));
}
