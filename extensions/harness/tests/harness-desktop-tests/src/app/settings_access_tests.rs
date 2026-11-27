use serde_json::{Map, Value, json};
use systemprompt_harness::extension::SettingsAccess;
use systemprompt_harness_views::app::settings_store::SettingsStore;
use systemprompt_identifiers::extension_id;

fn folder(path: &str) -> Map<String, Value> {
    let Value::Object(values) = json!({ "folder": path }) else {
        unreachable!("a JSON object literal");
    };
    values
}

#[test]
fn an_extensions_preferences_are_kept_under_its_id_and_announced() {
    let dir = tempfile::tempdir().unwrap();
    let store = SettingsStore::load(dir.path().join("ui-settings.json"));
    let drive = extension_id!("drive");
    let mut changes = store.changes();
    assert_eq!(store.read(&drive), None);

    store.write(&drive, folder("/docs"));
    assert!(changes.has_changed().unwrap());
    let seen = *changes.borrow_and_update();
    assert_eq!(store.read(&drive), Some(folder("/docs")));
    assert_eq!(store.read(&extension_id!("brain")), None);

    store.write(&drive, folder("/docs"));
    assert!(!changes.has_changed().unwrap());
    store.update(|settings| settings.sidebar_open = !settings.sidebar_open);
    assert!(changes.has_changed().unwrap());
    assert_ne!(*changes.borrow_and_update(), seen);
}

#[test]
fn an_extensions_preferences_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ui-settings.json");
    let drive = extension_id!("drive");
    let store = SettingsStore::load(path.clone());
    store.write(&drive, folder("/work/drive"));
    store.flush().unwrap();

    let reopened = SettingsStore::load(path);
    assert_eq!(reopened.read(&drive), Some(folder("/work/drive")));
    assert_eq!(
        reopened.get().extensions.keys().collect::<Vec<_>>(),
        [&drive]
    );
}
