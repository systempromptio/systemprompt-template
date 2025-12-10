use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Args;
use colored::Colorize;
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::time::Duration;
use systemprompt_core_system::models::AppContext;
use tabled::settings::Style;
use tabled::{Table, Tabled};
use uuid::Uuid;

#[derive(Args)]
pub struct TraceOptions {
    /// Specify agent name (default: first from registry)
    #[arg(long)]
    agent: Option<String>,

    /// Custom message to send (default: "Hello")
    #[arg(long, short = 'm')]
    message: Option<String>,

    /// Show detailed metadata for each event
    #[arg(long)]
    verbose: bool,

    /// Output as JSON instead of formatted text
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone)]
struct TraceEvent {
    event_type: String,
    timestamp: DateTime<Utc>,
    details: String,
    user_id: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
    context_id: Option<String>,
    metadata: Option<String>,
}

#[derive(Tabled)]
struct TraceRow {
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Delta")]
    delta: String,
    #[tabled(rename = "Type")]
    event_type: String,
    #[tabled(rename = "Details")]
    details: String,
    #[tabled(rename = "Latency")]
    latency: String,
}

async fn get_first_agent(client: &Client, base_url: &str, token: &str) -> Result<String> {
    let response = client
        .get(format!("{}/api/v1/agents/registry", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("Failed to fetch agent registry")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch agent registry: {}", response.status());
    }

    let registry: Value = response.json().await?;
    let agents = registry
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("No agents array in registry response"))?;

    if agents.is_empty() {
        anyhow::bail!("No agents found in registry. Is the API running?");
    }

    agents[0]
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No agent name found in registry"))
}

async fn get_anonymous_token(client: &Client, base_url: &str) -> Result<String> {
    let response = client
        .post(format!("{}/api/v1/core/oauth/session", base_url))
        .header("Content-Type", "application/json")
        .json(&json!({}))
        .send()
        .await
        .context("Failed to get anonymous token")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to get anonymous token: {} - {}", status, body);
    }

    let body: Value = response.json().await?;
    body.get("access_token")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No access_token in auth response: {:?}", body))
}

async fn create_context(client: &Client, base_url: &str, token: &str) -> Result<String> {
    let response = client
        .post(format!("{}/api/v1/core/contexts", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&json!({
            "name": "CLI Trace Test"
        }))
        .send()
        .await
        .context("Failed to create context")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to create context: {}", response.status());
    }

    let body: Value = response.json().await?;
    body.get("context_id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No context_id in response"))
}

async fn send_test_message(
    client: &Client,
    base_url: &str,
    agent_name: &str,
    token: &str,
    trace_id: &str,
    context_id: &str,
    message: &str,
) -> Result<()> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "message/send",
        "params": {
            "message": {
                "contextId": context_id,
                "messageId": uuid::Uuid::new_v4().to_string(),
                "role": "user",
                "kind": "message",
                "parts": [
                    {
                        "kind": "text",
                        "text": message
                    }
                ]
            }
        },
        "id": uuid::Uuid::new_v4().to_string()
    });

    let response = client
        .post(format!("{}/api/v1/agents/{}/", base_url, agent_name))
        .header("Authorization", format!("Bearer {}", token))
        .header("x-trace-id", trace_id)
        .header("Content-Type", "application/json")
        .timeout(Duration::from_secs(300)) // 5 minutes for long-running operations like research
        .json(&payload)
        .send()
        .await
        .context("Failed to send message to agent")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Agent request failed: {} - {}", status, body);
    }

    Ok(())
}

