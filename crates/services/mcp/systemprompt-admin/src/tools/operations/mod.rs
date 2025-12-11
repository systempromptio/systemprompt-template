use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_scheduler::models::ScheduledJob;
use systemprompt_core_scheduler::repository::SchedulerRepository;
use systemprompt_core_scheduler::services::jobs;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    Column, ColumnType, ExecutionMetadata, TableArtifact, TableHints, ToolResponse,
};

pub fn operations_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "execute_job": {
                "type": "string",
                "description": "Optional job name to execute (leave blank to just list jobs)",
                "enum": [
                    "cleanup_anonymous_users",
                    "cleanup_inactive_sessions",
                    "database_cleanup",
                    "publish_content",
                    "evaluate_conversations",
                    "ingest_content",
                    "ingest_files",
                    "optimize_images",
                    "regenerate_static_content",
                    "rebuild_static_site"
                ]
            }
        }
    })
}

pub fn operations_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "Table of all scheduler jobs with current status",
        "properties": {
            "x-artifact-type": {"type": "string", "enum": ["table"]},
            "columns": {"type": "array"},
            "rows": {"type": "array"}
        },
        "required": ["x-artifact-type", "columns", "rows"]
    })
}

pub async fn handle_operations(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
    app_context: Arc<AppContext>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let execute_job = request
        .arguments
        .as_ref()
        .and_then(|args| args.get("execute_job"))
        .and_then(|v| v.as_str());

    if let Some(job_name) = execute_job {
        logger
            .info(
                "operations",
                &format!("Spawning job in background: {}", job_name),
            )
            .await
            .ok();

        spawn_job(job_name, pool.clone(), logger.clone(), app_context);
    }

    let repo = SchedulerRepository::new(pool.clone());
    let jobs = repo
        .list_enabled_jobs()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let table = build_jobs_table(jobs);

    let metadata = ExecutionMetadata::new().tool("operations");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        table,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Scheduler Jobs{}",
            execute_job
                .map(|j| format!(" (executing: {})", j))
                .unwrap_or_default()
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}

fn spawn_job(job_name: &str, pool: DbPool, logger: LogService, app_context: Arc<AppContext>) {
    let job_name = job_name.to_string();
    let pool = pool.clone();
    let logger = logger.clone();
    let app_context = app_context.clone();

    tokio::spawn(async move {
        logger
            .info(
                "operations",
                &format!("Starting manual execution: {}", job_name),
            )
            .await
            .ok();

        let result = match job_name.as_str() {
            "cleanup_anonymous_users" => {
                jobs::cleanup_anonymous_users(pool, logger.clone(), app_context).await
            }
            "cleanup_inactive_sessions" => {
                jobs::cleanup_inactive_sessions(pool, logger.clone(), app_context).await
            }
            "database_cleanup" => jobs::database_cleanup(pool, logger.clone(), app_context).await,
            "publish_content" => jobs::publish_content(pool, logger.clone(), app_context).await,
            "evaluate_conversations" => {
                jobs::evaluate_conversations(pool, logger.clone(), app_context).await
            }
            "ingest_content" => jobs::ingest_content(pool, logger.clone(), app_context).await,
            "ingest_files" => jobs::ingest_files(pool, logger.clone()).await,
            "optimize_images" => {
                systemprompt_core_scheduler::services::static_content::optimize_images(
                    pool,
                    logger.clone(),
                )
                .await
            }
            "regenerate_static_content" => {
                jobs::regenerate_static_content(pool, logger.clone(), app_context).await
            }
            "rebuild_static_site" => {
                jobs::rebuild_static_site(pool, logger.clone(), app_context).await
            }
            _ => {
                logger
                    .error("operations", &format!("Unknown job: {}", job_name))
                    .await
                    .ok();
                return;
            }
        };

        match result {
            Ok(_) => {
                logger
                    .info("operations", &format!("Completed: {}", job_name))
                    .await
                    .ok();
            }
            Err(e) => {
                logger
                    .error("operations", &format!("Failed {}: {}", job_name, e))
                    .await
                    .ok();
            }
        }
    });
}

fn build_jobs_table(jobs: Vec<ScheduledJob>) -> TableArtifact {
    let columns = vec![
        Column::new("job_name", ColumnType::String).with_header("Job Name"),
        Column::new("schedule", ColumnType::String).with_header("Schedule"),
        Column::new("enabled", ColumnType::Boolean).with_header("Enabled"),
        Column::new("last_run", ColumnType::String).with_header("Last Run"),
        Column::new("last_status", ColumnType::String).with_header("Status"),
        Column::new("run_count", ColumnType::Number).with_header("Run Count"),
        Column::new("last_error", ColumnType::String).with_header("Error"),
    ];

    let rows: Vec<serde_json::Value> = jobs
        .iter()
        .map(|job| {
            json!({
                "job_name": job.job_name,
                "schedule": job.schedule,
                "enabled": job.enabled,
                "last_run": job.last_run.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| "Never".to_string()),
                "last_status": job.last_status.as_deref().unwrap_or("—"),
                "run_count": job.run_count,
                "last_error": job.last_error.as_deref().unwrap_or(""),
            })
        })
        .collect();

    TableArtifact::new(columns).with_rows(rows).with_hints(
        TableHints::new()
            .with_sortable(vec![
                "job_name".to_string(),
                "last_run".to_string(),
                "run_count".to_string(),
            ])
            .filterable(),
    )
}
