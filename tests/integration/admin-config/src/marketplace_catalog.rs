//! `repositories::marketplace` — the filesystem halves that walk a
//! `services/` tree. No database is involved, but these sit in the same
//! repository layer and share its naming contract.

use std::path::Path;

use systemprompt_web_admin::repositories::marketplace::hooks::list_configured_hooks;
use systemprompt_web_admin::repositories::marketplace::plugins::{
    list_agent_catalog, list_skill_catalog,
};

use crate::fixtures::write_services_file;

fn write_skill(dir: &Path, id: &str, body: &str) {
    write_services_file(dir, &format!("skills/{id}/config.yaml"), body);
}

fn write_hook(dir: &Path, name: &str, body: &str) {
    write_services_file(dir, &format!("hooks/{name}/config.yaml"), body);
}

#[test]
fn list_skill_catalog_is_empty_when_there_is_no_skills_directory() {
    let dir = tempfile::tempdir().expect("temp services dir");

    let entries = list_skill_catalog(dir.path()).expect("walk an empty tree");

    assert!(entries.is_empty());
}

#[test]
fn list_skill_catalog_reads_metadata_and_sorts_by_id() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_skill(
        dir.path(),
        "zeta",
        "id: zeta\nname: Zeta\ndescription: Last\nenabled: false\n",
    );
    write_skill(
        dir.path(),
        "alpha",
        "id: alpha\nname: Alpha\ndescription: First\n",
    );

    let entries = list_skill_catalog(dir.path()).expect("walk skills");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id.as_str(), "alpha");
    assert_eq!(entries[0].name, "Alpha");
    assert!(entries[0].enabled, "a skill is enabled unless it says not");
    assert!(!entries[1].enabled);
    assert_eq!(entries[0].source_path, "services/skills/alpha/config.yaml");
}

#[test]
fn list_skill_catalog_falls_back_to_the_directory_name() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_skill(dir.path(), "no_id", "description: Nameless\n");

    let entries = list_skill_catalog(dir.path()).expect("walk skills");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id.as_str(), "no_id");
    assert_eq!(entries[0].name, "no_id", "the id stands in for a name");
}

#[test]
fn list_skill_catalog_skips_directories_without_a_config_and_invalid_yaml() {
    let dir = tempfile::tempdir().expect("temp services dir");
    std::fs::create_dir_all(dir.path().join("skills/bare")).expect("create bare skill dir");
    write_skill(dir.path(), "broken", "id: [unclosed\n");
    write_skill(dir.path(), "good", "id: good\n");

    let entries = list_skill_catalog(dir.path()).expect("walk skills");

    assert_eq!(entries.len(), 1, "one unreadable skill is not fatal");
    assert_eq!(entries[0].id.as_str(), "good");
}

#[test]
fn list_agent_catalog_is_empty_when_there_is_no_agents_directory() {
    let dir = tempfile::tempdir().expect("temp services dir");

    let entries = list_agent_catalog(dir.path()).expect("walk an empty tree");

    assert!(entries.is_empty());
}

#[test]
fn list_agent_catalog_flattens_every_agent_in_every_file() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(
        dir.path(),
        "agents/pair.yaml",
        "agents:\n  zeta:\n    card:\n      displayName: Zeta\n      description: The last one\n  alpha:\n    card:\n      description: The first one\n    enabled: false\n",
    );

    let entries = list_agent_catalog(dir.path()).expect("walk agents");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id.as_str(), "alpha");
    assert_eq!(entries[0].description, "The first one");
    assert!(!entries[0].enabled);
    assert_eq!(entries[1].name, "Zeta");
    assert_eq!(entries[1].source_path, "services/agents/pair.yaml");
}

#[test]
fn list_agent_catalog_ignores_files_without_an_agents_map() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_services_file(dir.path(), "agents/notes.yaml", "unrelated: true\n");
    write_services_file(dir.path(), "agents/readme.md", "agents:\n  ignored: {}\n");

    let entries = list_agent_catalog(dir.path()).expect("walk agents");

    assert!(
        entries.is_empty(),
        "only .yaml files carrying an agents map count"
    );
}

#[test]
fn list_configured_hooks_is_empty_without_a_hooks_directory() {
    let dir = tempfile::tempdir().expect("temp services dir");

    let hooks = list_configured_hooks(dir.path(), &["admin".to_owned()]).expect("walk hooks");

    assert!(hooks.is_empty());
}

#[test]
fn list_configured_hooks_reads_the_binding_a_plugin_declares() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_hook(
        dir.path(),
        "track",
        "id: track\nevent: PostToolUse\nmatcher: Bash\ncommand: /bin/true\nasync: true\n",
    );

    let hooks = list_configured_hooks(dir.path(), &["user".to_owned()]).expect("walk hooks");

    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, "track");
    assert_eq!(hooks[0].event, "PostToolUse");
    assert_eq!(hooks[0].matcher, "Bash");
    assert_eq!(hooks[0].command, "/bin/true");
    assert!(hooks[0].is_async);
}

#[test]
fn list_configured_hooks_falls_back_to_the_directory_name_for_an_id() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_hook(
        dir.path(),
        "govern",
        "event: PreToolUse\ncommand: /bin/true\n",
    );

    let hooks = list_configured_hooks(dir.path(), &["admin".to_owned()]).expect("walk hooks");

    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, "govern");
    assert_eq!(hooks[0].matcher, "*", "the default matcher is every tool");
}

#[test]
fn list_configured_hooks_hides_a_disabled_hook_from_a_non_admin() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_hook(
        dir.path(),
        "off",
        "event: PreToolUse\ncommand: /bin/true\nenabled: false\n",
    );

    let as_user = list_configured_hooks(dir.path(), &["user".to_owned()]).expect("walk as user");
    let as_admin = list_configured_hooks(dir.path(), &["admin".to_owned()]).expect("walk as admin");

    assert!(as_user.is_empty());
    assert_eq!(as_admin.len(), 1, "an admin sees what is switched off");
}

#[test]
fn list_configured_hooks_honours_the_visible_to_list() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_hook(
        dir.path(),
        "scoped",
        "event: PreToolUse\ncommand: /bin/true\nvisible_to: [auditor]\n",
    );

    let stranger =
        list_configured_hooks(dir.path(), &["user".to_owned()]).expect("walk as a stranger");
    let auditor =
        list_configured_hooks(dir.path(), &["auditor".to_owned()]).expect("walk as an auditor");

    assert!(stranger.is_empty());
    assert_eq!(auditor.len(), 1);
}

#[test]
fn list_configured_hooks_skips_unparseable_and_incomplete_entries() {
    let dir = tempfile::tempdir().expect("temp services dir");
    write_hook(dir.path(), "broken", "event: NotAnEvent\ncommand: x\n");
    std::fs::create_dir_all(dir.path().join("hooks/bare")).expect("create bare hook dir");
    write_hook(dir.path(), "good", "event: Stop\ncommand: /bin/true\n");

    let hooks = list_configured_hooks(dir.path(), &["admin".to_owned()]).expect("walk hooks");

    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0].id, "good");
}
