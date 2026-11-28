//! The preferences store: changes read back before they are written, a burst
//! written as one file, a failed write kept for the next, and a restart.

use serde_json::{Map, Value};
use systemprompt_harness::extension::SettingsAccess;
use systemprompt_harness_views::app::settings_store::{SettingsStore, decode};
use systemprompt_identifiers::extension_id;

use crate::TestResult;

fn folder(path: &str) -> Map<String, Value> {
    std::iter::once(("folder".to_owned(), Value::from(path))).collect()
}

#[test]
fn a_missing_file_starts_from_defaults_and_is_created_on_the_first_write() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    let store = SettingsStore::load(file.clone());
    assert!(store.get().recent_projects.is_empty());
    assert!(!file.exists());
    store.flush()?;
    assert!(file.exists());
    Ok(())
}

#[test]
fn a_change_reads_back_before_it_is_written() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    let store = SettingsStore::load(file.clone());
    let drive = extension_id!("drive");
    store.update(|settings| settings.remember_project("/work/a".as_ref()));
    store.write(&drive, folder("/docs"));
    assert_eq!(store.get().last_project, Some("/work/a".into()));
    assert_eq!(store.read(&drive), Some(folder("/docs")));
    assert!(!file.exists());
    Ok(())
}

#[test]
fn every_change_raises_the_revision() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = SettingsStore::load(dir.path().join("ui-settings.json"));
    let mut changes = store.changes();
    let start = *changes.borrow_and_update();
    store.update(|settings| settings.sidebar_open = !settings.sidebar_open);
    let first = *changes.borrow_and_update();
    store.update(|settings| settings.verdict_enabled = !settings.verdict_enabled);
    let second = *changes.borrow_and_update();
    assert!(start < first && first < second);
    Ok(())
}

#[test]
fn clearing_one_extensions_preferences_leaves_the_others() -> TestResult {
    let dir = tempfile::tempdir()?;
    let store = SettingsStore::load(dir.path().join("ui-settings.json"));
    let (drive, brain) = (extension_id!("drive"), extension_id!("brain"));
    store.write(&drive, folder("/docs"));
    store.write(&brain, folder("/notes"));
    store.write(&drive, Map::new());
    assert_eq!(store.read(&drive), Some(Map::new()));
    assert_eq!(store.read(&brain), Some(folder("/notes")));
    Ok(())
}

#[test]
fn a_burst_of_changes_is_written_as_one_file_with_the_latest_values() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    let store = SettingsStore::load(file.clone());
    for project in ["/work/a", "/work/b", "/work/c"] {
        store.update(|settings| settings.remember_project(project.as_ref()));
    }
    store.update(|settings| settings.sidebar_open = false);
    store.flush()?;
    let written = decode(&std::fs::read(&file)?)?;
    assert_eq!(written.last_project, Some("/work/c".into()));
    assert_eq!(written.recent_projects.len(), 3);
    assert!(!written.sidebar_open);
    Ok(())
}

#[test]
fn a_failed_write_keeps_the_change_until_a_later_write_succeeds() -> TestResult {
    let dir = tempfile::tempdir()?;
    let blocker = dir.path().join("state");
    std::fs::write(&blocker, "not a directory")?;
    let file = blocker.join("ui-settings.json");
    let store = SettingsStore::load(file.clone());
    store.update(|settings| settings.remember_project("/work/kept".as_ref()));
    assert!(store.flush().is_err());
    assert_eq!(store.get().last_project, Some("/work/kept".into()));
    std::fs::remove_file(&blocker)?;
    std::fs::create_dir(&blocker)?;
    store.flush()?;
    assert_eq!(
        SettingsStore::load(file).get().last_project,
        Some("/work/kept".into())
    );
    Ok(())
}

#[test]
fn preferences_survive_a_restart() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    let store = SettingsStore::load(file.clone());
    store.update(|settings| {
        settings.remember_project("/work/a".as_ref());
        settings.sidebar_open = false;
    });
    store.flush()?;
    drop(store);
    let reopened = SettingsStore::load(file).get();
    assert_eq!(reopened.last_project, Some("/work/a".into()));
    assert!(!reopened.sidebar_open);
    Ok(())
}

#[test]
fn a_file_the_app_wrote_is_rewritten_in_the_same_shape() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    let store = SettingsStore::load(file.clone());
    store.update(|settings| settings.remember_project("/work/a".as_ref()));
    store.flush()?;
    let first = std::fs::read(&file)?;
    SettingsStore::load(file.clone()).flush()?;
    assert_eq!(std::fs::read(&file)?, first);
    Ok(())
}

#[test]
#[ignore = "parity ledger: an unreadable preferences file is overwritten by the next save, not set aside"]
fn an_unreadable_preferences_file_is_set_aside_rather_than_overwritten() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("ui-settings.json");
    std::fs::write(&file, "{oops")?;
    let store = SettingsStore::load(file.clone());
    store.update(|settings| settings.sidebar_open = false);
    store.flush()?;
    let kept = std::fs::read_dir(dir.path())?
        .filter_map(Result::ok)
        .filter(|entry| entry.path() != file)
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .any(|text| text == "{oops");
    assert!(kept, "the unreadable file must be kept beside the new one");
    Ok(())
}
