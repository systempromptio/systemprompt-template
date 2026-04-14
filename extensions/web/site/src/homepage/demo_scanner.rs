use std::fs;
use std::path::{Path, PathBuf};

use super::config::{DemoCategory, DemoScript, DemosConfig, QuickStartStep};

const DEFAULT_TITLE: &str = "Run the Platform";
const DEFAULT_SUBTITLE: &str =
    "Forty-plus runnable shell scripts walk through every capability. Copy any command and run \
     it against your local instance.";

struct CategoryMeta {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    cost: &'static str,
}

const CATEGORIES: &[CategoryMeta] = &[
    CategoryMeta {
        id: "governance",
        title: "Governance",
        description: "Tool access control, scope enforcement, secret detection, audit trails, and hooks.",
        cost: "Free",
    },
    CategoryMeta {
        id: "agents",
        title: "Agents",
        description: "Agent discovery, configuration, messaging, tracing, and the A2A registry.",
        cost: "2 scripts ~$0.01 each",
    },
    CategoryMeta {
        id: "mcp",
        title: "MCP Servers",
        description: "MCP server management, access tracking, and tool execution.",
        cost: "Free",
    },
    CategoryMeta {
        id: "skills",
        title: "Skills & Content",
        description: "Skills, content, files, plugins, and contexts.",
        cost: "Free",
    },
    CategoryMeta {
        id: "infrastructure",
        title: "Infrastructure",
        description: "Services, database, jobs, logs, and configuration.",
        cost: "Free",
    },
    CategoryMeta {
        id: "analytics",
        title: "Analytics",
        description: "Overview, agents, costs, requests, sessions, traffic, conversations, and tools.",
        cost: "Free",
    },
    CategoryMeta {
        id: "users",
        title: "Users & Auth",
        description: "User CRUD, roles, sessions, and IP bans.",
        cost: "Free",
    },
    CategoryMeta {
        id: "web",
        title: "Web Generation",
        description: "Content types, templates, sitemaps, and validation.",
        cost: "Free",
    },
    CategoryMeta {
        id: "cloud",
        title: "Cloud",
        description: "Auth status, profiles, and deployment info.",
        cost: "Free",
    },
    CategoryMeta {
        id: "performance",
        title: "Performance",
        description: "Request tracing, benchmarks, and load testing.",
        cost: "Free",
    },
];

pub fn scan_demos(demo_root: &Path) -> anyhow::Result<DemosConfig> {
    if !demo_root.is_dir() {
        anyhow::bail!("demo root not found: {}", demo_root.display());
    }

    let quick_start = scan_quick_start(demo_root);

    let mut categories = Vec::new();
    for meta in CATEGORIES {
        let dir = demo_root.join(meta.id);
        if !dir.is_dir() {
            continue;
        }
        let scripts = scan_category_scripts(&dir, meta.id)?;
        if scripts.is_empty() {
            continue;
        }
        categories.push(DemoCategory {
            id: meta.id.to_string(),
            title: meta.title.to_string(),
            description: meta.description.to_string(),
            cost: meta.cost.to_string(),
            scripts,
        });
    }

    Ok(DemosConfig {
        title: Some(DEFAULT_TITLE.to_string()),
        subtitle: Some(DEFAULT_SUBTITLE.to_string()),
        quick_start,
        categories,
    })
}

fn scan_quick_start(demo_root: &Path) -> Vec<QuickStartStep> {
    let mut steps = vec![
        QuickStartStep {
            label: "Build".to_string(),
            command: "just build".to_string(),
            description: Some("Compile the single Rust binary.".to_string()),
        },
        QuickStartStep {
            label: "Start".to_string(),
            command: "just start".to_string(),
            description: Some("Start all services on localhost.".to_string()),
        },
    ];

    if demo_root.join("00-preflight.sh").is_file() {
        steps.push(QuickStartStep {
            label: "Preflight".to_string(),
            command: "./demo/00-preflight.sh".to_string(),
            description: Some(
                "Health-check services, create an admin session, and fetch a token.".to_string(),
            ),
        });
    }
    if demo_root.join("01-seed-data.sh").is_file() {
        steps.push(QuickStartStep {
            label: "Seed data".to_string(),
            command: "./demo/01-seed-data.sh".to_string(),
            description: Some(
                "Populate governance decisions, events, skills, and content.".to_string(),
            ),
        });
    }

    steps
}

fn scan_category_scripts(dir: &Path, category_id: &str) -> anyhow::Result<Vec<DemoScript>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|ext| ext == "sh")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| !n.starts_with('_'))
        })
        .collect();
    paths.sort();

    let mut scripts = Vec::new();
    for path in paths {
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let rel_path = format!("demo/{category_id}/{name}");
        let (title, description) = parse_header(&content, &name);
        let commands = extract_commands(&content);
        if commands.is_empty() {
            continue;
        }
        scripts.push(DemoScript {
            path: rel_path,
            name,
            title,
            description,
            commands,
        });
    }
    Ok(scripts)
}

fn parse_header(content: &str, fallback_name: &str) -> (String, String) {
    let mut title: Option<String> = None;
    let mut description_lines: Vec<String> = Vec::new();
    let mut in_what_block = false;

    for line in content.lines().take(25) {
        if line.starts_with("#!") {
            continue;
        }
        let Some(rest) = line.strip_prefix('#') else {
            if title.is_some() {
                break;
            }
            continue;
        };
        let trimmed = rest.trim();
        if trimmed.is_empty() {
            if title.is_some() && !description_lines.is_empty() {
                break;
            }
            continue;
        }
        if title.is_none() {
            title = Some(clean_title(trimmed));
            continue;
        }
        if trimmed.eq_ignore_ascii_case("what this does:") {
            in_what_block = true;
            continue;
        }
        if trimmed.starts_with("Cost:") || trimmed.starts_with("Usage:") {
            break;
        }
        if in_what_block {
            let cleaned = trimmed.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ')' || c.is_whitespace());
            if !cleaned.is_empty() {
                description_lines.push(cleaned.to_string());
            }
            if description_lines.len() >= 2 {
                break;
            }
        } else {
            description_lines.push(trimmed.to_string());
            if description_lines.len() >= 2 {
                break;
            }
        }
    }

    let title = title.unwrap_or_else(|| fallback_name.to_string());
    let description = if description_lines.is_empty() {
        String::new()
    } else {
        description_lines.join(" ")
    };
    (title, description)
}

fn clean_title(raw: &str) -> String {
    let without_prefix = raw
        .trim_start_matches("DEMO:")
        .trim_start_matches("DEMO")
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == ':' || c == '.' || c.is_whitespace());
    without_prefix.trim().to_string()
}

fn extract_commands(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(args) = trimmed.strip_prefix("run_cli_indented ") {
            out.push(format!("systemprompt {}", args.trim()));
        } else if let Some(args) = trimmed.strip_prefix("run_cli ") {
            out.push(format!("systemprompt {}", args.trim()));
        } else if trimmed.starts_with("systemprompt ") {
            out.push(trimmed.to_string());
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}
