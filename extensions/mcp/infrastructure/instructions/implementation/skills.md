# Skills Integration

Skills are reusable instruction templates loaded from the database and injected into AI prompts.

---

## SkillInjector Pattern

```rust
use systemprompt_core_agent::services::SkillService;

pub struct SkillInjector {
    skill_service: Arc<SkillService>,
}

impl SkillInjector {
    #[must_use]
    pub async fn inject_for_tool(
        &self,
        skill_id: Option<&str>,
        base_prompt: String,
        ctx: &RequestContext,
    ) -> Result<String> {
        let Some(sid) = skill_id else {
            return Ok(base_prompt);
        };

        match self.skill_service.load_skill(sid, ctx).await {
            Ok(skill_content) => Ok(format!(
                "{}\n\n## Writing Guidance\n\n{}",
                base_prompt, skill_content
            )),
            Err(e) => {
                tracing::warn!(skill_id = %sid, error = %e, "Failed to load skill");
                Ok(base_prompt)
            }
        }
    }
}
```

---

## Combining Multiple Skills

```rust
let voice_content = skill_loader
    .load_skill("edwards_voice", &ctx)
    .await
    .ok()
    .filter(|s| !s.is_empty());

let platform_skill = skill_loader.load_skill(skill_id, &ctx).await?;

let combined = match voice_content {
    Some(voice) => format!("{}\n\n---\n\n{}", voice, platform_skill),
    None => platform_skill,
};

let system_prompt = format!(
    "{}\n\n## Writing Guidance\n\n{}",
    base_instructions, combined
);
```

---

## AI Service with Skills

```rust
#[must_use]
pub async fn generate_with_skill(
    ai_service: &AiService,
    skill_loader: &SkillService,
    skill_id: &str,
    user_prompt: &str,
    ctx: RequestContext,
) -> Result<String> {
    let skill_content = skill_loader.load_skill(skill_id, &ctx).await?;

    let messages = vec![
        AiMessage { role: MessageRole::System, content: skill_content },
        AiMessage { role: MessageRole::User, content: user_prompt.to_string() },
    ];

    let model_config = ctx.tool_model_config()?;
    let request = AiRequest::builder(
        messages,
        model_config.provider.as_deref().ok_or_else(|| anyhow!("No provider"))?,
        model_config.model.as_deref().ok_or_else(|| anyhow!("No model"))?,
        model_config.max_output_tokens.ok_or_else(|| anyhow!("No max tokens"))?,
        ctx,
    ).build();

    let response = ai_service.generate(&request).await?;
    Ok(response.content)
}
```

---

## Complete Tool with Skills

```rust
use serde::Deserialize;

pub struct ResearchTool {
    ai_service: Arc<AiService>,
    skill_loader: Arc<SkillService>,
}

#[derive(Debug, Deserialize)]
pub struct ResearchInput {
    topic: String,
    #[serde(default)]
    skill_id: Option<String>,
}

#[async_trait]
impl ToolHandler for ResearchTool {
    const NAME: &'static str = "research";
    type Input = ResearchInput;

    fn tool() -> Tool {
        Tool {
            name: Self::NAME.into(),
            description: Some("Research a topic using AI with optional skill".into()),
            input_schema: Arc::new(json!({
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic to research" },
                    "skill_id": { "type": "string", "description": "Optional skill ID" }
                },
                "required": ["topic"]
            }).as_object().expect("valid schema").clone()),
            ..Default::default()
        }
    }

    async fn execute(
        &self,
        input: Self::Input,
        ctx: ToolContext<'_>,
    ) -> Result<CallToolResult, McpError> {
        report_progress(&ctx.progress, 0.0, "Starting research...").await;

        let base_prompt = "You are a research assistant. Provide thorough analysis.";
        let system_prompt = match input.skill_id.as_deref() {
            Some(sid) => {
                report_progress(&ctx.progress, 10.0, "Loading skill...").await;
                match ctx.skill_loader.load_skill(sid, &ctx.request).await {
                    Ok(skill) => format!("{}\n\n## Guidance\n\n{}", base_prompt, skill),
                    Err(e) => {
                        tracing::warn!(skill_id = %sid, error = %e, "Skill load failed");
                        base_prompt.to_string()
                    }
                }
            }
            None => base_prompt.to_string(),
        };

        report_progress(&ctx.progress, 20.0, "Querying AI...").await;

        let messages = vec![
            AiMessage { role: MessageRole::System, content: system_prompt },
            AiMessage { role: MessageRole::User, content: input.topic.clone() },
        ];

        let model_config = ctx.request.tool_model_config()
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        let request = AiRequest::builder(
            messages,
            model_config.provider.as_deref()
                .ok_or_else(|| ToolError::InvalidInput("No provider".into()))?,
            model_config.model.as_deref()
                .ok_or_else(|| ToolError::InvalidInput("No model".into()))?,
            model_config.max_output_tokens
                .ok_or_else(|| ToolError::InvalidInput("No max tokens".into()))?,
            ctx.request.clone(),
        ).build();

        let response = execute_with_timeout(self.ai_service.generate(&request))
            .await
            .map_err(|e| ToolError::AiService(e.to_string()))?;

        report_progress(&ctx.progress, 80.0, "Processing results...").await;

        let result = ResearchResult {
            topic: input.topic,
            content: response.content,
            skill_used: input.skill_id,
        };

        report_progress(&ctx.progress, 100.0, "Research complete").await;

        Ok(build_tool_response(
            Self::NAME,
            result,
            "Research completed successfully",
            ctx.execution_id,
        ))
    }
}
```

---

## See Also

- [tools.md](./tools.md) - Tool implementation
- [progress.md](./progress.md) - Progress reporting
- [errors.md](./errors.md) - Error handling
