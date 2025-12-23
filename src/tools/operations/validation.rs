use rmcp::{model::{CallToolResult, Content}, ErrorData as McpError};
use serde_json::{json, Value as JsonValue};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use systemprompt_models::ConfigLoader;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardArtifact, DashboardHints, DashboardSection, ExecutionMetadata,
    LayoutMode, LayoutWidth, SectionLayout, SectionType, TableArtifact, ToolResponse,
};

/// Validation issue representation
#[derive(Debug)]
struct ValidationIssue {
    severity: String,
    category: String,
    file: String,
    message: String,
}

/// Result of validation operations
#[derive(Debug, Default)]
struct ValidationResult {
    issues: Vec<ValidationIssue>,
    passed: usize,
    warnings: usize,
    errors: usize,
}

impl ValidationResult {
    fn add_error(&mut self, category: &str, file: &str, message: &str) {
        self.errors += 1;
        self.issues.push(ValidationIssue {
            severity: "error".to_string(),
            category: category.to_string(),
            file: file.to_string(),
            message: message.to_string(),
        });
    }

    fn add_warning(&mut self, category: &str, file: &str, message: &str) {
        self.warnings += 1;
        self.issues.push(ValidationIssue {
            severity: "warning".to_string(),
            category: category.to_string(),
            file: file.to_string(),
            message: message.to_string(),
        });
    }

    fn add_pass(&mut self) {
        self.passed += 1;
    }
}

fn get_services_path() -> PathBuf {
    std::env::var("SYSTEMPROMPT_SERVICES_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/www/html/systemprompt-template/services"))
}

fn get_config_path() -> String {
    std::env::var("SYSTEMPROMPT_CONFIG_PATH")
        .unwrap_or_else(|_| "/var/www/html/systemprompt-template/services/config/config.yml".to_string())
}

/// Validate skills configuration
pub async fn handle_validate_skills(
    _args: &serde_json::Map<String, JsonValue>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    tracing::info!("Starting skills validation");

    let services_path = get_services_path();
    let skills_dir = services_path.join("skills");
    let mut result = ValidationResult::default();

    // Check skills directory exists
    if !skills_dir.exists() {
        result.add_error("structure", "services/skills", "Skills directory not found");
        return build_validation_response("Skills Validation", &result, mcp_execution_id);
    }
    result.add_pass();

    // Check master config exists
    let master_config_path = skills_dir.join("config.yml");
    if !master_config_path.exists() {
        result.add_error("structure", "services/skills/config.yml", "Master skills config not found");
        return build_validation_response("Skills Validation", &result, mcp_execution_id);
    }

    // Parse master config
    let master_content = match std::fs::read_to_string(&master_config_path) {
        Ok(c) => c,
        Err(e) => {
            result.add_error("read", "services/skills/config.yml", &format!("Failed to read: {e}"));
            return build_validation_response("Skills Validation", &result, mcp_execution_id);
        }
    };

    #[derive(serde::Deserialize)]
    struct SkillsMaster {
        #[serde(default)]
        includes: Vec<String>,
    }

    let master: SkillsMaster = match serde_yaml::from_str(&master_content) {
        Ok(c) => c,
        Err(e) => {
            result.add_error("parse", "services/skills/config.yml", &format!("Invalid YAML: {e}"));
            return build_validation_response("Skills Validation", &result, mcp_execution_id);
        }
    };
    result.add_pass();

    // Collect known agents for cross-reference validation
    let agents_dir = services_path.join("agents");
    let known_agents = collect_agent_names(&agents_dir);

    let mut skill_ids: HashSet<String> = HashSet::new();

    // Validate each included skill
    for include in &master.includes {
        let skill_path = skills_dir.join(include);
        let skill_dir = skill_path.parent().unwrap_or(&skills_dir);
        let relative_path = format!("services/skills/{include}");

        if !skill_path.exists() {
            result.add_error("include", &relative_path, "Included config file not found");
            continue;
        }

        let skill_content = match std::fs::read_to_string(&skill_path) {
            Ok(c) => c,
            Err(e) => {
                result.add_error("read", &relative_path, &format!("Failed to read: {e}"));
                continue;
            }
        };

        #[derive(serde::Deserialize)]
        struct SkillConfig {
            id: String,
            #[serde(default)]
            file: Option<String>,
            #[serde(default)]
            assigned_agents: Vec<String>,
            #[serde(default)]
            tags: Vec<String>,
            #[serde(default)]
            description: Option<String>,
        }

        let skill: SkillConfig = match serde_yaml::from_str(&skill_content) {
            Ok(c) => c,
            Err(e) => {
                result.add_error("parse", &relative_path, &format!("Invalid YAML: {e}"));
                continue;
            }
        };

        // Check duplicate IDs
        if skill_ids.contains(&skill.id) {
            result.add_error("duplicate", &relative_path, &format!("Duplicate skill ID: {}", skill.id));
        } else {
            skill_ids.insert(skill.id.clone());
            result.add_pass();
        }

        // Check ID format
        if skill.id.contains('-') || skill.id.contains(' ') {
            result.add_warning("naming", &relative_path, &format!("Skill ID '{}' should be snake_case", skill.id));
        }

        // Check index file exists
        let index_file = skill.file.as_deref().unwrap_or("index.md");
        let index_path = skill_dir.join(index_file);
        if !index_path.exists() {
            result.add_error("missing_file", &relative_path, &format!("Skill file '{}' not found", index_file));
        } else {
            result.add_pass();
        }

        // Check assigned agents exist
        for agent in &skill.assigned_agents {
            if !known_agents.contains(agent) {
                result.add_warning("reference", &relative_path, &format!("Assigned agent '{}' not found", agent));
            }
        }

        // Check has tags
        if skill.tags.is_empty() {
            result.add_warning("metadata", &relative_path, "Skill has no tags defined");
        }

        // Check has description
        if skill.description.is_none() || skill.description.as_ref().map_or(true, |d| d.is_empty()) {
            result.add_warning("metadata", &relative_path, "Skill has no description");
        }
    }

    tracing::info!(
        passed = result.passed,
        warnings = result.warnings,
        errors = result.errors,
        "Skills validation complete"
    );

    build_validation_response("Skills Validation", &result, mcp_execution_id)
}

