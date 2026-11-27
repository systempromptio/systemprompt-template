use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use harness_config::Skill;
use harness_desktop::views::governance::feeds::{AppliedView, McpState, held};
use harness_mcp::{ServerStatus, StatusMap};
use serde_json::Map;

fn skill(name: &str, dir: &Path, description: &str) -> Skill {
    Skill {
        name: name.to_owned(),
        description: description.to_owned(),
        location: dir.join("SKILL.md"),
        dir: dir.to_path_buf(),
        body: String::new(),
        extra: Map::new(),
    }
}

#[test]
fn only_skills_under_the_managed_root_are_held() {
    let root = PathBuf::from("/config/skills");
    let skills = vec![
        skill("zeta", &root.join("zeta"), "  "),
        skill("alpha", &root.join("alpha"), "Writes docs"),
        skill("mine", Path::new("/project/.harness/skills/mine"), "Local"),
    ];
    let applied = AppliedView {
        instance_url: "https://a.example.com".to_owned(),
        mcp: Vec::new(),
        rules: 2,
        governed: true,
    };
    let held = held(applied, &skills, &root, &BTreeMap::new());
    let names: Vec<&str> = held
        .skills
        .iter()
        .map(|skill| skill.name.as_str())
        .collect();
    assert_eq!(names, vec!["alpha", "zeta"]);
    assert_eq!(held.skills[0].description.as_deref(), Some("Writes docs"));
    assert_eq!(held.skills[1].description, None);
    assert_eq!(held.rules, 2);
}

#[test]
fn servers_take_their_live_status_or_read_as_not_started() {
    let mut statuses = StatusMap::new();
    statuses.insert("docs".to_owned(), ServerStatus::Connected { tools: 4 });
    statuses.insert(
        "crm".to_owned(),
        ServerStatus::Failed {
            error: "boom".to_owned(),
        },
    );
    let applied = AppliedView {
        instance_url: String::new(),
        mcp: vec!["docs".to_owned(), "crm".to_owned(), "new".to_owned()],
        rules: 0,
        governed: false,
    };
    let held = held(applied, &[], Path::new("/config/skills"), &statuses);
    let states: Vec<McpState> = held.servers.iter().map(|server| server.state).collect();
    assert_eq!(
        states,
        vec![
            McpState::Connected { tools: 4 },
            McpState::Failed,
            McpState::NotStarted
        ]
    );
}