async fn send_and_trace(options: &TraceOptions, base_url: &str) -> Result<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(360)) // 6 minutes total for long operations
        .build()?;

    let trace_id = Uuid::new_v4().to_string();
    let message = options.message.as_deref().unwrap_or("Hello");

    println!();
    println!("{}", "Sending test message...".bright_cyan().bold());
    println!("{}", "─".repeat(60).bright_blue());

    println!("  Trace ID: {}", trace_id.bright_yellow());

    let token = get_anonymous_token(&client, base_url)
        .await
        .context("Failed to authenticate")?;
    println!("  {} Got anonymous token", "[OK]".bright_green());

    let agent_name = if let Some(ref agent) = options.agent {
        agent.clone()
    } else {
        get_first_agent(&client, base_url, &token).await?
    };
    println!(
        "  {} Using agent: {}",
        "[OK]".bright_green(),
        agent_name.bright_yellow()
    );

    let context_id = create_context(&client, base_url, &token).await?;
    println!(
        "  {} Created context: {}...",
        "[OK]".bright_green(),
        &context_id[..context_id.len().min(8)]
    );

    println!(
        "  {} Sending message: \"{}\"",
        "->".bright_blue(),
        message.bright_white()
    );

    send_test_message(
        &client,
        base_url,
        &agent_name,
        &token,
        &trace_id,
        &context_id,
        message,
    )
    .await?;

    println!(
        "  {} Message sent, waiting for processing...",
        "[OK]".bright_green()
    );

    tokio::time::sleep(Duration::from_millis(500)).await;

    println!();

    Ok(trace_id)
}

async fn fetch_trace_events(pool: &Arc<PgPool>, trace_id: &str) -> Result<Vec<TraceEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT
            timestamp,
            level as type,
            CONCAT(module, ': ', message) as details,
            user_id,
            session_id,
            task_id,
            context_id,
            metadata
        FROM logs
        WHERE trace_id = $1
        ORDER BY timestamp ASC
        "#,
    )
    .bind(trace_id)
    .fetch_all(&**pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let timestamp: DateTime<Utc> = row.get("timestamp");

        events.push(TraceEvent {
            event_type: row.get::<String, _>("type"),
            timestamp,
            details: row.get::<String, _>("details"),
            user_id: row.get::<Option<String>, _>("user_id"),
            session_id: row.get::<Option<String>, _>("session_id"),
            task_id: row.get::<Option<String>, _>("task_id"),
            context_id: row.get::<Option<String>, _>("context_id"),
            metadata: row.get::<Option<String>, _>("metadata"),
        });
    }

    Ok(events)
}

#[derive(Debug)]
struct AiRequestSummary {
    total_cost_cents: i64,
    total_tokens: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    request_count: i64,
    total_latency_ms: i64,
}

async fn fetch_ai_request_summary(pool: &Arc<PgPool>, trace_id: &str) -> Result<AiRequestSummary> {
    let row = sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(cost_cents), 0)::bigint as total_cost_cents,
            COALESCE(SUM(COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)), 0)::bigint as total_tokens,
            COALESCE(SUM(input_tokens), 0)::bigint as total_input_tokens,
            COALESCE(SUM(output_tokens), 0)::bigint as total_output_tokens,
            COUNT(*)::bigint as request_count,
            COALESCE(SUM(latency_ms), 0)::bigint as total_latency_ms
        FROM ai_requests
        WHERE trace_id = $1
        "#,
    )
    .bind(trace_id)
    .fetch_one(&**pool)
    .await?;

    Ok(AiRequestSummary {
        total_cost_cents: row.get("total_cost_cents"),
        total_tokens: row.get("total_tokens"),
        total_input_tokens: row.get("total_input_tokens"),
        total_output_tokens: row.get("total_output_tokens"),
        request_count: row.get("request_count"),
        total_latency_ms: row.get("total_latency_ms"),
    })
}

