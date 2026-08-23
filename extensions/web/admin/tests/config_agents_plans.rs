#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use std::path::Path;

use systemprompt::identifiers::AgentId;
use systemprompt_web_admin::repositories::config::agents::{find_agent, list_configured_agents};
use systemprompt_web_admin::repositories::config::plan_yaml_types::{PlanLoadReport, PlansDoc};

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

#[test]
fn plans_doc_defaults_to_empty() -> anyhow::Result<()> {
    let doc: PlansDoc = serde_yaml::from_str("{}")?;
    assert!(doc.plans.is_empty());
    assert!(doc.organizations.is_empty());
    Ok(())
}

#[test]
fn plan_optional_fields_default_and_grant_access_defaults_to_allow() -> anyhow::Result<()> {
    let doc: PlansDoc = serde_yaml::from_str(
        "plans:\n  - id: free\n    name: Free\n    grants:\n      - entity_type: gateway_route\n        entity_id: claude-star\n",
    )?;
    let plan = &doc.plans[0];
    assert_eq!(plan.description, "");
    assert_eq!(plan.seat_limit, None);
    assert_eq!(plan.monthly_cost_cap_usd, None);
    assert_eq!(plan.monthly_cost_warn_usd, None);
    assert_eq!(plan.monthly_price_usd, None);
    assert_eq!(plan.grants.len(), 1);
    assert_eq!(plan.grants[0].access, "allow");
    Ok(())
}

#[test]
fn explicit_plan_values_are_preserved() -> anyhow::Result<()> {
    let doc: PlansDoc = serde_yaml::from_str(
        "plans:\n  - id: pro\n    name: Pro\n    description: paid\n    seat_limit: 25\n    monthly_cost_cap_usd: 500.5\n    monthly_price_usd: 1200.0\n    grants:\n      - entity_type: mcp_server\n        entity_id: salesforce\n        access: deny\n",
    )?;
    let plan = &doc.plans[0];
    assert_eq!(plan.seat_limit, Some(25));
    assert!((plan.monthly_cost_cap_usd.expect("cap") - 500.5).abs() < f64::EPSILON);
    assert!((plan.monthly_price_usd.expect("price") - 1200.0).abs() < f64::EPSILON);
    assert_eq!(plan.grants[0].access, "deny");
    Ok(())
}

#[test]
fn plan_warn_threshold_is_read() -> anyhow::Result<()> {
    let doc: PlansDoc = serde_yaml::from_str(
        "plans:\n  - id: pro\n    name: Pro\n    monthly_cost_cap_usd: 500.0\n    monthly_cost_warn_usd: 400.0\n",
    )?;
    let plan = &doc.plans[0];
    assert!((plan.monthly_cost_warn_usd.expect("warn") - 400.0).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn organization_defaults_status_active_and_non_platform() -> anyhow::Result<()> {
    let doc: PlansDoc =
        serde_yaml::from_str("organizations:\n  - slug: acme\n    name: Acme\n    plan: pro\n")?;
    let org = &doc.organizations[0];
    assert_eq!(org.status, "active");
    assert!(!org.platform);
    assert_eq!(org.seat_limit_override, None);
    assert!(org.email_domains.is_empty());
    Ok(())
}

#[test]
fn organization_overrides_are_read() -> anyhow::Result<()> {
    let doc: PlansDoc = serde_yaml::from_str(
        "organizations:\n  - slug: house\n    name: House\n    plan: internal\n    seat_limit_override: 3\n    email_domains:\n      - astounddigital.com\n    status: suspended\n    platform: true\n",
    )?;
    let org = &doc.organizations[0];
    assert_eq!(org.seat_limit_override, Some(3));
    assert_eq!(org.email_domains, vec!["astounddigital.com".to_owned()]);
    assert_eq!(org.status, "suspended");
    assert!(org.platform);
    Ok(())
}

#[test]
fn unknown_plan_keys_are_rejected() {
    assert!(
        serde_yaml::from_str::<PlansDoc>("plans:\n  - id: p\n    name: P\n    typo_field: 1\n")
            .is_err()
    );
    assert!(serde_yaml::from_str::<PlansDoc>("unexpected: 1\n").is_err());
}

#[test]
fn missing_required_plan_fields_are_rejected() {
    assert!(serde_yaml::from_str::<PlansDoc>("plans:\n  - name: no id\n").is_err());
    assert!(
        serde_yaml::from_str::<PlansDoc>("organizations:\n  - slug: a\n    name: A\n").is_err(),
        "plan is required on an organization"
    );
}

#[test]
fn plan_load_report_starts_at_zero() {
    let report = PlanLoadReport::default();
    assert_eq!(report.plans_upserted, 0);
    assert_eq!(report.organizations_upserted, 0);
    assert_eq!(report.grants_projected, 0);
}
