use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use systemprompt_core_system::models::AppContext;
use tabled::settings::Style;
use tabled::{Table, Tabled};

#[derive(Args)]
pub struct AiTraceOptions {
    /// Task ID (can be partial, will match prefix)
    pub task_id: String,

    /// Show full conversation history (all messages from all AI requests)
    #[arg(long)]
    pub history: bool,

    /// Show artifacts produced by the task
    #[arg(long)]
    pub artifact: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Tabled)]
struct TaskInfoRow {
    #[tabled(rename = "Task ID")]
    task_id: String,
    #[tabled(rename = "Agent")]
    agent_name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Started")]
    started_at: String,
    #[tabled(rename = "Duration")]
    duration: String,
}

#[derive(Tabled)]
struct StepRow {
    #[tabled(rename = "#")]
    step_number: i32,
    #[tabled(rename = "Type")]
    step_type: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Duration")]
    duration: String,
}

#[derive(Tabled)]
struct AiRequestRow {
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "Tokens")]
    tokens: String,
    #[tabled(rename = "Cost")]
    cost: String,
    #[tabled(rename = "Latency")]
    latency: String,
}

#[derive(Tabled)]
struct ToolCallRow {
    #[tabled(rename = "Tool")]
    tool_name: String,
    #[tabled(rename = "Server")]
    server: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Duration")]
    duration: String,
}

#[derive(Tabled)]
struct ArtifactRow {
    #[tabled(rename = "ID")]
    artifact_id: String,
    #[tabled(rename = "Type")]
    artifact_type: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Tool")]
    tool_name: String,
}

fn truncate(s: &str, max_len: usize) -> String {
    let s = s.replace('\n', " ").replace('\r', "");
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s
    }
}

fn print_section(title: &str) {
    println!();
    println!("{}", format!("═══ {} ", title).bright_cyan().bold());
}

fn print_content_block(content: &str, color: colored::Color) {
    for line in content.lines() {
        println!("  {}", line.color(color));
    }
}

