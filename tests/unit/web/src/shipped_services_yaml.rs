//! Every YAML the repository ships under `services/` that declares a
//! `ServicesConfig` section must deserialise into it.
//!
//! `setup-local` validates the whole `services/` tree before it will write a
//! profile, and the config structs are `deny_unknown_fields`. A shipped file
//! naming an option core does not implement therefore fails setup outright for
//! anyone starting from a clean clone -- which is how `slack/example.yaml` came
//! to document a `link_by_workspace_email` flag no code ever read, breaking
//! first-run setup. Nothing else in the suite reads these files, because
//! nothing else runs setup.
//!
//! Files under `services/` that carry a different schema (page templates, for
//! instance) declare none of these keys and are out of scope.

use std::fs;
use std::path::{Path, PathBuf};

use systemprompt::models::services::ServicesConfig;

const SECTIONS: [&str; 14] = [
    "includes",
    "settings",
    "agents",
    "mcp_servers",
    "scheduler",
    "ai",
    "web",
    "plugins",
    "marketplaces",
    "skills",
    "external_agents",
    "slack_apps",
    "teams_apps",
    "bridge_policy",
];

fn services_root() -> PathBuf {
    let mut dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    loop {
        if dir.join("services").is_dir() && dir.join("justfile").is_file() {
            return dir.join("services");
        }
        assert!(
            dir.pop(),
            "no repository root with a services/ tree above the crate"
        );
    }
}

fn yaml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            yaml_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            out.push(path);
        }
    }
}

fn declares_a_section(text: &str) -> bool {
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .any(|(key, _)| !key.starts_with([' ', '\t', '#']) && SECTIONS.contains(&key.trim()))
}

#[test]
fn every_shipped_services_yaml_deserialises() {
    let root = services_root();
    let mut files = Vec::new();
    yaml_files(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no YAML found under {}", root.display());

    let mut checked = 0_usize;
    let mut failures = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).expect("read shipped services yaml");
        if !declares_a_section(&text) {
            continue;
        }
        checked += 1;
        if let Err(e) = serde_yaml::from_str::<ServicesConfig>(&text) {
            failures.push(format!("{}: {e}", path.display()));
        }
    }

    assert!(
        checked > 0,
        "no shipped YAML declared a ServicesConfig section -- the key list is stale"
    );
    assert!(
        failures.is_empty(),
        "shipped services YAML that setup-local would reject:\n  {}",
        failures.join("\n  ")
    );
}
