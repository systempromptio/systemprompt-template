#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use std::path::Path;

use systemprompt_web_admin::repositories::marketplace::plugins::{
    list_agent_catalog, list_skill_catalog,
};

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, body).expect("write fixture");
}

fn skill(root: &Path, dir: &str, body: &str) {
    write(&root.join("skills").join(dir).join("config.yaml"), body);
}

#[test]
fn skill_catalog_is_empty_when_directory_absent() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    assert!(list_skill_catalog(dir.path())?.is_empty());
    Ok(())
}

#[test]
fn skill_catalog_reads_declared_fields() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    skill(
        dir.path(),
        "alpha",
        "id: alpha\nname: Alpha Skill\ndescription: does alpha\nenabled: false\n",
    );
    let out = list_skill_catalog(dir.path())?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id.as_str(), "alpha");
    assert_eq!(out[0].name, "Alpha Skill");
    assert_eq!(out[0].description, "does alpha");
    assert!(!out[0].enabled);
    assert_eq!(out[0].source_path, "services/skills/alpha/config.yaml");
    Ok(())
}

#[test]
fn skill_catalog_defaults_id_to_directory_and_name_to_id() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    skill(dir.path(), "from_dir", "description: no id or name\n");
    let out = list_skill_catalog(dir.path())?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id.as_str(), "from_dir");
    assert_eq!(out[0].name, "from_dir");
    assert!(out[0].enabled, "enabled defaults to true");
    Ok(())
}

#[test]
fn skill_catalog_skips_dirs_without_config_and_loose_files() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join("skills").join("no_config"))?;
    write(&dir.path().join("skills").join("stray.yaml"), "id: stray\n");
    skill(dir.path(), "real", "id: real\n");
    let out = list_skill_catalog(dir.path())?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id.as_str(), "real");
    Ok(())
}

#[test]
fn skill_catalog_skips_unparseable_yaml_but_keeps_the_rest() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    skill(dir.path(), "broken", "id: [unclosed\n");
    skill(dir.path(), "good", "id: good\n");
    let out = list_skill_catalog(dir.path())?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id.as_str(), "good");
    Ok(())
}

#[test]
fn skill_catalog_skips_empty_id() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    skill(dir.path(), "blank", "id: ''\n");
    assert!(list_skill_catalog(dir.path())?.is_empty());
    Ok(())
}

#[test]
fn skill_catalog_sorts_by_id_not_directory_order() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    skill(dir.path(), "one", "id: zebra\n");
    skill(dir.path(), "two", "id: aardvark\n");
    skill(dir.path(), "three", "id: mongoose\n");
    let ids: Vec<String> = list_skill_catalog(dir.path())?
        .into_iter()
        .map(|e| e.id.as_str().to_owned())
        .collect();
    assert_eq!(ids, vec!["aardvark", "mongoose", "zebra"]);
    Ok(())
}

fn agents(root: &Path, file: &str, body: &str) {
    write(&root.join("agents").join(file), body);
}

#[test]
fn agent_catalog_is_empty_when_directory_absent() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    assert!(list_agent_catalog(dir.path())?.is_empty());
    Ok(())
}

#[test]
fn agent_catalog_lists_every_agent_in_a_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    agents(
        dir.path(),
        "team.yaml",
        "agents:\n  scribe:\n    name: Scribe\n    description: writes\n  auditor:\n    name: Auditor\n",
    );
    let out = list_agent_catalog(dir.path())?;
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].id.as_str(), "auditor");
    assert_eq!(out[1].id.as_str(), "scribe");
    assert_eq!(out[1].description, "writes");
    assert_eq!(out[0].source_path, "services/agents/team.yaml");
    Ok(())
}

#[test]
fn agent_name_falls_back_to_card_display_name_then_id() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    agents(
        dir.path(),
        "a.yaml",
        "agents:\n  carded:\n    card:\n      displayName: Carded Agent\n  bare: {}\n",
    );
    let out = list_agent_catalog(dir.path())?;
    let by_id = |id: &str| {
        out.iter()
            .find(|a| a.id.as_str() == id)
            .expect("agent present")
    };
    assert_eq!(by_id("carded").name, "Carded Agent");
    assert_eq!(by_id("bare").name, "bare");
    Ok(())
}

#[test]
fn agent_description_prefers_card_over_top_level() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    agents(
        dir.path(),
        "a.yaml",
        "agents:\n  both:\n    description: top level\n    card:\n      description: from card\n  only_top:\n    description: top only\n  neither: {}\n",
    );
    let out = list_agent_catalog(dir.path())?;
    let desc = |id: &str| {
        out.iter()
            .find(|a| a.id.as_str() == id)
            .expect("agent present")
            .description
            .clone()
    };
    assert_eq!(desc("both"), "from card");
    assert_eq!(desc("only_top"), "top only");
    assert_eq!(desc("neither"), "");
    Ok(())
}

#[test]
fn agent_enabled_defaults_true_and_honours_explicit_false() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    agents(
        dir.path(),
        "a.yaml",
        "agents:\n  on_by_default: {}\n  turned_off:\n    enabled: false\n",
    );
    let out = list_agent_catalog(dir.path())?;
    let enabled = |id: &str| {
        out.iter()
            .find(|a| a.id.as_str() == id)
            .expect("agent present")
            .enabled
    };
    assert!(enabled("on_by_default"));
    assert!(!enabled("turned_off"));
    Ok(())
}

#[test]
fn agent_catalog_ignores_yml_extension_and_files_without_agents_key() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    agents(dir.path(), "ignored.yml", "agents:\n  hidden: {}\n");
    agents(dir.path(), "notes.txt", "agents:\n  hidden2: {}\n");
    agents(dir.path(), "other.yaml", "something_else: true\n");
    agents(dir.path(), "broken.yaml", "agents: [unclosed\n");
    agents(dir.path(), "real.yaml", "agents:\n  kept: {}\n");
    let out = list_agent_catalog(dir.path())?;
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id.as_str(), "kept");
    Ok(())
}

#[test]
fn agent_catalog_sorts_across_files() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    agents(dir.path(), "z.yaml", "agents:\n  alpha: {}\n");
    agents(dir.path(), "a.yaml", "agents:\n  omega: {}\n");
    let ids: Vec<String> = list_agent_catalog(dir.path())?
        .into_iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    assert_eq!(ids, vec!["alpha", "omega"]);
    Ok(())
}
