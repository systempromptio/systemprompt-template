mod categories_capabilities;
mod categories_platform;
mod meta;

use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use super::config::{DemoCategory, DemoPillar, DemoStep, DemosConfig, QuickStartStep};
use categories_capabilities::CAPABILITY_CATEGORIES;
use categories_platform::PLATFORM_CATEGORIES;
use meta::{CategoryMeta, PillarMeta};

const CLI_PREFIXES: &[&str] = &[
    "run_cli_indented ",
    "run_cli_head ",
    "run_cli ",
    "\"$CLI\" ",
];

const DEFAULT_TITLE: &str = "Run the Platform";
const DEFAULT_SUBTITLE: &str = "Nine guided walkthroughs, each a sequential story you can run against your local instance. \
     Every step is a real shell script; every command is copy-paste ready.";

const PILLARS: &[PillarMeta] = &[
    PillarMeta {
        id: "infrastructure",
        title: "Infrastructure",
        subtitle: "What It Is \u{00b7} How You Run It \u{00b7} Why You Can Trust It",
        feature_url: "https://systemprompt.io/features/self-hosted-ai-platform",
        category_ids: &["infrastructure"],
    },
    PillarMeta {
        id: "capabilities",
        title: "Capabilities",
        subtitle: "What It Does \u{00b7} How It Protects You \u{00b7} Why It Passes Audit",
        feature_url: "https://systemprompt.io/features/governance-pipeline",
        category_ids: &["governance", "agents", "mcp", "analytics", "users"],
    },
    PillarMeta {
        id: "integrations",
        title: "Integrations",
        subtitle: "What It Connects To \u{00b7} How You Use It \u{00b7} Why It Scales",
        feature_url: "https://systemprompt.io/features/any-ai-agent",
        category_ids: &["skills", "web", "performance"],
    },
];

/// Errors raised while scanning the `demo/` tree for the homepage showcase.
#[derive(Debug, Error)]
pub enum DemoScanError {
    #[error("demo root not found: {0}")]
    RootMissing(String),