/// Validate agents configuration
pub async fn handle_validate_agents(
    _args: &serde_json::Map<String, JsonValue>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    tracing::info!("Starting agents validation");

    let services_path = get_services_path();
    let agents_dir = services_path.join("agents");
    let mut result = ValidationResult::default();

    if !agents_dir.exists() {
        result.add_error("structure", "services/agents", "Agents directory not found");
        return build_validation_response("Agents Validation", &result, mcp_execution_id);
    }
    result.add_pass();

    // Collect known skills for cross-reference validation
    let skills_dir = services_path.join("skills");
    let known_skills = collect_skill_ids(&skills_dir);

    let mut agent_names: HashSet<String> = HashSet::new();
    let mut agent_ports: HashMap<u16, String> = HashMap::new();

    let entries = match std::fs::read_dir(&agents_dir) {
        Ok(e) => e,
        Err(e) => {
            result.add_error("read", "services/agents", &format!("Failed to read directory: {e}"));
            return build_validation_response("Agents Validation", &result, mcp_execution_id);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().map_or(false, |e| e == "yml" || e == "yaml") {
            continue;
        }

        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let relative_path = format!("services/agents/{file_name}");

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                result.add_error("read", &relative_path, &format!("Failed to read: {e}"));
                continue;
            }
        };

        #[derive(serde::Deserialize)]
        struct AgentFile {
            agents: HashMap<String, AgentConfig>,
        }

        #[derive(serde::Deserialize)]
        struct AgentConfig {
            name: String,
            port: u16,
            #[serde(default)]
            card: Option<AgentCard>,
            #[serde(default)]
            metadata: Option<AgentMetadata>,
        }

        #[derive(serde::Deserialize)]
        struct AgentCard {
            #[serde(default)]
            skills: Vec<SkillRef>,
            #[serde(default)]
            security: Vec<serde_json::Value>,
        }

        #[derive(serde::Deserialize)]
        struct SkillRef {
            id: String,
        }

        #[derive(serde::Deserialize)]
        struct AgentMetadata {
            #[serde(default, rename = "systemPrompt")]
            system_prompt: Option<String>,
        }

        let agent_file: AgentFile = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                result.add_error("parse", &relative_path, &format!("Invalid YAML: {e}"));
                continue;
            }
        };

        for (key, agent) in &agent_file.agents {
            // Check name matches key
            if key != &agent.name {
                result.add_warning("consistency", &relative_path, &format!("Agent key '{}' doesn't match name '{}'", key, agent.name));
            }

            // Check duplicate names
            if agent_names.contains(&agent.name) {
                result.add_error("duplicate", &relative_path, &format!("Duplicate agent name: {}", agent.name));
            } else {
                agent_names.insert(agent.name.clone());
                result.add_pass();
            }

            // Check port conflicts
            if let Some(existing) = agent_ports.get(&agent.port) {
                result.add_error("port_conflict", &relative_path, &format!("Port {} already used by agent '{}'", agent.port, existing));
            } else {
                agent_ports.insert(agent.port, agent.name.clone());
                result.add_pass();
            }

            // Check port range
            if agent.port < 9000 || agent.port > 9999 {
                result.add_warning("port_range", &relative_path, &format!("Agent port {} outside recommended range 9000-9999", agent.port));
            }

            // Check system prompt
            if let Some(metadata) = &agent.metadata {
                if metadata.system_prompt.is_none() || metadata.system_prompt.as_ref().map_or(true, |p| p.trim().is_empty()) {
                    result.add_warning("config", &relative_path, &format!("Agent '{}' has no system prompt", agent.name));
                } else {
                    result.add_pass();
                }
            } else {
                result.add_warning("config", &relative_path, &format!("Agent '{}' has no metadata section", agent.name));
            }

            // Check card and skills
            if let Some(card) = &agent.card {
                for skill_ref in &card.skills {
                    if !known_skills.contains(&skill_ref.id) {
                        result.add_warning("reference", &relative_path, &format!("Agent '{}' references unknown skill '{}'", agent.name, skill_ref.id));
                    }
                }

                if card.security.is_empty() {
                    result.add_warning("security", &relative_path, &format!("Agent '{}' has no security configuration", agent.name));
                }
            } else {
                result.add_warning("config", &relative_path, &format!("Agent '{}' has no card configuration", agent.name));
            }
        }
    }

    tracing::info!(
        passed = result.passed,
        warnings = result.warnings,
        errors = result.errors,
        "Agents validation complete"
    );

    build_validation_response("Agents Validation", &result, mcp_execution_id)
}

