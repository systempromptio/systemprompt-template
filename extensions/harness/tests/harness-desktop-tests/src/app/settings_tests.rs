use std::path::{Path, PathBuf};

use systemprompt_engine::store::SessionId;
use systemprompt_harness_shared::Scheme;
use systemprompt_harness_views::app::settings::{DeliveryPref, UiSettings, WindowGeometry};
use systemprompt_harness_views::app::settings_store::{SettingsStore, decode, encode};

use crate::TestResult;

const RECENT_LIMIT: usize = 10;

#[test]
fn a_round_trip_keeps_every_field() -> TestResult {
    let mut settings = UiSettings::default();
    settings.set_scheme(Scheme::Dark);
    settings.remember_project(Path::new("/work/a"));
    settings.project_mut(Path::new("/work/a")).last_session = Some(SessionId::mint(&crate::IDS));
    settings.window = Some(WindowGeometry {
        x: 10.0,
        y: 20.0,
        width: 1200.0,
        height: 800.0,
        maximized: true,
    });
    settings.verdict_enabled = false;
    settings.composer.delivery = DeliveryPref::Queue;
    let bytes = encode(&settings)?;
    let back = decode(&bytes)?;
    assert_eq!(back, settings);
    assert_eq!(back.scheme(), Scheme::Dark);
    Ok(())
}

#[test]
fn missing_fields_take_their_defaults() -> TestResult {
    let back = decode(br#"{"scheme":"light"}"#)?;
    assert_eq!(back.scheme(), Scheme::Light);
    assert!(back.verdict_enabled);
    assert!(back.sidebar_open);
    assert!(back.composer.enter_sends);
    assert_eq!(back.version, UiSettings::default().version);
    Ok(())
}

#[test]
fn an_unknown_scheme_reads_as_system() -> TestResult {
    let back = decode(br#"{"scheme":"sepia"}"#)?;
    assert_eq!(back.scheme(), Scheme::System);
    Ok(())
}

#[test]
fn recent_projects_are_unique_newest_first_and_bounded() {
    let mut settings = UiSettings::default();
    for index in 0..12 {
        settings.remember_project(&PathBuf::from(format!("/work/{index}")));
    }
    settings.remember_project(Path::new("/work/5"));
    assert_eq!(settings.recent_projects.len(), RECENT_LIMIT);
    assert_eq!(settings.recent_projects[0], PathBuf::from("/work/5"));
    assert_eq!(
        settings
            .recent_projects
            .iter()
            .filter(|path| path.ends_with("5"))
            .count(),
        1
    );
    settings.forget_project(Path::new("/work/5"));
    assert_eq!(settings.last_project, None);
}

#[test]
fn window_geometry_must_be_usable() {
    let good = WindowGeometry {
        x: 0.0,
        y: 0.0,
        width: 1000.0,
        height: 700.0,
        maximized: false,
    };
    assert!(good.is_sane());
    assert!(
        !WindowGeometry {
            width: 10.0,
            ..good
        }
        .is_sane()
    );
    assert!(
        !WindowGeometry {
            x: f32::NAN,
            ..good
        }
        .is_sane()
    );
}

#[test]
fn the_store_writes_and_reads_back() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("ui.json");
    let store = SettingsStore::load(path.clone());
    assert_eq!(store.get(), UiSettings::default());
    store.update(|settings| settings.sidebar_open = false);
    store.flush()?;
    let again = SettingsStore::load(path);
    assert!(!again.get().sidebar_open);
    Ok(())
}

#[test]
fn an_unchanged_update_does_not_notify() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = SettingsStore::load(dir.path().join("ui.json"));
    let changes = store.subscribe();
    store.update(|settings| settings.sidebar_open = true);
    assert!(!changes.has_changed()?);
    store.update(|settings| settings.sidebar_open = false);
    assert!(changes.has_changed()?);
    Ok(())
}