async fn get_full_task_id(pool: &Arc<PgPool>, partial_id: &str) -> Result<String> {
    let row = sqlx::query(
        "SELECT task_id FROM agent_tasks WHERE task_id LIKE $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(format!("{}%", partial_id))
    .fetch_optional(&**pool)
    .await?;

    row.map(|r| r.get::<String, _>("task_id"))
        .ok_or_else(|| anyhow::anyhow!("No task found matching: {}", partial_id))
}

async fn print_task_info(pool: &Arc<PgPool>, task_id: &str) -> Result<String> {
    let row = sqlx::query(
        r#"SELECT task_id, context_id, agent_name, status, created_at, started_at, completed_at, execution_time_ms
           FROM agent_tasks WHERE task_id = $1"#,
    )
    .bind(task_id)
    .fetch_one(&**pool)
    .await?;

    let context_id: String = row.get("context_id");
    let started: Option<chrono::DateTime<chrono::Utc>> = row.get("started_at");
    let execution_ms: Option<i32> = row.get("execution_time_ms");

    let rows = vec![TaskInfoRow {
        task_id: row.get::<String, _>("task_id")[..8].to_string(),
        agent_name: row.get("agent_name"),
        status: row.get("status"),
        started_at: started
            .map(|t| t.format("%H:%M:%S").to_string())
            .unwrap_or("-".to_string()),
        duration: execution_ms
            .map(|ms| format!("{}ms", ms))
            .unwrap_or("-".to_string()),
    }];

    print_section("TASK");
    let table = Table::new(rows).with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(context_id)
}

async fn print_user_input(pool: &Arc<PgPool>, task_id: &str) {
    let rows = sqlx::query(
        r#"SELECT mp.text_content
           FROM task_messages tm
           JOIN message_parts mp ON mp.message_id = tm.message_id AND mp.task_id = tm.task_id
           WHERE tm.task_id = $1 AND tm.role = 'user' AND mp.part_kind = 'text'
           ORDER BY tm.sequence_number DESC LIMIT 1"#,
    )
    .bind(task_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if let Some(row) = rows.first() {
        let content: Option<String> = row.get("text_content");
        if let Some(text) = content {
            print_section("USER INPUT");
            println!("  {}", text.bright_white());
        }
    }
}

async fn print_agent_response(pool: &Arc<PgPool>, task_id: &str) {
    let rows = sqlx::query(
        r#"SELECT mp.text_content
           FROM task_messages tm
           JOIN message_parts mp ON mp.message_id = tm.message_id AND mp.task_id = tm.task_id
           WHERE tm.task_id = $1 AND tm.role = 'agent' AND mp.part_kind = 'text'
           ORDER BY tm.sequence_number DESC LIMIT 1"#,
    )
    .bind(task_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if let Some(row) = rows.first() {
        let content: Option<String> = row.get("text_content");
        if let Some(text) = content {
            print_section("AGENT RESPONSE");
            print_content_block(&text, colored::Color::Green);
        }
    }
}

async fn print_execution_steps(pool: &Arc<PgPool>, task_id: &str) {
    let rows = sqlx::query(
        r#"SELECT
               id,
               content->>'type' as step_type,
               COALESCE(content->>'title', content->>'type') as title,
               status,
               duration_ms,
               error_message
           FROM task_execution_steps
           WHERE task_id = $1
           ORDER BY created_at"#,
    )
    .bind(task_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return;
    }

    let step_rows: Vec<StepRow> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| StepRow {
            step_number: (i + 1) as i32,
            step_type: r
                .get::<Option<String>, _>("step_type")
                .unwrap_or_else(|| "unknown".to_string()),
            title: truncate(
                &r.get::<Option<String>, _>("title")
                    .unwrap_or_else(|| "".to_string()),
                40,
            ),
            status: r.get("status"),
            duration: r
                .get::<Option<i32>, _>("duration_ms")
                .map(|ms| format!("{}ms", ms))
                .unwrap_or("-".to_string()),
        })
        .collect();

    print_section("EXECUTION STEPS");
    let table = Table::new(step_rows).with(Style::rounded()).to_string();
    println!("{}", table);

    for row in &rows {
        let status: String = row.get("status");
        if status == "failed" {
            if let Some(error) = row.get::<Option<String>, _>("error_message") {
                if !error.is_empty() {
                    let step_type = row
                        .get::<Option<String>, _>("step_type")
                        .unwrap_or_else(|| "step".to_string());
                    println!();
                    println!(
                        "  {} {} failed:",
                        "✗".bright_red(),
                        step_type.bright_yellow()
                    );
                    print_content_block(&error, colored::Color::Red);
                }
            }
        }
    }
}

async fn print_ai_requests(pool: &Arc<PgPool>, task_id: &str) -> Vec<String> {
    let rows = sqlx::query(
        r#"SELECT id, model, provider, input_tokens, output_tokens, cost_cents, latency_ms
           FROM ai_requests
           WHERE task_id = $1
           ORDER BY created_at"#,
    )
    .bind(task_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return vec![];
    }

    let request_ids: Vec<String> = rows.iter().map(|r| r.get("id")).collect();

    let ai_rows: Vec<AiRequestRow> = rows
        .iter()
        .map(|r| {
            let input: Option<i32> = r.get("input_tokens");
            let output: Option<i32> = r.get("output_tokens");
            let cost: i32 = r.get("cost_cents");
            let latency: Option<i32> = r.get("latency_ms");

            AiRequestRow {
                model: format!(
                    "{}/{}",
                    r.get::<String, _>("provider"),
                    r.get::<String, _>("model")
                ),
                tokens: format!(
                    "{} (in:{}, out:{})",
                    input.unwrap_or(0) + output.unwrap_or(0),
                    input.unwrap_or(0),
                    output.unwrap_or(0)
                ),
                cost: format!("${:.4}", cost as f64 / 1_000_000.0),
                latency: latency
                    .map(|ms| format!("{}ms", ms))
                    .unwrap_or("-".to_string()),
            }
        })
        .collect();

    print_section("AI REQUESTS");
    let table = Table::new(ai_rows).with(Style::rounded()).to_string();
    println!("{}", table);

    request_ids
}

async fn print_system_prompt(pool: &Arc<PgPool>, request_ids: &[String]) {
    if request_ids.is_empty() {
        return;
    }

    let row = sqlx::query(
        r#"SELECT content
           FROM ai_request_messages
           WHERE request_id = $1 AND role = 'system' AND sequence_number = 0
           LIMIT 1"#,
    )
    .bind(&request_ids[0])
    .fetch_optional(&**pool)
    .await
    .unwrap_or(None);

    if let Some(row) = row {
        let content: String = row.get("content");
        print_section("SYSTEM PROMPT");
        print_content_block(&content, colored::Color::BrightBlack);
    }
}

async fn print_conversation_history(pool: &Arc<PgPool>, request_ids: &[String]) {
    if request_ids.is_empty() {
        return;
    }

    print_section("CONVERSATION HISTORY");

    for (req_idx, request_id) in request_ids.iter().enumerate() {
        let rows = sqlx::query(
            r#"SELECT role, content, sequence_number
               FROM ai_request_messages
               WHERE request_id = $1
               ORDER BY sequence_number"#,
        )
        .bind(request_id)
        .fetch_all(&**pool)
        .await
        .unwrap_or_default();

        if rows.is_empty() {
            continue;
        }

        println!();
        println!(
            "{}",
            format!("── Request {} ──", req_idx + 1)
                .bright_blue()
                .bold()
        );

        for row in rows {
            let role: String = row.get("role");
            let content: String = row.get("content");
            let seq: i32 = row.get("sequence_number");

            let (label, color) = match role.as_str() {
                "system" => ("SYSTEM", colored::Color::BrightBlack),
                "user" => ("USER", colored::Color::White),
                "assistant" => ("ASSISTANT", colored::Color::Green),
                _ => (&role as &str, colored::Color::Yellow),
            };

            println!();
            println!("{}", format!("[{}] #{}", label, seq).color(color).bold());

            if role == "system" && content.len() > 500 {
                println!(
                    "  {}",
                    format!("[System prompt: {} chars]", content.len()).bright_black()
                );
            } else {
                print_content_block(&content, color);
            }
        }
    }
}

async fn print_mcp_executions(pool: &Arc<PgPool>, task_id: &str, context_id: &str) {
    let rows = sqlx::query(
        r#"SELECT mcp_execution_id, tool_name, mcp_server_name, status, execution_time_ms, error_message
           FROM mcp_tool_executions
           WHERE task_id = $1 OR context_id = $2
           ORDER BY started_at"#,
    )
    .bind(task_id)
    .bind(context_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        print_tool_errors_from_logs(pool, task_id, context_id).await;
        return;
    }

    let tool_rows: Vec<ToolCallRow> = rows
        .iter()
        .map(|r| ToolCallRow {
            tool_name: r.get("tool_name"),
            server: r.get("mcp_server_name"),
            status: r.get("status"),
            duration: r
                .get::<Option<i32>, _>("execution_time_ms")
                .map(|ms| format!("{}ms", ms))
                .unwrap_or("-".to_string()),
        })
        .collect();

    print_section("MCP TOOL EXECUTIONS");
    let table = Table::new(tool_rows).with(Style::rounded()).to_string();
    println!("{}", table);

    for row in &rows {
        let mcp_execution_id: String = row.get("mcp_execution_id");
        let tool_name: String = row.get("tool_name");
        let status: String = row.get("status");

        if status == "failed" {
            if let Some(error) = row.get::<Option<String>, _>("error_message") {
                println!();
                println!(
                    "  {} {} failed:",
                    "✗".bright_red(),
                    tool_name.bright_yellow()
                );
                print_content_block(&error, colored::Color::Red);
            }
        }

        print_mcp_linked_ai_requests(pool, &mcp_execution_id, &tool_name).await;
    }
}

async fn print_mcp_linked_ai_requests(pool: &Arc<PgPool>, mcp_execution_id: &str, tool_name: &str) {
    let rows = sqlx::query(
        r#"SELECT id, model, provider, input_tokens, output_tokens, cost_cents, latency_ms
           FROM ai_requests
           WHERE mcp_execution_id = $1
           ORDER BY created_at"#,
    )
    .bind(mcp_execution_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return;
    }

    println!();
    println!(
        "  {} AI requests made by {}:",
        "→".bright_blue(),
        tool_name.bright_cyan()
    );

    for row in &rows {
        let id: String = row.get("id");
        let provider: String = row.get("provider");
        let model: String = row.get("model");
        let input: Option<i32> = row.get("input_tokens");
        let output: Option<i32> = row.get("output_tokens");
        let latency: Option<i32> = row.get("latency_ms");

        println!(
            "    {} {}/{} | {} tokens | {}",
            truncate(&id, 8).bright_black(),
            provider.bright_yellow(),
            model.bright_green(),
            input.unwrap_or(0) + output.unwrap_or(0),
            latency
                .map(|ms| format!("{}ms", ms))
                .unwrap_or("-".to_string())
        );

        print_ai_request_messages(pool, &id).await;
    }
}

async fn print_ai_request_messages(pool: &Arc<PgPool>, request_id: &str) {
    let rows = sqlx::query(
        r#"SELECT role, LEFT(content, 500) as content_preview, sequence_number
           FROM ai_request_messages
           WHERE request_id = $1
           ORDER BY sequence_number"#,
    )
    .bind(request_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return;
    }

    for row in rows {
        let role: String = row.get("role");
        let content: String = row.get("content_preview");
        let seq: i32 = row.get("sequence_number");

        let (label_color, content_color) = match role.as_str() {
            "system" => (colored::Color::BrightBlack, colored::Color::BrightBlack),
            "user" => (colored::Color::White, colored::Color::White),
            "assistant" => (colored::Color::Green, colored::Color::Green),
            _ => (colored::Color::Yellow, colored::Color::Yellow),
        };

        let preview = if content.len() >= 500 {
            format!("{}...", truncate(&content, 200))
        } else if role == "system" && content.len() > 100 {
            format!("[System: {} chars]", content.len())
        } else {
            truncate(&content, 200)
        };

        println!(
            "      {} [{}] {}",
            format!("#{}", seq).bright_black(),
            role.to_uppercase().color(label_color),
            preview.color(content_color)
        );
    }
}

async fn print_tool_errors_from_logs(pool: &Arc<PgPool>, task_id: &str, context_id: &str) {
    let rows = sqlx::query(
        r#"SELECT timestamp, level, module, message
           FROM logs
           WHERE (task_id = $1 OR context_id = $2)
             AND (
                 (module LIKE '%_tools' OR module LIKE '%_manager' OR module LIKE 'create_%' OR module LIKE 'update_%' OR module LIKE 'research_%')
                 OR (level = 'ERROR' AND message LIKE '%tool%')
                 OR message LIKE 'Tool executed%'
                 OR message LIKE 'Tool failed%'
                 OR message LIKE 'MCP execution%'
             )
           ORDER BY timestamp"#,
    )
    .bind(task_id)
    .bind(context_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return;
    }

    print_section("TOOL EXECUTION LOGS");
    println!(
        "  {}",
        "(MCP execution records not found - showing logs)".bright_black()
    );
    println!();

    let mut has_errors = false;
    for row in &rows {
        let level: String = row.get("level");
        let module: String = row.get("module");
        let message: String = row.get("message");
        let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");

        let time_str = timestamp.format("%H:%M:%S%.3f").to_string();

        let (level_color, level_symbol) = match level.as_str() {
            "ERROR" => {
                has_errors = true;
                (colored::Color::Red, "✗")
            },
            "WARN" => (colored::Color::Yellow, "⚠"),
            "INFO" => (colored::Color::Green, "•"),
            "DEBUG" => (colored::Color::BrightBlack, "·"),
            _ => (colored::Color::White, "•"),
        };

        println!(
            "  {} {} [{}] {}",
            time_str.bright_black(),
            level_symbol.color(level_color),
            module.bright_cyan(),
            truncate(&message, 100).color(level_color)
        );
    }

    if has_errors {
        println!();
        println!("  {} Tool Errors:", "✗".bright_red());
        for row in &rows {
            let level: String = row.get("level");
            if level == "ERROR" {
                let message: String = row.get("message");
                let module: String = row.get("module");
                println!();
                println!("    {} {}:", module.bright_yellow(), "error".bright_red());
                print_content_block(&format!("      {}", message), colored::Color::Red);
            }
        }
    }
}

async fn print_artifacts(pool: &Arc<PgPool>, task_id: &str, context_id: &str) {
    let rows = sqlx::query(
        r#"SELECT ta.artifact_id, ta.artifact_type, ta.name, ta.source, ta.tool_name,
                  ap.part_kind, ap.text_content, ap.data_content
           FROM task_artifacts ta
           LEFT JOIN artifact_parts ap ON ta.artifact_id = ap.artifact_id AND ta.context_id = ap.context_id
           WHERE ta.task_id = $1 OR ta.context_id = $2
           ORDER BY ta.created_at, ap.sequence_number"#,
    )
    .bind(task_id)
    .bind(context_id)
    .fetch_all(&**pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return;
    }

    let mut seen_artifacts: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut artifact_rows: Vec<ArtifactRow> = Vec::new();

    for row in &rows {
        let artifact_id: String = row.get("artifact_id");
        if seen_artifacts.insert(artifact_id.clone()) {
            artifact_rows.push(ArtifactRow {
                artifact_id: truncate(&artifact_id, 12),
                artifact_type: row.get("artifact_type"),
                name: row
                    .get::<Option<String>, _>("name")
                    .map(|s| truncate(&s, 30))
                    .unwrap_or("-".to_string()),
                source: row
                    .get::<Option<String>, _>("source")
                    .unwrap_or("-".to_string()),
                tool_name: row
                    .get::<Option<String>, _>("tool_name")
                    .unwrap_or("-".to_string()),
            });
        }
    }

    print_section("ARTIFACTS");
    let table = Table::new(&artifact_rows)
        .with(Style::rounded())
        .to_string();
    println!("{}", table);

    let mut current_artifact: Option<String> = None;
    for row in &rows {
        let artifact_id: String = row.get("artifact_id");
        let part_kind: Option<String> = row.get("part_kind");

        if current_artifact.as_ref() != Some(&artifact_id) {
            current_artifact = Some(artifact_id.clone());
            let name: Option<String> = row.get("name");
            let artifact_type: String = row.get("artifact_type");
            println!();
            println!(
                "{}",
                format!(
                    "── {} ({}) ──",
                    name.unwrap_or_else(|| truncate(&artifact_id, 12)),
                    artifact_type
                )
                .bright_magenta()
                .bold()
            );
        }

        match part_kind.as_deref() {
            Some("text") => {
                if let Some(content) = row.get::<Option<String>, _>("text_content") {
                    print_content_block(&content, colored::Color::White);
                }
            },
            Some("data") => {
                if let Some(data) = row.get::<Option<serde_json::Value>, _>("data_content") {
                    let formatted = serde_json::to_string_pretty(&data).unwrap_or_default();
                    print_content_block(&formatted, colored::Color::Yellow);
                }
            },
            _ => {},
        }
    }
}

pub async fn execute(options: AiTraceOptions) -> Result<()> {
    dotenvy::dotenv().ok();

    let ctx = AppContext::new().await?;
    let pool = ctx
        .db_pool()
        .pool_arc()
        .expect("Database must be PostgreSQL");

    let task_id = get_full_task_id(&pool, &options.task_id)
        .await
        .context("Failed to find task")?;

    println!();
    println!("{}", format!("AI TRACE: {}", task_id).bright_cyan().bold());
    println!("{}", "═".repeat(60).bright_blue());

    let context_id = print_task_info(&pool, &task_id).await?;
    print_user_input(&pool, &task_id).await;
    print_execution_steps(&pool, &task_id).await;

    let request_ids = print_ai_requests(&pool, &task_id).await;

    print_system_prompt(&pool, &request_ids).await;

    if options.history {
        print_conversation_history(&pool, &request_ids).await;
    }

    print_mcp_executions(&pool, &task_id, &context_id).await;

    if options.artifact {
        print_artifacts(&pool, &task_id, &context_id).await;
    }

    print_agent_response(&pool, &task_id).await;

    println!();
    println!("{}", "═".repeat(60).bright_blue());
    println!();

    Ok(())
}
