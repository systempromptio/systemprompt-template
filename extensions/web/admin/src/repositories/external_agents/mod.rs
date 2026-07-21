//! External-agent (super-agent) catalog repository.
//!
//! Reads `services/external_agents/*.yaml` from disk and exposes a flat list
//! consumable by the admin SSR handler. Distinct from `agents` (A2A agents);
//! the `id` field MUST equal the bridge `HostApp::id()` for the same host.

use std::path::{Path, PathBuf};

use serde::Deserialize;

const EXTERNAL_AGENTS_DIR: &str = "services/external_agents";

#[derive(Debug, Clone)]
pub struct ExternalAgentRow {
    pub id: String,
    pub display_name: String,
    pub kind: String,
    pub enabled: bool,
    pub description: String,
    pub platforms: Vec<String>,
    pub docs_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiskEntry {
    id: String,
    display_name: String,
    kind: String,
    enabled: bool,
    #[serde(default)]
    description: String,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default)]
    docs_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiskFile {
    #[serde(default)]
    external_agents: std::collections::BTreeMap<String, DiskEntry>,
}

// Live upstream in systemprompt-template via the ssr_governance
// handlers, which this fork does not ship. Kept so the shared
// repository files stay identical across both trees.
// lint-ok: unused-pub
pub fn list_external_agents() -> Vec<ExternalAgentRow> {
    let dir = resolve_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        tracing::warn!(path = %dir.display(), "external_agents directory not readable");
        return Vec::new();
    };

    let mut rows: Vec<ExternalAgentRow> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let parsed: DiskFile = match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "failed to parse external_agents yaml");
                continue;
            },
        };
        for (_key, e) in parsed.external_agents {
            rows.push(ExternalAgentRow {
                id: e.id,
                display_name: e.display_name,
                kind: e.kind,
                enabled: e.enabled,
                description: e.description,
                platforms: e.platforms,
                docs_url: e.docs_url,
            });
        }
    }

    rows.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    rows
}

fn resolve_dir() -> PathBuf {
    let primary = Path::new(EXTERNAL_AGENTS_DIR);
    if primary.exists() {
        return primary.to_path_buf();
    }
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join(EXTERNAL_AGENTS_DIR);
        if candidate.exists() {
            return candidate;
        }
    }
    primary.to_path_buf()
}
