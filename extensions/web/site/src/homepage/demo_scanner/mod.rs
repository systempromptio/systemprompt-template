mod categories_capabilities;
mod categories_platform;
mod meta;

use std::fs;
use std::path::Path;

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
const DEFAULT_SUBTITLE: &str =
    "Ten guided walkthroughs, each a sequential story you can run against your local instance. \
     Every step is a real shell script; every command is copy-paste ready.";

const PILLARS: &[PillarMeta] = &[
    PillarMeta {
        id: "infrastructure",
        title: "Infrastructure",
        subtitle: "What It Is \u{00b7} How You Run It \u{00b7} Why You Can Trust It",
        feature_url: "https://systemprompt.io/features/self-hosted-ai-platform",
        category_ids: &["infrastructure", "cloud"],
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

pub fn scan_demos(demo_root: &Path) -> anyhow::Result<DemosConfig> {
    if !demo_root.is_dir() {
        anyhow::bail!("demo root not found: {}", demo_root.display());
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
        let steps = build_category_steps(&dir, meta);
        if steps.is_empty() {
            continue;
        }
        category_map.push((
            meta.id.to_string(),
            DemoCategory {
                id: meta.id.to_string(),
                title: meta.title.to_string(),
                tagline: meta.tagline.to_string(),
                story: meta.story.to_string(),
                cost: meta.cost.to_string(),
                feature_url: meta.feature_url.to_string(),
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
            id: pillar.id.to_string(),
            title: pillar.title.to_string(),
            subtitle: pillar.subtitle.to_string(),
            feature_url: pillar.feature_url.to_string(),
            categories,
        });
    }

    Ok(DemosConfig {
        title: Some(DEFAULT_TITLE.to_string()),
        subtitle: Some(DEFAULT_SUBTITLE.to_string()),
        quick_start,
        pillars,
    })
}

fn scan_quick_start(demo_root: &Path) -> Vec<QuickStartStep> {
    let mut steps = vec![
        QuickStartStep {
            label: "Build".to_string(),
            command: "just build".to_string(),
            description: Some("Compile the Rust workspace into a single binary.".to_string()),
        },
        QuickStartStep {
            label: "Seed local profile + Postgres".to_string(),
            command: "just setup-local <anthropic_key>".to_string(),
            description: Some(
                "Create an eval profile, start the Docker Postgres container, and run the publish pipeline. Pass whichever provider keys you have — Anthropic, OpenAI, or Gemini."
                    .to_string(),
            ),
        },
        QuickStartStep {
            label: "Start services".to_string(),
            command: "just start".to_string(),
            description: Some(
                "Launch every service on localhost:8080 — dashboard, admin panel, governance pipeline."
                    .to_string(),
            ),
        },
    ];

    if demo_root.join("governance/01-happy-path.sh").is_file() {
        steps.push(QuickStartStep {
            label: "First governance trace".to_string(),
            command: "./demo/governance/01-happy-path.sh".to_string(),
            description: Some(
                "Fire a PreToolUse hook, watch governance return allow, and land a row in governance_decisions."
                    .to_string(),
            ),
        });
    } else if demo_root.join("00-preflight.sh").is_file() {
        steps.push(QuickStartStep {
            label: "Preflight".to_string(),
            command: "./demo/00-preflight.sh".to_string(),
            description: Some(
                "Health-check services, create an admin session, and fetch a token.".to_string(),
            ),
        });
    }

    steps
}

fn build_category_steps(dir: &Path, meta: &CategoryMeta) -> Vec<DemoStep> {
    let mut out = Vec::new();
    for step in meta.steps {
        let path = dir.join(step.script);
        let Ok(content) = fs::read_to_string(&path) else {
            tracing::warn!(
                category = meta.id,
                script = step.script,
                "demo_scanner: missing script — skipping step"
            );
            continue;
        };
        let commands = extract_commands(&content);
        out.push(DemoStep {
            path: format!("demo/{}/{}", meta.id, step.script),
            name: step.script.to_string(),
            label: step.label.to_string(),
            narrative: step.narrative.to_string(),
            outcome: step.outcome.to_string(),
            commands,
        });
    }
    out
}

fn extract_commands(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let cmd = CLI_PREFIXES
            .iter()
            .find_map(|prefix| {
                trimmed
                    .strip_prefix(prefix)
                    .map(|args| format!("systemprompt {}", args.trim()))
            })
            .or_else(|| {
                trimmed
                    .starts_with("systemprompt ")
                    .then(|| trimmed.to_string())
            });
        if let Some(c) = cmd {
            let cleaned = c.trim_end_matches(['\\']).trim().to_string();
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
