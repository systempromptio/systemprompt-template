//! The project directory chosen at launch: a missing folder, a file, the last
//! project and a first run.

use std::path::{Path, PathBuf};

use systemprompt_harness_views::app::deeplink::parse_args;
use systemprompt_harness_views::app::paths::{Forbidden, choose_directory};
use systemprompt_harness_views::app::settings_store::SettingsStore;

use crate::TestResult;

fn chosen(requested: Option<PathBuf>, last: Option<PathBuf>, home: &Path) -> PathBuf {
    choose_directory(requested, last, home, &Forbidden::default(), Path::is_dir)
}

#[test]
fn a_missing_directory_named_at_launch_opens_home_instead_of_failing() -> TestResult {
    let home = tempfile::tempdir()?;
    let missing = home.path().join("no-such-directory");
    let scheme = systemprompt_harness_brand::identity().scheme();
    let request = parse_args(scheme, [missing.to_string_lossy().into_owned()])?;
    assert_eq!(request.directory.as_deref(), Some(missing.as_path()));
    assert_eq!(chosen(request.directory, None, home.path()), home.path());
    Ok(())
}

#[test]
fn a_file_named_at_launch_opens_home() -> TestResult {
    let home = tempfile::tempdir()?;
    let file = home.path().join("notes.txt");
    std::fs::write(&file, "")?;
    assert_eq!(chosen(Some(file), None, home.path()), home.path());
    Ok(())
}

#[test]
fn the_last_project_reopens_only_while_it_is_still_a_folder() -> TestResult {
    let home = tempfile::tempdir()?;
    let project = tempfile::tempdir()?;
    let last = project.path().to_path_buf();
    assert_eq!(chosen(None, Some(last.clone()), home.path()), last);
    assert_eq!(
        chosen(None, Some(PathBuf::from("relative/project")), home.path()),
        home.path()
    );
    drop(project);
    assert_eq!(chosen(None, Some(last), home.path()), home.path());
    Ok(())
}

#[test]
fn a_first_run_opens_in_the_home_folder() -> TestResult {
    let home = tempfile::tempdir()?;
    let settings = SettingsStore::load(home.path().join("state").join("ui-settings.json"));
    let last = settings.get().last_project;
    assert_eq!(last, None);
    assert_eq!(chosen(None, last, home.path()), home.path());
    Ok(())
}