/// Validate full configuration using core ConfigLoader
pub async fn handle_validate_config(
    _args: &serde_json::Map<String, JsonValue>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    tracing::info!("Starting full configuration validation using core loader");

    let config_path = get_config_path();
    let mut result = ValidationResult::default();

    // Use core ConfigLoader to validate
    match ConfigLoader::validate_file(&config_path).await {
        Ok(()) => {
            result.add_pass();
            tracing::info!("Core config validation passed");
        }
        Err(e) => {
            result.add_error("config", &config_path, &format!("Validation failed: {e}"));
            tracing::error!(error = %e, "Core config validation failed");
        }
    }

    // Also run skills and agents validation
    let services_path = get_services_path();

    // Skills validation
    let skills_dir = services_path.join("skills");
    if skills_dir.exists() {
        let skills_result = validate_skills_internal(&skills_dir).await;
        result.issues.extend(skills_result.issues);
        result.passed += skills_result.passed;
        result.warnings += skills_result.warnings;
        result.errors += skills_result.errors;
    }

    // Agents validation
    let agents_dir = services_path.join("agents");
    if agents_dir.exists() {
        let agents_result = validate_agents_internal(&agents_dir, &skills_dir).await;
        result.issues.extend(agents_result.issues);
        result.passed += agents_result.passed;
        result.warnings += agents_result.warnings;
        result.errors += agents_result.errors;
    }

    tracing::info!(
        passed = result.passed,
        warnings = result.warnings,
        errors = result.errors,
        "Full validation complete"
    );

    build_validation_response("Configuration Validation", &result, mcp_execution_id)
}

// Helper functions

fn collect_agent_names(agents_dir: &PathBuf) -> HashSet<String> {
    let mut names = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yml" || e == "yaml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    #[derive(serde::Deserialize)]
                    struct AgentFile {
                        agents: HashMap<String, serde_json::Value>,
                    }
                    if let Ok(file) = serde_yaml::from_str::<AgentFile>(&content) {
                        names.extend(file.agents.keys().cloned());
                    }
                }
            }
        }
    }
    names
}

fn collect_skill_ids(skills_dir: &PathBuf) -> HashSet<String> {
    let mut ids = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let config_path = path.join("config.yml");
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        #[derive(serde::Deserialize)]
                        struct SkillConfig {
                            id: String,
                        }
                        if let Ok(skill) = serde_yaml::from_str::<SkillConfig>(&content) {
                            ids.insert(skill.id);
                        }
                    }
                }
            }
        }
    }
    ids
}

async fn validate_skills_internal(skills_dir: &PathBuf) -> ValidationResult {
    let mut result = ValidationResult::default();

    let master_config_path = skills_dir.join("config.yml");
    if !master_config_path.exists() {
        result.add_error("structure", "services/skills/config.yml", "Master config not found");
        return result;
    }

    if let Ok(content) = std::fs::read_to_string(&master_config_path) {
        #[derive(serde::Deserialize)]
        struct SkillsMaster {
            #[serde(default)]
            includes: Vec<String>,
        }

        if let Ok(config) = serde_yaml::from_str::<SkillsMaster>(&content) {
            result.add_pass();
            for include in &config.includes {
                let path = skills_dir.join(include);
                if !path.exists() {
                    result.add_error("include", &format!("skills/{include}"), "File not found");
                } else {
                    result.add_pass();
                }
            }
        }
    }

    result
}

