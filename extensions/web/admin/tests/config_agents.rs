//! Reading agent definitions out of the flat `services/agents/*.yaml` files.

#![allow(
    clippy::expect_used,
    reason = "test fixtures assert by panicking, exactly as the tests/ workspace allows"
)]

use std::path::Path;

use systemprompt::identifiers::AgentId;
use systemprompt_web_admin::repositories::config::agents::{find_agent, list_configured_agents};

fn write_agents(root: &Path, file: &str, body: &str) {
    let dir = root.join("agents");
    std::fs::create_dir_all(&dir).expect("create agents dir");
    std::fs::write(dir.join(file), body).expect("write agents yaml");
}

fn write_skill(root: &Path, id: &str, body: &str) {
    let dir = root.join("skills").join(id);
    std::fs::create_dir_all(&dir).expect("create skill dir");
    std::fs::write(dir.join("config.yaml"), body).expect("write skill config");
}

const RICH_AGENT: &str = "agents:\n  auditor:\n    card:\n      displayName: Auditor\n      description: reviews decisions\n    enabled: false\n    is_primary: true\n    show_in_ui: true\n    port: 8123\n    endpoint: http://localhost:8123\n    mcp_servers:\n      - systemprompt\n      - salesforce\n    metadata:\n      systemPrompt: You audit.\n      skills:\n        - review\n        - missing_skill\n";

#[test]
fn absent_agents_directory_yields_empty() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    assert!(list_configured_agents(dir.path())?.is_empty());
    Ok(())
}

#[test]
fn agent_detail_reads_every_declared_field() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "auditor.yaml", RICH_AGENT);
    write_skill(
        dir.path(),
        "review",
        "id: review\nname: Review\ndescription: reviews things\n",
    );

    let agents = list_configured_agents(dir.path())?;
    assert_eq!(agents.len(), 1);
    let a = &agents[0];
    assert_eq!(a.id.as_str(), "auditor");
    assert_eq!(a.name, "Auditor");
    assert_eq!(a.description, "reviews decisions");
    assert!(!a.enabled);
    assert!(a.is_primary);
    assert!(a.show_in_ui);
    assert_eq!(a.system_prompt, "You audit.");
    assert_eq!(a.port, Some(8123));
    assert_eq!(a.endpoint.as_deref(), Some("http://localhost:8123"));
    assert_eq!(a.mcp_servers.len(), 2);
    assert_eq!(a.mcp_servers[0].as_str(), "systemprompt");
    Ok(())
}

#[test]
fn skills_join_the_catalog_and_unknown_ids_drop_out() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "auditor.yaml", RICH_AGENT);
    write_skill(
        dir.path(),
        "review",
        "id: review\nname: Review\ndescription: reviews things\n",
    );

    let agents = list_configured_agents(dir.path())?;
    let skills = &agents[0].skills;
    assert_eq!(skills.len(), 1, "missing_skill has no catalog entry");
    assert_eq!(skills[0].id.as_str(), "review");
    assert_eq!(skills[0].name, "Review");
    assert_eq!(skills[0].description, "reviews things");
    Ok(())
}

#[test]
fn agent_defaults_apply_when_fields_are_absent() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "bare.yaml", "agents:\n  bare: {}\n");
    let agents = list_configured_agents(dir.path())?;
    let a = &agents[0];
    assert_eq!(a.name, "bare", "name falls back to the id");
    assert_eq!(a.description, "");
    assert!(a.enabled, "enabled defaults to true");
    assert!(!a.is_primary);
    assert!(!a.show_in_ui);
    assert_eq!(a.system_prompt, "");
    assert_eq!(a.port, None);
    assert_eq!(a.endpoint, None);
    assert!(a.mcp_servers.is_empty());
    assert!(a.skills.is_empty());
    Ok(())
}

#[test]
fn card_name_is_used_when_display_name_is_absent() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(
        dir.path(),
        "a.yaml",
        "agents:\n  fallback:\n    card:\n      name: Fallback Name\n",
    );
    let agents = list_configured_agents(dir.path())?;
    assert_eq!(agents[0].name, "Fallback Name");
    Ok(())
}

#[test]
fn oversized_port_saturates_rather_than_panicking() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "a.yaml", "agents:\n  big:\n    port: 99999\n");
    let agents = list_configured_agents(dir.path())?;
    assert_eq!(agents[0].port, Some(u16::MAX));
    Ok(())
}

#[test]
fn blank_mcp_server_ids_are_dropped() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(
        dir.path(),
        "a.yaml",
        "agents:\n  mixed:\n    mcp_servers:\n      - ''\n      - ok\n      - 42\n",
    );
    let agents = list_configured_agents(dir.path())?;
    assert_eq!(agents[0].mcp_servers.len(), 1);
    assert_eq!(agents[0].mcp_servers[0].as_str(), "ok");
    Ok(())
}

#[test]
fn yml_files_are_read_and_other_extensions_ignored() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "short.yml", "agents:\n  from_yml: {}\n");
    write_agents(dir.path(), "notes.txt", "agents:\n  ignored: {}\n");
    let ids: Vec<String> = list_configured_agents(dir.path())?
        .into_iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    assert_eq!(ids, vec!["from_yml"]);
    Ok(())
}

#[test]
fn agents_sort_by_id_across_files() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "z.yaml", "agents:\n  beta: {}\n");
    write_agents(dir.path(), "a.yaml", "agents:\n  alpha: {}\n  gamma: {}\n");
    let ids: Vec<String> = list_configured_agents(dir.path())?
        .into_iter()
        .map(|a| a.id.as_str().to_owned())
        .collect();
    assert_eq!(ids, vec!["alpha", "beta", "gamma"]);
    Ok(())
}

#[test]
fn find_agent_matches_by_id_and_misses_cleanly() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    write_agents(dir.path(), "a.yaml", "agents:\n  alpha: {}\n  beta: {}\n");
    let found = find_agent(dir.path(), &AgentId::from("beta"))?;
    assert_eq!(found.map(|a| a.id.as_str().to_owned()), Some("beta".into()));
    assert!(find_agent(dir.path(), &AgentId::from("nope"))?.is_none());
    Ok(())
}
