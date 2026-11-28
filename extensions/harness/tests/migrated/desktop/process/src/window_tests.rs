//! The saved window place: what counts as usable, and how an unreadable or
//! partial preferences file is treated.

use systemprompt_harness_views::app::settings::WindowGeometry;
use systemprompt_harness_views::app::settings_store::{SettingsStore, decode};

use crate::TestResult;

const fn place(x: f32, y: f32, width: f32, height: f32) -> WindowGeometry {
    WindowGeometry {
        x,
        y,
        width,
        height,
        maximized: false,
    }
}

#[test]
fn a_saved_place_on_a_display_at_negative_coordinates_is_kept() {
    assert!(place(-1920.0, -400.0, 1280.0, 800.0).is_sane());
}

#[test]
fn a_window_snapped_just_past_the_left_edge_is_kept() {
    assert!(place(-7.0, 0.0, 974.0, 1047.0).is_sane());
}

#[test]
fn an_unusable_saved_size_is_not_restored() {
    assert!(!place(0.0, 0.0, 0.0, -1.0).is_sane());
    assert!(!place(0.0, 0.0, 1280.0, 10.0).is_sane());
    assert!(!place(f32::NAN, 0.0, 1280.0, 800.0).is_sane());
}

#[test]
fn an_unreadable_preferences_file_does_not_stop_the_start() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    for junk in ["\"junk\"", "{oops", "[]"] {
        std::fs::write(&file, junk)?;
        let settings = SettingsStore::load(file.clone()).get();
        assert_eq!(settings.window, None);
        assert_eq!(settings.last_project, None);
    }
    Ok(())
}

#[test]
#[ignore = "parity ledger: a saved maximised flag without a saved place makes the whole preferences file unreadable"]
fn a_maximised_flag_is_kept_without_a_saved_place() -> TestResult {
    let settings = decode(br#"{"window":{"maximized":true}}"#)?;
    assert_eq!(settings.window.map(|window| window.maximized), Some(true));
    Ok(())
}
