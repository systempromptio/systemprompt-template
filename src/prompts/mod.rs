pub mod admin_analysis;
pub mod agent_management;
pub mod system_health;

use anyhow::Result;
use rmcp::{model::{PaginatedRequestParam, ListPromptsResult, Prompt, PromptArgument, GetPromptRequestParam, GetPromptResult, PromptMessage, PromptMessageRole, PromptMessageContent}, service::RequestContext, ErrorData as McpError, RoleServer};
use systemprompt_core_database::DbPool;

pub use admin_analysis::build_admin_analysis_prompt;
pub use agent_management::{
    build_agent_prompt_content, get_agent_operation_schema, AGENT_MANAGEMENT_PROMPT,
};
pub use system_health::build_system_health_prompt;

#[derive(Debug, Clone)]
pub struct AdminPrompts {
    _db_pool: DbPool,
    _server_name: String,
}

impl AdminPrompts {
    #[must_use] pub fn new(db_pool: DbPool, server_name: String) -> Self {
        Self {
            _db_pool: db_pool,
            _server_name: server_name,
        }
    }

    pub async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![
                Prompt {
                    name: "admin_analysis".into(),
                    description: Some("Comprehensive system analysis prompt for administrative tasks".into()),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "focus_area".into(),
                            description: Some("Area to focus analysis on (logs, database, system, users, all)".into()),
                            required: Some(false),
                            title: None,
                        },
                        PromptArgument {
                            name: "time_period".into(),
                            description: Some("Time period for analysis (1h, 24h, 7d, 30d)".into()),
                            required: Some(false),
                            title: None,
                        },
                    ]),
                    title: None,
                    icons: None,
                },
                Prompt {
                    name: "system_health".into(),
                    description: Some("System health check prompt with detailed analysis guidelines".into()),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "include_recommendations".into(),
                            description: Some("Include actionable recommendations in the analysis".into()),
                            required: Some(false),
                            title: None,
                        },
                    ]),
                    title: None,
                    icons: None,
                },
                Prompt {
                    name: "agent_management".into(),
                    description: Some("Agent management prompt for creating, updating, and managing agents with AI assistance".into()),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "task_type".into(),
                            description: Some("Type of agent task: design, review, optimize, troubleshoot".into()),
                            required: Some(false),
                            title: None,
                        },
                        PromptArgument {
                            name: "domain".into(),
                            description: Some("Agent domain or specialization area".into()),
                            required: Some(false),
                            title: None,
                        },
                    ]),
                    title: None,
                    icons: None,
                },
            ],
            next_cursor: None,
        })
    }

    pub async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match request.name.as_ref() {
            "admin_analysis" => {
                let focus_area = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("focus_area"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("all");

                let time_period = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("time_period"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("24h");

                let prompt_content =
                    admin_analysis::build_admin_analysis_prompt(focus_area, time_period);

                Ok(GetPromptResult {
                    description: Some(format!(
                        "Administrative analysis focused on {focus_area} over {time_period}"
                    )),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt_content),
                    }],
                })
            }
            "system_health" => {
                let include_recommendations = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("include_recommendations"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);

                let prompt_content =
                    system_health::build_system_health_prompt(include_recommendations);

                Ok(GetPromptResult {
                    description: Some(
                        "Comprehensive system health check with diagnostic guidance".to_string(),
                    ),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt_content),
                    }],
                })
            }
            "agent_management" => {
                let task_type = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("task_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("design");

                let domain = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("domain"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("general");

                let prompt_content =
                    agent_management::build_agent_prompt_content(task_type, domain);

                Ok(GetPromptResult {
                    description: Some(
                        "Agent management guidance for SystemPrompt architects".to_string(),
                    ),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt_content),
                    }],
                })
            }
            _ => Err(McpError::invalid_params(
                format!("Unknown prompt: {}", request.name),
                None,
            )),
        }
    }
}