async fn fetch_ai_request_events(pool: &Arc<PgPool>, trace_id: &str) -> Result<Vec<TraceEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT
            created_at as timestamp,
            provider,
            model,
            input_tokens,
            output_tokens,
            cost_cents,
            latency_ms,
            status,
            user_id,
            session_id,
            task_id,
            context_id
        FROM ai_requests
        WHERE trace_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(trace_id)
    .fetch_all(&**pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let timestamp: DateTime<Utc> = row.get("timestamp");
        let provider: String = row.get("provider");
        let model: String = row.get("model");
        let input_tokens: Option<i32> = row.get("input_tokens");
        let output_tokens: Option<i32> = row.get("output_tokens");
        let cost_cents: i32 = row.get("cost_cents");
        let latency_ms: Option<i32> = row.get("latency_ms");
        let status: String = row.get("status");

        let details = format!(
            "{}/{}: {} (in:{}, out:{}, {}ms)",
            provider,
            model,
            status,
            input_tokens.unwrap_or(0),
            output_tokens.unwrap_or(0),
            latency_ms.unwrap_or(0)
        );

        let metadata = json!({
            "cost_cents": cost_cents,
            "latency_ms": latency_ms,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "tokens_used": input_tokens.unwrap_or(0) + output_tokens.unwrap_or(0),
            "provider": provider,
            "model": model
        });

        events.push(TraceEvent {
            event_type: "AI".to_string(),
            timestamp,
            details,
            user_id: row.get::<Option<String>, _>("user_id"),
            session_id: row.get::<Option<String>, _>("session_id"),
            task_id: row.get::<Option<String>, _>("task_id"),
            context_id: row.get::<Option<String>, _>("context_id"),
            metadata: Some(metadata.to_string()),
        });
    }

    Ok(events)
}

#[derive(Debug)]
struct McpExecutionSummary {
    execution_count: i64,
    total_execution_time_ms: i64,
}

#[derive(Debug)]
struct ExecutionStep {
    step_count: i64,
    completed_count: i64,
    failed_count: i64,
    pending_count: i64,
}

async fn fetch_mcp_execution_summary(
    pool: &Arc<PgPool>,
    trace_id: &str,
) -> Result<McpExecutionSummary> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*)::bigint as execution_count,
            COALESCE(SUM(execution_time_ms), 0)::bigint as total_execution_time_ms
        FROM mcp_tool_executions
        WHERE trace_id = $1
        "#,
    )
    .bind(trace_id)
    .fetch_one(&**pool)
    .await?;

    Ok(McpExecutionSummary {
        execution_count: row.get("execution_count"),
        total_execution_time_ms: row.get("total_execution_time_ms"),
    })
}

async fn fetch_mcp_execution_events(pool: &Arc<PgPool>, trace_id: &str) -> Result<Vec<TraceEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT
            started_at as timestamp,
            tool_name,
            mcp_server_name,
            execution_time_ms,
            status,
            user_id,
            session_id,
            task_id,
            context_id
        FROM mcp_tool_executions
        WHERE trace_id = $1
        ORDER BY started_at ASC
        "#,
    )
    .bind(trace_id)
    .fetch_all(&**pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let timestamp: DateTime<Utc> = row.get("timestamp");
        let tool_name: String = row.get("tool_name");
        let mcp_server_name: String = row.get("mcp_server_name");
        let execution_time_ms: Option<i32> = row.get("execution_time_ms");
        let status: String = row.get("status");

        let details = format!(
            "{}/{}: {} ({}ms)",
            mcp_server_name,
            tool_name,
            status,
            execution_time_ms.unwrap_or(0)
        );

        let metadata = json!({
            "execution_time_ms": execution_time_ms,
            "tool_name": tool_name,
            "mcp_server_name": mcp_server_name
        });

        events.push(TraceEvent {
            event_type: "MCP".to_string(),
            timestamp,
            details,
            user_id: row.get::<Option<String>, _>("user_id"),
            session_id: row.get::<Option<String>, _>("session_id"),
            task_id: row.get::<Option<String>, _>("task_id"),
            context_id: row.get::<Option<String>, _>("context_id"),
            metadata: Some(metadata.to_string()),
        });
    }

    Ok(events)
}

async fn fetch_task_id_for_trace(pool: &Arc<PgPool>, trace_id: &str) -> Result<Option<String>> {
    let row = sqlx::query("SELECT task_id FROM agent_tasks WHERE trace_id = $1 LIMIT 1")
        .bind(trace_id)
        .fetch_optional(&**pool)
        .await?;

    Ok(row.map(|r| r.get::<String, _>("task_id")))
}

