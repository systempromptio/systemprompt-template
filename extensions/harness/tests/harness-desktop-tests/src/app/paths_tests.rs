use std::path::{Path, PathBuf};

use systemprompt_harness_views::app::paths::{AppPaths, Forbidden, choose_directory, display_name};

fn forbidden() -> Forbidden {
    Forbidden {
        system_root: Some(PathBuf::from(r"C:\Windows")),
        install_dir: Some(PathBuf::from(r"C:\Users\me\AppData\Local\Programs\Harness")),
    }
}

#[test]
fn every_directory_sits_under_the_root() {
    let paths = AppPaths::under(PathBuf::from("/root/app"), PathBuf::from("/home/me"));
    for dir in [
        &paths.data,
        &paths.config,
        &paths.cache,
        &paths.state,
        &paths.tmp,
        &paths.logs,
    ] {
        assert!(dir.starts_with("/root/app"), "{}", dir.display());
    }
    assert!(paths.store_file().starts_with(&paths.data));
    assert!(paths.accounts_file().starts_with(&paths.state));
    assert!(paths.ui_settings_file().starts_with(&paths.state));
    let engine = paths.engine_paths();
    assert_eq!(engine.home, PathBuf::from("/home/me"));
    assert_eq!(engine.state, paths.state);
}

#[test]
fn system_and_install_folders_are_rejected() {
    let forbidden = forbidden();
    assert!(forbidden.rejects(Path::new(r"C:\Windows\System32")));
    assert!(forbidden.rejects(Path::new(r"c:\windows")));
    assert!(forbidden.rejects(Path::new(r"C:\Users\me\AppData\Local\Programs\Harness")));
    assert!(!forbidden.rejects(Path::new(r"C:\WindowsApps2\project")));
    assert!(!forbidden.rejects(Path::new(r"D:\work\app")));
}

#[test]
fn the_requested_directory_wins_then_the_last_project_then_home() {
    let base = std::env::temp_dir();
    let home = base.join("home");
    let a = base.join("a");
    let b = base.join("b");
    let none = Forbidden::default();
    let exists = |_: &Path| true;
    assert_eq!(
        choose_directory(Some(a.clone()), Some(b.clone()), &home, &none, exists),
        a
    );
    assert_eq!(
        choose_directory(None, Some(b.clone()), &home, &none, exists),
        b
    );
    assert_eq!(choose_directory(None, None, &home, &none, exists), home);
    let only_b = |path: &Path| path == b.as_path();
    assert_eq!(
        choose_directory(Some(a), Some(b.clone()), &home, &none, only_b),
        b
    );
}

#[test]
fn a_forbidden_or_relative_request_falls_back() {
    let home = PathBuf::from(r"C:\Users\me");
    let exists = |_: &Path| true;
    let system = Some(PathBuf::from(r"C:\Windows\System32"));
    assert_eq!(
        choose_directory(system, None, &home, &forbidden(), exists),
        home
    );
    assert_eq!(
        choose_directory(
            Some(PathBuf::from("relative")),
            None,
            &home,
            &Forbidden::default(),
            exists
        ),
        home
    );
}

#[test]
fn display_names_use_the_last_component() {
    assert_eq!(display_name(Path::new("/work/my-app")), "my-app");
    assert_eq!(display_name(Path::new("/")), "/");
}
