use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use systemprompt_core_agent::repository::SkillRepository;
use systemprompt_core_database::{DatabaseProvider, DbPool};
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::SkillId;

pub async fn validate_agent_skill_references(db_pool: &DbPool, logger: &LogService) -> Result<()> {
    let agents_path = std::env::var("SYSTEMPROMPT_SERVICES_PATH")
        .map(|p| PathBuf::from(p).join("agents"))
        .unwrap_or_else(|_| PathBuf::from("crates/services/agents"));

    if !agents_path.exists() {
        logger
            .warn(
                "scheduler",
                &format!("Agents path not found: {}", agents_path.display()),
            )
            .await
            .ok();
        return Ok(());
    }

    let db_provider: Arc<dyn DatabaseProvider> = db_pool.clone();
    let skill_repo = SkillRepository::new(db_provider);
    let ingested_skills = skill_repo.list_all().await?;
    let ingested_skill_ids: HashSet<SkillId> =
        ingested_skills.into_iter().map(|s| s.skill_id).collect();

    logger
        .info(
            "scheduler",
            &format!(
                "Found {} ingested skills in database",
                ingested_skill_ids.len()
            ),
        )
        .await
        .ok();

    let mut referenced_skills = HashSet::new();
    let mut missing_skills = Vec::new();

    for entry in std::fs::read_dir(&agents_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yml") {
            let agent_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    process_agent_config(
                        &content,
                        agent_name,
                        &ingested_skill_ids,
                        &mut referenced_skills,
                        &mut missing_skills,
                        logger,
                    )
                    .await;
                },
                Err(e) => {
                    logger
                        .warn(
                            "scheduler",
                            &format!("Failed to read agent config {}: {}", agent_name, e),
                        )
                        .await
                        .ok();
                },
            }
        }
    }

    logger
        .info(
            "scheduler",
            &format!(
                "Found {} skill references in agent configs",
                referenced_skills.len()
            ),
        )
        .await
        .ok();

    if !missing_skills.is_empty() {
        let error_msg = format!(
            "Missing skills referenced by agents:\n{}",
            missing_skills.join("\n")
        );
        println!("\n{}", error_msg);
        return Err(anyhow::anyhow!(error_msg));
    }

    println!(
        "All {} agent skill references validated",
        referenced_skills.len()
    );
    Ok(())
}

async fn process_agent_config(
    content: &str,
    agent_name: &str,
    ingested_skill_ids: &HashSet<SkillId>,
    referenced_skills: &mut HashSet<String>,
    missing_skills: &mut Vec<String>,
    logger: &LogService,
) {
    match serde_yaml::from_str::<serde_yaml::Value>(content) {
        Ok(yaml) => {
            extract_skills_from_yaml(
                &yaml,
                agent_name,
                ingested_skill_ids,
                referenced_skills,
                missing_skills,
            );
        },
        Err(e) => {
            logger
                .warn(
                    "scheduler",
                    &format!("Failed to parse agent config {}: {}", agent_name, e),
                )
                .await
                .ok();
        },
    }
}

fn extract_skills_from_yaml(
    yaml: &serde_yaml::Value,
    agent_name: &str,
    ingested_skill_ids: &HashSet<SkillId>,
    referenced_skills: &mut HashSet<String>,
    missing_skills: &mut Vec<String>,
) {
    let Some(agents) = yaml.get("agents").and_then(|a| a.as_mapping()) else {
        return;
    };

    for (_, agent_config) in agents {
        let Some(card) = agent_config.get("card") else {
            continue;
        };

        let Some(skills) = card.get("skills").and_then(|s| s.as_sequence()) else {
            continue;
        };

        for skill in skills {
            if let Some(skill_id) = skill.get("id").and_then(|id| id.as_str()) {
                referenced_skills.insert(skill_id.to_string());

                if !ingested_skill_ids.contains(&SkillId::new(skill_id)) {
                    missing_skills.push(format!(
                        "Agent '{}' references missing skill '{}'",
                        agent_name, skill_id
                    ));
                }
            }
        }
    }
}