async fn fetch_execution_step_summary(pool: &Arc<PgPool>, trace_id: &str) -> Result<ExecutionStep> {
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*)::bigint as step_count,
            COUNT(*) FILTER (WHERE s.status = 'completed')::bigint as completed_count,
            COUNT(*) FILTER (WHERE s.status = 'failed')::bigint as failed_count,
            COUNT(*) FILTER (WHERE s.status = 'pending' OR s.status = 'in_progress')::bigint as pending_count
        FROM task_execution_steps s
        JOIN agent_tasks t ON s.task_id = t.task_id
        WHERE t.trace_id = $1
        "#,
    )
    .bind(trace_id)
    .fetch_one(&**pool)
    .await?;

    Ok(ExecutionStep {
        step_count: row.get("step_count"),
        completed_count: row.get("completed_count"),
        failed_count: row.get("failed_count"),
        pending_count: row.get("pending_count"),
    })
}

async fn fetch_execution_step_events(
    pool: &Arc<PgPool>,
    trace_id: &str,
) -> Result<Vec<TraceEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT
            s.started_at as timestamp,
            s.content,
            s.status,
            s.duration_ms,
            t.user_id,
            t.session_id,
            t.task_id,
            t.context_id
        FROM task_execution_steps s
        JOIN agent_tasks t ON s.task_id = t.task_id
        WHERE t.trace_id = $1
        ORDER BY s.started_at ASC
        "#,
    )
    .bind(trace_id)
    .fetch_all(&**pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let timestamp: DateTime<Utc> = row.get("timestamp");
        let content: Value = row.get("content");
        let status: String = row.get("status");
        let duration_ms: Option<i32> = row.get("duration_ms");

        let step_type = content
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let tool_name = content.get("tool_name").and_then(|v| v.as_str());
        let skill_name = content.get("skill_name").and_then(|v| v.as_str());

        let details = match step_type {
            "understanding" => format!("[{}] Analyzing request... - {}", step_type, status),
            "planning" => format!("[{}] Planning response... - {}", step_type, status),
            "skill_usage" => {
                let name = skill_name.unwrap_or("unknown");
                format!("[{}] Using {} skill... - {}", step_type, name, status)
            },
            "tool_execution" => {
                let name = tool_name.unwrap_or("unknown");
                format!("[{}] Running {}... - {}", step_type, name, status)
            },
            "completion" => format!("[{}] Complete - {}", step_type, status),
            _ => format!("[{}] - {}", step_type, status),
        };

        let metadata = json!({
            "step_type": step_type,
            "status": status,
            "duration_ms": duration_ms,
            "tool_name": tool_name,
            "skill_name": skill_name
        });

        events.push(TraceEvent {
            event_type: "STEP".to_string(),
            timestamp,
            details,
            user_id: row.get::<Option<String>, _>("user_id"),
            session_id: row.get::<Option<String>, _>("session_id"),
            task_id: row.get::<Option<String>, _>("task_id"),
            context_id: row.get::<Option<String>, _>("context_id"),
            metadata: Some(metadata.to_string()),
        });
    }

    Ok(events)
}

