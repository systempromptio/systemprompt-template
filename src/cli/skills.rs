use anyhow::{Context, Result};
use clap::Args;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct SkillsArgs {
    #[arg(long, help = "Target directory (defaults to ~/.claude/skills)")]
    target: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct AgentsConfig {
    agents: HashMap<String, AgentConfig>,
}

#[derive(Debug, Deserialize)]
struct AgentConfig {
    metadata: Option<AgentMetadata>,
}

#[derive(Debug, Deserialize)]
struct AgentMetadata {
    skills: Option<Vec<String>>,
}

fn get_skills_source_dir() -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push("crates/services/skills");

    if !path.exists() {
        anyhow::bail!("Skills source directory not found at: {}", path.display());
    }

    Ok(path)
}

fn get_skills_target_dir(custom_target: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(target) = custom_target {
        return Ok(target);
    }

    let mut path = std::env::current_dir()?;
    path.push(".claude");
    path.push("skills");

    Ok(path)
}

fn sync_skill(name: &str, source_dir: &Path, target_dir: &Path) -> Result<()> {
    let source_path = source_dir.join(name);
    let target_path = target_dir.join(name);

    if !source_path.exists() {
        anyhow::bail!("Skill '{}' not found in source directory", name);
    }

    let skill_file = source_path.join("SKILL.md");
    if !skill_file.exists() {
        anyhow::bail!("Invalid skill '{}': SKILL.md not found", name);
    }

    if target_path.exists() {
        fs::remove_dir_all(&target_path)?;
    }

    fs::create_dir_all(&target_path).context(format!(
        "Failed to create target directory: {}",
        target_path.display()
    ))?;

    copy_dir_recursive(&source_path, &target_path)?;

    println!("✅ Synced: {}", name);

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dst_path = dst.join(file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

fn get_agents_config_path() -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push("crates/services/agents/agents.yml");

    if !path.exists() {
        anyhow::bail!("Agents config not found at: {}", path.display());
    }

    Ok(path)
}

fn get_available_skills(source_dir: &Path) -> Result<HashSet<String>> {
    let mut skills = HashSet::new();

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                if let Some(name) = path.file_name() {
                    skills.insert(name.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(skills)
}

fn get_referenced_skills(agents_config_path: &Path) -> Result<HashMap<String, Vec<String>>> {
    let content = fs::read_to_string(agents_config_path).context("Failed to read agents config")?;

    let config: AgentsConfig =
        serde_yaml::from_str(&content).context("Failed to parse agents config")?;

    let mut agent_skills = HashMap::new();

    for (agent_name, agent_config) in config.agents {
        if let Some(metadata) = agent_config.metadata {
            if let Some(skills) = metadata.skills {
                if !skills.is_empty() {
                    agent_skills.insert(agent_name, skills);
                }
            }
        }
    }

    Ok(agent_skills)
}

fn validate_agent_skills(
    available_skills: &HashSet<String>,
    agent_skills: &HashMap<String, Vec<String>>,
) -> Result<()> {
    let mut errors = Vec::new();
    let mut has_errors = false;

    for (agent_name, skills) in agent_skills {
        for skill in skills {
            if !available_skills.contains(skill) {
                errors.push(format!(
                    "Agent '{}' references undefined skill: '{}'",
                    agent_name, skill
                ));
                has_errors = true;
            }
        }
    }

    if has_errors {
        eprintln!("\n❌ Skill validation errors:\n");
        for error in &errors {
            eprintln!("   {}", error);
        }
        eprintln!("\n💡 Available skills:");
        let mut sorted_skills: Vec<_> = available_skills.iter().collect();
        sorted_skills.sort();
        for skill in sorted_skills {
            eprintln!("   - {}", skill);
        }
        anyhow::bail!("Found {} undefined skill reference(s)", errors.len());
    }

    Ok(())
}

pub async fn execute(args: SkillsArgs) -> Result<()> {
    let source_dir = get_skills_source_dir()?;
    let target_dir = get_skills_target_dir(args.target)?;

    println!("🔍 Validating agent skill references...\n");

    let available_skills = get_available_skills(&source_dir)?;

    if available_skills.is_empty() {
        println!("No skills found to sync");
        return Ok(());
    }

    let agents_config_path = get_agents_config_path()?;
    let agent_skills = get_referenced_skills(&agents_config_path)?;

    validate_agent_skills(&available_skills, &agent_skills)?;

    println!("✅ All agent skill references are valid\n");

    fs::create_dir_all(&target_dir).context(format!(
        "Failed to create skills directory: {}",
        target_dir.display()
    ))?;

    let mut skills: Vec<String> = available_skills.into_iter().collect();
    skills.sort();

    println!("🔄 Syncing {} skills to Claude Code...\n", skills.len());

    let mut synced = 0;
    let mut failed = 0;

    for skill in skills {
        match sync_skill(&skill, &source_dir, &target_dir) {
            Ok(_) => synced += 1,
            Err(e) => {
                eprintln!("❌ Failed to sync {}: {}", skill, e);
                failed += 1;
            },
        }
    }

    println!("\n📊 Summary:");
    println!("   Synced: {}", synced);
    if failed > 0 {
        println!("   Failed: {}", failed);
    }

    println!("\n📍 Skills location: {}", target_dir.display());
    println!("💡 Skills are now available in Claude Code");

    Ok(())
}