    #[error("read {path}: {source}")]
    ReadManifest {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("parse {path}: {source}")]
    ParseManifest {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
}

#[derive(Debug, Deserialize)]
struct CategoryManifest {
    steps: Vec<ManifestStep>,
}

#[derive(Debug, Deserialize)]
struct ManifestStep {
    script: String,
    label: String,
    narrative: String,
    outcome: String,
}

pub fn scan_demos(demo_root: &Path) -> Result<DemosConfig, DemoScanError> {
    if !demo_root.is_dir() {
        return Err(DemoScanError::RootMissing(demo_root.display().to_string()));
    }

    let quick_start = scan_quick_start(demo_root);

    let mut category_map: Vec<(String, DemoCategory)> = Vec::new();
    for meta in CAPABILITY_CATEGORIES
        .iter()
        .chain(PLATFORM_CATEGORIES.iter())
    {
        let dir = demo_root.join(meta.id);
        if !dir.is_dir() {
            tracing::warn!(
                category = meta.id,
                path = %dir.display(),
                "demo_scanner: skipping category — directory missing"
            );
            continue;
        }
        let steps = match build_category_steps(&dir, meta) {
            Ok(steps) => steps,
            Err(err) => {
                tracing::warn!(
                    category = meta.id,
                    path = %dir.display(),
                    error = %err,
                    "demo_scanner: skipping category — manifest invalid"
                );
                continue;
            },
        };
        if steps.is_empty() {
            continue;
        }
        category_map.push((
            meta.id.to_owned(),
            DemoCategory {
                id: meta.id.to_owned(),
                title: meta.title.to_owned(),
                tagline: meta.tagline.to_owned(),
                story: meta.story.to_owned(),
                cost: meta.cost.to_owned(),
                feature_url: meta.feature_url.to_owned(),
                steps,
            },
        ));
    }

    let mut pillars = Vec::new();
    for pillar in PILLARS {
        let categories: Vec<DemoCategory> = pillar
            .category_ids
            .iter()
            .filter_map(|id| {
                category_map
                    .iter()
                    .find(|(cid, _)| cid == id)
                    .map(|(_, cat)| cat.clone())
            })
            .collect();
        if categories.is_empty() {
            continue;
        }
        pillars.push(DemoPillar {
            id: pillar.id.to_owned(),
            title: pillar.title.to_owned(),
            subtitle: pillar.subtitle.to_owned(),
            feature_url: pillar.feature_url.to_owned(),
            categories,
        });
    }

    Ok(DemosConfig {
        title: Some(DEFAULT_TITLE.to_owned()),
        subtitle: Some(DEFAULT_SUBTITLE.to_owned()),
        quick_start,
        pillars,
    })
}

fn scan_quick_start(demo_root: &Path) -> Vec<QuickStartStep> {
    let mut steps = vec![
        QuickStartStep {
            label: "Build".to_owned(),
            command: "just build".to_owned(),
            description: Some("Compile the Rust workspace into a single binary.".to_owned()),
        },
        QuickStartStep {
            label: "Seed local profile + Postgres".to_owned(),
            command: "just setup-local <anthropic_key>".to_owned(),
            description: Some(
                "Create an eval profile, start the Docker Postgres container, and run the publish pipeline. Pass whichever provider keys you have — Anthropic, OpenAI, or Gemini.".to_owned(),
            ),
        },
        QuickStartStep {
            label: "Start services".to_owned(),
            command: "just start".to_owned(),
            description: Some(
                "Launch every service on localhost:8080 — dashboard, admin panel, governance pipeline.".to_owned(),
            ),
        },
    ];

    if demo_root.join("governance/01-happy-path.sh").is_file() {
        steps.push(QuickStartStep {
            label: "First governance trace".to_owned(),
            command: "./demo/governance/01-happy-path.sh".to_owned(),
            description: Some(
                "Fire a PreToolUse hook, watch governance return allow, and land a row in governance_decisions.".to_owned(),
            ),
        });
    } else if demo_root.join("00-preflight.sh").is_file() {
        steps.push(QuickStartStep {
            label: "Preflight".to_owned(),
            command: "./demo/00-preflight.sh".to_owned(),
            description: Some(
                "Health-check services, create an admin session, and fetch a token.".to_owned(),
            ),
        });
    }

    steps
}

fn build_category_steps(dir: &Path, meta: &CategoryMeta) -> Result<Vec<DemoStep>, DemoScanError> {
    let manifest_path = dir.join("manifest.yaml");
    if !manifest_path.is_file() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&manifest_path).map_err(|source| DemoScanError::ReadManifest {
        path: manifest_path.display().to_string(),
        source,
    })?;
    let manifest: CategoryManifest =
        serde_yaml::from_str(&raw).map_err(|source| DemoScanError::ParseManifest {
            path: manifest_path.display().to_string(),
            source,
        })?;

    let mut out = Vec::with_capacity(manifest.steps.len());
    for step in manifest.steps {
        let script_path = dir.join(&step.script);
        let Ok(content) = fs::read_to_string(&script_path) else {
            tracing::warn!(
                category = meta.id,
                script = %step.script,
                path = %script_path.display(),
                "demo_scanner: manifest references missing script — fix the .sh file or remove the \
                 entry from manifest.yaml"
            );
            continue;
        };
        let commands = extract_commands(&content);
        out.push(DemoStep {
            path: format!("demo/{}/{}", meta.id, step.script),
            name: step.script,
            label: step.label,
            narrative: step.narrative,
            outcome: step.outcome,
            commands,
        });
    }
    Ok(out)
}

fn extract_commands(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let cmd = CLI_PREFIXES
            .iter()
            .find_map(|prefix| {
                trimmed.strip_prefix(prefix).map(|args| {
                    // `run_cli_head` takes a leading numeric line-limit before
                    // the real subcommand; drop it so the rendered command is
                    // the actual CLI invocation, not `systemprompt 40 …`.
                    let trimmed_args = args.trim();
                    let cleaned = if *prefix == "run_cli_head " {
                        trimmed_args
                            .split_once(char::is_whitespace)
                            .filter(|(head, _)| head.chars().all(|c| c.is_ascii_digit()))
                            .map_or(trimmed_args, |(_, rest)| rest.trim_start())
                    } else {
                        trimmed_args
                    };
                    format!("systemprompt {cleaned}")
                })
            })
            .or_else(|| {
                trimmed
                    .starts_with("systemprompt ")
                    .then(|| trimmed.to_owned())
            });
        if let Some(c) = cmd {
            let cleaned = c.trim_end_matches(['\\']).trim().to_owned();
            if !cleaned.is_empty() && !out.contains(&cleaned) {
                out.push(cleaned);
            }
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}
