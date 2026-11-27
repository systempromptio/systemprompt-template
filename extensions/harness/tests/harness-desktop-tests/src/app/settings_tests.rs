use std::path::{Path, PathBuf};

use harness_desktop::app::settings::{DeliveryPref, UiSettings, WindowGeometry};
use harness_desktop::app::settings_store::{SettingsStore, decode, encode};
use harness_store::SessionId;
use harness_ui::Scheme;

const RECENT_LIMIT: usize = 10;

#[test]
fn a_round_trip_keeps_every_field() {
    let mut settings = UiSettings::default();
    settings.set_scheme(Scheme::Dark);
    settings.remember_project(Path::new("/work/a"));
    settings.project_mut(Path::new("/work/a")).last_session = Some(SessionId::mint());
    settings.window = Some(WindowGeometry {
        x: 10.0,
        y: 20.0,
        width: 1200.0,
        height: 800.0,
        maximized: true,
    });
    settings.verdict_enabled = false;
    settings.composer.delivery = DeliveryPref::Queue;
    let bytes = encode(&settings).unwrap();
    let back = decode(&bytes).unwrap();
    assert_eq!(back, settings);
    assert_eq!(back.scheme(), Scheme::Dark);
}

#[test]
fn missing_fields_take_their_defaults() {
    let back = decode(br#"{"scheme":"light"}"#).unwrap();
    assert_eq!(back.scheme(), Scheme::Light);
    assert!(back.verdict_enabled);
    assert!(back.sidebar_open);
    assert!(back.composer.enter_sends);
    assert_eq!(back.version, UiSettings::default().version);
}

#[test]
fn an_unknown_scheme_reads_as_system() {
    let back = decode(br#"{"scheme":"sepia"}"#).unwrap();
    assert_eq!(back.scheme(), Scheme::System);
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
fn the_store_writes_and_reads_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ui.json");
    let store = SettingsStore::load(path.clone());
    assert_eq!(store.get(), UiSettings::default());
    store.update(|settings| settings.sidebar_open = false);
    store.flush().unwrap();
    let again = SettingsStore::load(path);
    assert!(!again.get().sidebar_open);
}

#[test]
fn an_unchanged_update_does_not_notify() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::load(dir.path().join("ui.json"));
    let changes = store.subscribe();
    store.update(|settings| settings.sidebar_open = true);
    assert!(!changes.has_changed().unwrap());
    store.update(|settings| settings.sidebar_open = false);
    assert!(changes.has_changed().unwrap());
}
