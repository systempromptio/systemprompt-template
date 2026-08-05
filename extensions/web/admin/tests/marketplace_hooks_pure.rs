#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use std::path::Path;

use systemprompt_web_admin::repositories::marketplace::hooks::list_configured_hooks;

fn hook(root: &Path, dir: &str, body: &str) {
    let path = root.join("hooks").join(dir);
    std::fs::create_dir_all(&path).expect("create hook dir");
    std::fs::write(path.join("config.yaml"), body).expect("write hook config");
}

fn roles(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

const GOVERN: &str =
    "id: govern\nevent: PreToolUse\nmatcher: Bash\ncommand: /usr/bin/govern\nasync: true\n";

#[test]
fn no_hooks_directory_yields_empty() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    assert!(list_configured_hooks(dir.path(), &roles(&["admin"]))?.is_empty());
    Ok(())
}

#[test]
fn hooks_path_that_is_a_file_yields_empty() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("hooks"), "not a directory")?;
    assert!(list_configured_hooks(dir.path(), &roles(&["admin"]))?.is_empty());
    Ok(())
}

#[test]
fn configured_hook_carries_event_matcher_and_command() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    hook(dir.path(), "govern", GOVERN);
    let out = list_configured_hooks(dir.path(), &roles(&["user"]))?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "govern");
    assert_eq!(out[0].plugin_id.as_str(), "govern");
    assert_eq!(out[0].event, "PreToolUse");
    assert_eq!(out[0].matcher, "Bash");
    assert_eq!(out[0].command, "/usr/bin/govern");
    assert!(out[0].is_async);
    assert_eq!(out[0].timeout_ms, None);
    Ok(())
}

#[test]
fn matcher_defaults_to_star_and_async_to_false() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    hook(dir.path(), "track", "id: track\nevent: Stop\n");
    let out = list_configured_hooks(dir.path(), &roles(&["user"]))?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].matcher, "*");
    assert!(!out[0].is_async);
    assert_eq!(out[0].command, "");
    Ok(())
}

#[test]
fn missing_id_falls_back_to_directory_name() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    hook(dir.path(), "dir-named", "event: SessionStart\n");
    let out = list_configured_hooks(dir.path(), &roles(&["user"]))?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "dir-named");
    Ok(())
}

#[test]
fn disabled_hook_is_hidden_from_users_and_shown_to_admins() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    hook(dir.path(), "off", "id: off\nevent: Stop\nenabled: false\n");
    assert!(list_configured_hooks(dir.path(), &roles(&["user"]))?.is_empty());
    let as_admin = list_configured_hooks(dir.path(), &roles(&["admin"]))?;
    assert_eq!(as_admin.len(), 1);
    assert_eq!(as_admin[0].id, "off");
    Ok(())
}

#[test]
fn visible_to_restricts_non_admins_to_listed_roles() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    hook(
        dir.path(),
        "finance",
        "id: finance\nevent: Stop\nvisible_to:\n  - finance\n  - ops\n",
    );
    assert!(list_configured_hooks(dir.path(), &roles(&["user"]))?.is_empty());
    assert_eq!(
        list_configured_hooks(dir.path(), &roles(&["ops"]))?.len(),
        1
    );
    assert_eq!(
        list_configured_hooks(dir.path(), &roles(&["user", "finance"]))?.len(),
        1
    );
    assert_eq!(
        list_configured_hooks(dir.path(), &roles(&["admin"]))?.len(),
        1,
        "admin bypasses visible_to"
    );
    Ok(())
}

#[test]
fn empty_visible_to_is_visible_to_everyone() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    hook(
        dir.path(),
        "open",
        "id: open\nevent: Stop\nvisible_to: []\n",
    );
    assert_eq!(list_configured_hooks(dir.path(), &[])?.len(), 1);
    Ok(())
}

#[test]
fn entries_without_a_config_or_with_bad_yaml_are_skipped() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("hooks").join("bare"))?;
    std::fs::write(dir.path().join("hooks").join("loose.yaml"), "id: loose\n")?;
    hook(dir.path(), "broken", "event: [unclosed\n");
    hook(dir.path(), "unknown-event", "id: x\nevent: NotAnEvent\n");
    hook(dir.path(), "govern", GOVERN);
    let out = list_configured_hooks(dir.path(), &roles(&["admin"]))?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, "govern");
    Ok(())
}
