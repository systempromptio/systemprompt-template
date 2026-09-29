//! Agent configuration is a filesystem boundary consumed by the resources,
//! access-control and bridge-profile pages. A malformed neighbouring file
//! must not hide healthy agents, while ids that cannot be trusted by the
//! bridge must never escape the loader.

use std::path::Path;

use systemprompt::identifiers::AgentId;
use systemprompt_web_admin::repositories::config::agents::{find_agent, list_configured_agents};

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture parent")).expect("create fixture parent");
    std::fs::write(path, body).expect("write fixture");
}

#[test]
fn absent_agents_are_an_empty_catalog_and_cannot_be_found() {
    let root = tempfile::tempdir().expect("temp services tree");

    assert!(
        list_configured_agents(root.path())
            .expect("absent directory is allowed")
            .is_empty()
    );
    assert!(
        find_agent(root.path(), &AgentId::new("missing"))
            .expect("absent directory is allowed")
            .is_none()
    );
}

#[test]
fn agent_loader_keeps_valid_entries_when_neighbouring_files_are_malformed() {
    let root = tempfile::tempdir().expect("temp services tree");
    write(root.path(), "agents/broken.yaml", "agents: [unclosed\n");
    write(root.path(), "agents/readme.txt", "agents:\n  ignored: {}\n");
    write(root.path(), "agents/other.yaml", "not_agents: true\n");
    write(
        root.path(),
        "agents/team.yml",
        "agents:\n  zebra:\n    card:\n      name: Zebra fallback\n  alpha:\n    card:\n      displayName: Alpha visible name\n",
    );

    let agents = list_configured_agents(root.path()).expect("directory reads");
    let ids: Vec<&str> = agents.iter().map(|agent| agent.id.as_str()).collect();
    assert_eq!(
        ids,
        ["alpha", "zebra"],
        "valid YAML and YML entries sort by id"
    );
    assert_eq!(agents[0].name, "Alpha visible name");
    assert_eq!(agents[1].name, "Zebra fallback");
}

#[test]
fn agent_loader_joins_declared_skills_and_discards_invalid_mcp_ids() {
    let root = tempfile::tempdir().expect("temp services tree");
    write(
        root.path(),
        "skills/incident/config.yaml",
        "id: incident\nname: Incident response\ndescription: Triage production incidents.\n",
    );
    write(
        root.path(),
        "skills/release/config.yaml",
        "id: release\nname: Release management\ndescription: Prepare a release.\n",
    );
    write(
        root.path(),
        "agents/ops.yaml",
        "agents:\n  operator:\n    card:\n      displayName: Operations operator\n      description: Handles incidents.\n    enabled: false\n    is_primary: true\n    show_in_ui: true\n    port: 99999\n    endpoint: http://127.0.0.1:9000\n    mcp_servers: [deployment, '']\n    metadata:\n      systemPrompt: Keep systems available.\n      skills: [incident, unknown, release]\n",
    );

    let agent = find_agent(root.path(), &AgentId::new("operator"))
        .expect("directory reads")
        .expect("declared agent");
    assert_eq!(agent.name, "Operations operator");
    assert_eq!(agent.description, "Handles incidents.");
    assert!(!agent.enabled);
    assert!(agent.is_primary && agent.show_in_ui);
    assert_eq!(agent.port, Some(u16::MAX), "out-of-range ports cannot wrap");
    assert_eq!(agent.endpoint.as_deref(), Some("http://127.0.0.1:9000"));
    assert_eq!(agent.system_prompt, "Keep systems available.");
    assert_eq!(
        agent
            .mcp_servers
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["deployment"],
        "empty MCP ids must not reach the bridge profile"
    );
    assert_eq!(
        agent
            .skills
            .iter()
            .map(|skill| skill.id.as_str())
            .collect::<Vec<_>>(),
        ["incident", "release"],
        "unknown skill ids are not presented as installed metadata"
    );
    assert_eq!(agent.skills[0].name, "Incident response");
}