async fn validate_agents_internal(agents_dir: &PathBuf, skills_dir: &PathBuf) -> ValidationResult {
    let mut result = ValidationResult::default();
    let mut ports_used: HashMap<u16, String> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yml" || e == "yaml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    #[derive(serde::Deserialize)]
                    struct AgentFile {
                        agents: HashMap<String, AgentConfig>,
                    }
                    #[derive(serde::Deserialize)]
                    struct AgentConfig {
                        name: String,
                        port: u16,
                    }

                    match serde_yaml::from_str::<AgentFile>(&content) {
                        Ok(agent_file) => {
                            result.add_pass();
                            for (_, agent) in &agent_file.agents {
                                if let Some(existing) = ports_used.get(&agent.port) {
                                    result.add_error("port_conflict", &path.display().to_string(), &format!("Port {} conflicts with {}", agent.port, existing));
                                } else {
                                    ports_used.insert(agent.port, agent.name.clone());
                                }
                            }
                        }
                        Err(e) => {
                            result.add_error("parse", &path.display().to_string(), &format!("Invalid YAML: {e}"));
                        }
                    }
                }
            }
        }
    }

    result
}

fn build_validation_response(
    title: &str,
    result: &ValidationResult,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let status = if result.errors > 0 {
        "error"
    } else if result.warnings > 0 {
        "warning"
    } else {
        "success"
    };

    let status_icon = if result.errors > 0 {
        "x-circle"
    } else if result.warnings > 0 {
        "alert-triangle"
    } else {
        "check-circle"
    };

    let mut dashboard = DashboardArtifact::new(title)
        .with_hints(DashboardHints::new().with_layout(LayoutMode::Vertical));

    // Add metrics cards
    dashboard = dashboard.add_section(
        DashboardSection::new("metrics", "Validation Results", SectionType::MetricsCards)
            .with_data(json!({
                "cards": [
                    {"title": "Passed", "value": result.passed, "icon": "check", "status": "success"},
                    {"title": "Warnings", "value": result.warnings, "icon": "alert-triangle", "status": if result.warnings > 0 { "warning" } else { "neutral" }},
                    {"title": "Errors", "value": result.errors, "icon": "x-circle", "status": if result.errors > 0 { "error" } else { "neutral" }},
                    {"title": "Status", "value": if result.errors > 0 { "Failed" } else if result.warnings > 0 { "Warnings" } else { "Passed" }, "icon": status_icon, "status": status}
                ]
            }))
            .with_layout(SectionLayout { width: LayoutWidth::Full, order: 1 }),
    );

    // Add issues table if there are any
    if !result.issues.is_empty() {
        let columns = vec![
            Column::new("severity", ColumnType::String).with_label("Severity"),
            Column::new("category", ColumnType::String).with_label("Category"),
            Column::new("file", ColumnType::String).with_label("File"),
            Column::new("message", ColumnType::String).with_label("Issue"),
        ];

        let rows: Vec<JsonValue> = result.issues.iter().map(|issue| {
            json!({"severity": issue.severity, "category": issue.category, "file": issue.file, "message": issue.message})
        }).collect();

        let table = TableArtifact::new(columns).with_rows(rows);

        dashboard = dashboard.add_section(
            DashboardSection::new("issues", "Issues Found", SectionType::Table)
                .with_data(json!({ "table": table }))
                .with_layout(SectionLayout { width: LayoutWidth::Full, order: 2 }),
        );
    }

    let metadata = ExecutionMetadata::new().tool("operations");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(&artifact_id, mcp_execution_id.clone(), dashboard, metadata.clone());

    // Build text summary
    let mut text_summary = format!("{title}\n\nResults: {} passed, {} warnings, {} errors\n\n", result.passed, result.warnings, result.errors);

    if !result.issues.is_empty() {
        text_summary.push_str("Issues:\n");
        for issue in &result.issues {
            text_summary.push_str(&format!("- [{}] {}: {} - {}\n", issue.severity.to_uppercase(), issue.category, issue.file, issue.message));
        }
    } else {
        text_summary.push_str("No issues found. Configuration is valid.");
    }

    Ok(CallToolResult {
        content: vec![Content::text(text_summary)],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
