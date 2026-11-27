use systemprompt_harness::sync::HeartbeatState;
use systemprompt_harness_shared::Tone;
use systemprompt_harness_views::governance::heartbeat::{
    HeartbeatLine, Unsupported, heartbeat_line,
};
use systemprompt_models::bridge::instance::WindowStatus;

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
