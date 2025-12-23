mod deployment_guide;
mod sync_workflow;

use anyhow::Result;

const DEFAULT_ENVIRONMENT: &str = "production";
const DEFAULT_INCLUDE_ROLLBACK: bool = true;
const DEFAULT_DIRECTION: &str = "push";
const DEFAULT_SCOPE: &str = "all";
use rmcp::{
    model::{
        GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam, Prompt,
        PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole,
    },
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use systemprompt::database::DbPool;

pub use deployment_guide::build_deployment_guide_prompt;
pub use sync_workflow::build_sync_workflow_prompt;

#[derive(Debug, Clone)]
pub struct InfrastructurePrompts {
    _db_pool: DbPool,
    _server_name: String,
}

impl InfrastructurePrompts {
    #[must_use]
    pub const fn new(db_pool: DbPool, server_name: String) -> Self {
        Self {
            _db_pool: db_pool,
            _server_name: server_name,
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: vec![
                Prompt {
                    name: "deployment_guide".into(),
                    description: Some(
                        "Step-by-step deployment guide for SystemPrompt applications".into(),
                    ),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "environment".into(),
                            description: Some(
                                "Target environment (development, staging, production)".into(),
                            ),
                            required: Some(false),
                            title: None,
                        },
                        PromptArgument {
                            name: "include_rollback".into(),
                            description: Some("Include rollback procedures in the guide".into()),
                            required: Some(false),
                            title: None,
                        },
                    ]),
                    title: None,
                    icons: None,
                },
                Prompt {
                    name: "sync_workflow".into(),
                    description: Some(
                        "Recommended sync workflow for keeping local and cloud in sync".into(),
                    ),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "direction".into(),
                            description: Some("Sync direction: push or pull".into()),
                            required: Some(false),
                            title: None,
                        },
                        PromptArgument {
                            name: "scope".into(),
                            description: Some("Sync scope: files, database, all".into()),
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

    #[allow(clippy::unused_async)]
    pub async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match request.name.as_ref() {
            "deployment_guide" => {
                let environment = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("environment"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(DEFAULT_ENVIRONMENT);

                let include_rollback = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("include_rollback"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(DEFAULT_INCLUDE_ROLLBACK);

                let prompt_content = build_deployment_guide_prompt(environment, include_rollback);

                Ok(GetPromptResult {
                    description: Some(format!("Deployment guide for {} environment", environment)),
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt_content),
                    }],
                })
            }
            "sync_workflow" => {
                let direction = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("direction"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(DEFAULT_DIRECTION);

                let scope = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("scope"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(DEFAULT_SCOPE);

                let prompt_content = build_sync_workflow_prompt(direction, scope);

                Ok(GetPromptResult {
                    description: Some(format!(
                        "Sync workflow guide for {} {} operations",
                        direction, scope
                    )),
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