fn print_formatted(
    events: &[TraceEvent],
    trace_id: &str,
    task_id: Option<&str>,
    verbose: bool,
    ai_summary: &AiRequestSummary,
    mcp_summary: &McpExecutionSummary,
    step_summary: &ExecutionStep,
) -> Result<()> {
    println!();
    println!(
        "{}",
        format!("Trace Flow: {}", trace_id).bright_cyan().bold()
    );
    println!("{}", "═".repeat(120).bright_blue());
    println!();

    let first_timestamp = events.first().map(|e| e.timestamp);
    let last_timestamp = events.last().map(|e| e.timestamp);

    if verbose {
        let mut prev_timestamp: Option<DateTime<Utc>> = None;
        for event in events {
            print_event(event, verbose, prev_timestamp);
            prev_timestamp = Some(event.timestamp);
        }
    } else {
        let mut prev_timestamp: Option<DateTime<Utc>> = None;
        let rows: Vec<TraceRow> = events
            .iter()
            .map(|e| {
                let time = e.timestamp.format("%H:%M:%S%.3f").to_string();
                let delta = if let Some(prev) = prev_timestamp {
                    let delta_ms = e.timestamp.signed_duration_since(prev).num_milliseconds();
                    format!("+{}ms", delta_ms)
                } else {
                    "+0ms".to_string()
                };
                prev_timestamp = Some(e.timestamp);

                let latency = extract_latency_from_metadata(&e.metadata, &e.event_type);

                TraceRow {
                    time,
                    delta,
                    event_type: e.event_type.clone(),
                    details: truncate_string(&e.details, 100),
                    latency,
                }
            })
            .collect();

        if !rows.is_empty() {
            let table = Table::new(rows).with(Style::modern()).to_string();
            println!("{}", table);
        }
    }

    println!();
    print_summary(
        events,
        first_timestamp,
        last_timestamp,
        task_id,
        ai_summary,
        mcp_summary,
        step_summary,
    );

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn format_metadata_value(key: &str, value: &Value) -> String {
    match key {
        "cost_cents" => {
            if let Some(microdollars) = value.as_i64() {
                let dollars = microdollars as f64 / 1_000_000.0;
                format!("${:.6}", dollars)
            } else {
                format!("{}", value).trim_matches('"').to_string()
            }
        },
        "latency_ms" | "execution_time_ms" => {
            if let Some(ms) = value.as_i64() {
                format!("{}ms", ms)
            } else {
                format!("{}", value).trim_matches('"').to_string()
            }
        },
        "tokens_used" => {
            if let Some(tokens) = value.as_i64() {
                format!("{}", tokens)
            } else {
                format!("{}", value).trim_matches('"').to_string()
            }
        },
        _ => format!("{}", value).trim_matches('"').to_string(),
    }
}

fn extract_latency_from_metadata(metadata: &Option<String>, event_type: &str) -> String {
    if let Some(ref meta) = metadata {
        if let Ok(parsed) = serde_json::from_str::<Value>(meta) {
            match event_type {
                "AI" => {
                    if let Some(latency) = parsed.get("latency_ms").and_then(|v| v.as_i64()) {
                        return format!("{}ms", latency);
                    }
                },
                "MCP" => {
                    if let Some(exec_time) =
                        parsed.get("execution_time_ms").and_then(|v| v.as_i64())
                    {
                        return format!("{}ms", exec_time);
                    }
                },
                _ => {},
            }
        }
    }
    "-".to_string()
}

fn print_event(event: &TraceEvent, verbose: bool, prev_timestamp: Option<DateTime<Utc>>) {
    let timestamp = event
        .timestamp
        .format("%H:%M:%S%.3f")
        .to_string()
        .bright_black();

    let delta = if let Some(prev) = prev_timestamp {
        let delta_ms = event
            .timestamp
            .signed_duration_since(prev)
            .num_milliseconds();
        format!("(+{}ms)", delta_ms).bright_black()
    } else {
        "(+0ms)".bright_black()
    };

    let (type_label, details_colored) = match event.event_type.as_str() {
        "LOG" => {
            let level_color = if event.details.starts_with("ERROR") {
                event.details.bright_red()
            } else if event.details.starts_with("WARN") {
                event.details.bright_yellow()
            } else if event.details.starts_with("INFO") {
                event.details.bright_green()
            } else if event.details.starts_with("DEBUG") {
                event.details.bright_cyan()
            } else {
                event.details.normal()
            };
            ("[LOG]   ".bright_white(), level_color)
        },
        "AI" => ("[AI]    ".bright_magenta(), event.details.bright_magenta()),
        "STEP" => ("[STEP]  ".bright_cyan(), event.details.bright_cyan()),
        "TASK" => ("[TASK]  ".bright_purple(), event.details.bright_purple()),
        "MESSAGE" => ("[MSG]   ".bright_cyan(), event.details.bright_cyan()),
        "MCP" => ("[MCP]   ".bright_yellow(), event.details.bright_yellow()),
        _ => ("[UNKNOWN]".normal(), event.details.normal()),
    };

    println!("{} {} {} {}", timestamp, delta, type_label, details_colored);

    if verbose {
        let mut context_parts = Vec::new();

        if let Some(ref session_id) = event.session_id {
            let len = session_id.len().min(12);
            context_parts.push(format!("session: {}", &session_id[..len]));
        }
        if let Some(ref user_id) = event.user_id {
            let len = user_id.len().min(12);
            context_parts.push(format!("user: {}", &user_id[..len]));
        }
        if let Some(ref task_id) = event.task_id {
            let len = task_id.len().min(12);
            context_parts.push(format!("task: {}", &task_id[..len]));
        }
        if let Some(ref context_id) = event.context_id {
            let len = context_id.len().min(12);
            context_parts.push(format!("context: {}", &context_id[..len]));
        }

        if !context_parts.is_empty() {
            println!("           {}", context_parts.join(" | ").bright_black());
        }

        if let Some(ref metadata) = event.metadata {
            if let Ok(parsed) = serde_json::from_str::<Value>(metadata) {
                if let Some(obj) = parsed.as_object() {
                    for (key, value) in obj {
                        if !value.is_null() {
                            let formatted_value = format_metadata_value(key, value);
                            println!(
                                "           {}: {}",
                                key.bright_black(),
                                formatted_value.bright_black()
                            );
                        }
                    }
                }
            }
        }
        println!();
    }
}

fn print_summary(
    events: &[TraceEvent],
    first: Option<DateTime<Utc>>,
    last: Option<DateTime<Utc>>,
    task_id: Option<&str>,
    ai_summary: &AiRequestSummary,
    mcp_summary: &McpExecutionSummary,
    step_summary: &ExecutionStep,
) {
    println!("{}", "─".repeat(80).bright_blue());
    println!("{}", "Summary".bright_cyan().bold());
    println!();

    if let (Some(first), Some(last)) = (first, last) {
        let duration = last.signed_duration_since(first);
        println!(
            "  Duration: {}ms",
            duration.num_milliseconds().to_string().bright_green()
        );
    }

    let mut event_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for event in events {
        *event_counts.entry(event.event_type.clone()).or_insert(0) += 1;
    }

    let mut count_vec: Vec<_> = event_counts.iter().collect();
    count_vec.sort_by_key(|&(k, _)| k);

    print!("  Events: {} (", events.len().to_string().bright_green());
    let event_parts: Vec<String> = count_vec
        .iter()
        .map(|(k, v)| format!("{} {}", v, k))
        .collect();
    print!("{}", event_parts.join(", "));
    println!(")");

    if ai_summary.request_count > 0 {
        println!();
        println!("  {} AI Requests:", "📊".bright_magenta());
        println!(
            "     Requests: {}",
            ai_summary.request_count.to_string().bright_green()
        );
        println!(
            "     Tokens: {} (in: {}, out: {})",
            ai_summary.total_tokens.to_string().bright_green(),
            ai_summary.total_input_tokens.to_string().bright_cyan(),
            ai_summary.total_output_tokens.to_string().bright_cyan()
        );
        let dollars = ai_summary.total_cost_cents as f64 / 1_000_000.0;
        println!("     Cost: {}", format!("${:.6}", dollars).bright_green());
        println!(
            "     Total Latency: {}ms",
            ai_summary.total_latency_ms.to_string().bright_yellow()
        );
        if ai_summary.request_count > 0 {
            let avg_latency = ai_summary.total_latency_ms / ai_summary.request_count;
            println!(
                "     Avg Latency: {}ms",
                avg_latency.to_string().bright_yellow()
            );
        }
    }

    if mcp_summary.execution_count > 0 {
        println!();
        println!("  {} MCP Tool Executions:", "🔧".bright_yellow());
        println!(
            "     Executions: {}",
            mcp_summary.execution_count.to_string().bright_green()
        );
        println!(
            "     Total Time: {}ms",
            mcp_summary
                .total_execution_time_ms
                .to_string()
                .bright_yellow()
        );
        if mcp_summary.execution_count > 0 {
            let avg_time = mcp_summary.total_execution_time_ms / mcp_summary.execution_count;
            println!("     Avg Time: {}ms", avg_time.to_string().bright_yellow());
        }
    }

    if step_summary.step_count > 0 {
        println!();
        println!("  {} Execution Steps:", "🔄".bright_cyan());
        println!(
            "     Steps: {} ({} completed, {} failed, {} pending)",
            step_summary.step_count.to_string().bright_green(),
            step_summary.completed_count.to_string().bright_green(),
            if step_summary.failed_count > 0 {
                step_summary.failed_count.to_string().bright_red()
            } else {
                step_summary.failed_count.to_string().bright_black()
            },
            step_summary.pending_count.to_string().bright_yellow()
        );
    }

    println!();

    if let Some(task_id) = task_id {
        println!(
            "  Task: {} {}",
            task_id.bright_yellow(),
            "(use: just ai-trace <task_id>)".bright_black()
        );
    }

    if let Some(session_id) = events.first().and_then(|e| e.session_id.as_ref()) {
        println!("  Session: {}", session_id.bright_yellow());
    }

    if let Some(user_id) = events.first().and_then(|e| e.user_id.as_ref()) {
        println!("  User: {}", user_id.bright_yellow());
    }

    let has_errors = events.iter().any(|e| {
        e.details.contains("ERROR")
            || e.details.contains("failed")
            || e.details.contains("(failed)")
    });

    if has_errors {
        println!("  Status: {}", "[FAILED]".bright_red());
    } else {
        println!("  Status: {}", "[OK]".bright_green());
    }

    println!();
}

fn print_json(
    events: &[TraceEvent],
    trace_id: &str,
    ai_summary: &AiRequestSummary,
    mcp_summary: &McpExecutionSummary,
    step_summary: &ExecutionStep,
) -> Result<()> {
    let json_events: Vec<Value> = events
        .iter()
        .map(|e| {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), Value::String(e.event_type.clone()));
            obj.insert(
                "timestamp".to_string(),
                Value::String(e.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
            );
            obj.insert("details".to_string(), Value::String(e.details.clone()));

            if let Some(ref user_id) = e.user_id {
                obj.insert("user_id".to_string(), Value::String(user_id.clone()));
            }
            if let Some(ref session_id) = e.session_id {
                obj.insert("session_id".to_string(), Value::String(session_id.clone()));
            }
            if let Some(ref task_id) = e.task_id {
                obj.insert("task_id".to_string(), Value::String(task_id.clone()));
            }
            if let Some(ref context_id) = e.context_id {
                obj.insert("context_id".to_string(), Value::String(context_id.clone()));
            }
            if let Some(ref metadata) = e.metadata {
                if let Ok(parsed) = serde_json::from_str::<Value>(metadata) {
                    obj.insert("metadata".to_string(), parsed);
                }
            }

            Value::Object(obj)
        })
        .collect();

    let output = serde_json::json!({
        "trace_id": trace_id,
        "events": json_events,
        "count": events.len(),
        "ai_summary": {
            "request_count": ai_summary.request_count,
            "total_tokens": ai_summary.total_tokens,
            "input_tokens": ai_summary.total_input_tokens,
            "output_tokens": ai_summary.total_output_tokens,
            "cost_cents": ai_summary.total_cost_cents,
            "total_latency_ms": ai_summary.total_latency_ms,
        },
        "mcp_summary": {
            "execution_count": mcp_summary.execution_count,
            "total_execution_time_ms": mcp_summary.total_execution_time_ms,
        },
        "step_summary": {
            "step_count": step_summary.step_count,
            "completed_count": step_summary.completed_count,
            "failed_count": step_summary.failed_count,
            "pending_count": step_summary.pending_count,
        }
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub async fn execute(trace_id: Option<&str>, options: TraceOptions) -> Result<()> {
    dotenvy::dotenv().ok();

    let ctx = AppContext::new().await?;
    let pool = ctx
        .db_pool()
        .pool_arc()
        .expect("Database must be PostgreSQL");

    let effective_trace_id = if let Some(id) = trace_id {
        id.to_string()
    } else {
        let base_url = std::env::var("API_EXTERNAL_URL")
            .or_else(|_| std::env::var("VITE_API_BASE_HOST"))
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        send_and_trace(&options, &base_url).await?
    };

    let (
        log_events,
        ai_events,
        mcp_events,
        step_events,
        ai_summary,
        mcp_summary,
        step_summary,
        task_id,
    ) = tokio::try_join!(
        fetch_trace_events(&pool, &effective_trace_id),
        fetch_ai_request_events(&pool, &effective_trace_id),
        fetch_mcp_execution_events(&pool, &effective_trace_id),
        fetch_execution_step_events(&pool, &effective_trace_id),
        fetch_ai_request_summary(&pool, &effective_trace_id),
        fetch_mcp_execution_summary(&pool, &effective_trace_id),
        fetch_execution_step_summary(&pool, &effective_trace_id),
        fetch_task_id_for_trace(&pool, &effective_trace_id),
    )?;

    // Filter log events - only show key events in non-verbose mode
    let filtered_log_events: Vec<TraceEvent> = if options.verbose {
        log_events
    } else {
        // By default, hide most logs but keep important ones
        log_events
            .into_iter()
            .filter(|e| {
                let event_type = e.event_type.to_lowercase();
                let details = e.details.to_lowercase();

                // Filter out DEBUG logs entirely in non-verbose mode
                if event_type == "debug" {
                    return false;
                }

                // Keep WARN and ERROR
                if event_type == "warn" || event_type == "error" {
                    return true;
                }

                // Filter out verbose noise first (before keeping important messages)
                if details.contains("broadcast execution_step")
                    || details.contains("resolved redirect")
                    || details.contains("resolving redirect")
                    || details.contains("artifacts count")
                    || details.contains("task.artifacts")
                    || details.contains("sending json")
                    || details.contains("received artifact")
                    || details.contains("received complete event")
                    || details.contains("setting task.artifacts")
                    || details.contains("sent complete event")
                {
                    return false;
                }

                // Keep key INFO messages that are important for understanding the flow
                if details.contains("🚀 starting")
                    || details.contains("agentic loop complete")
                    || details.contains("loop finished")
                    || details.contains("processing complete")
                    || details.contains("ai request")
                    || details.contains("tool executed")
                    || details.contains("iteration") && details.contains("starting")
                    || details.contains("decision:")
                    || details.contains("research complete")
                    || details.contains("synthesis complete")
                {
                    return true;
                }

                // In non-verbose mode, be more selective
                false
            })
            .collect()
    };

    let mut events = filtered_log_events;
    events.extend(ai_events);
    events.extend(mcp_events);
    events.extend(step_events);
    events.sort_by_key(|e| e.timestamp);

    if events.is_empty() {
        println!(
            "{}",
            format!("No events found for trace_id: {}", effective_trace_id).yellow()
        );
        println!();
        println!(
            "{}",
            "Tip: The trace may take a moment to populate. Try again in a few seconds."
                .bright_black()
        );
        return Ok(());
    }

    if options.json {
        print_json(
            &events,
            &effective_trace_id,
            &ai_summary,
            &mcp_summary,
            &step_summary,
        )?;
    } else {
        print_formatted(
            &events,
            &effective_trace_id,
            task_id.as_deref(),
            options.verbose,
            &ai_summary,
            &mcp_summary,
            &step_summary,
        )?;
    }

    Ok(())
}
