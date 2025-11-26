use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use colored::Colorize;
use serde_json::Value;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_core_system::models::AppContext;
use tabled::{settings::Style, Table, Tabled};

#[derive(Args)]
pub struct TraceOptions {
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
    #[tabled(rename = "Type")]
    event_type: String,
    #[tabled(rename = "Details")]
    details: String,
    #[tabled(rename = "Session")]
    session: String,
    #[tabled(rename = "User")]
    user: String,
}

async fn fetch_trace_events(db: &dyn DatabaseProvider, trace_id: &str) -> Result<Vec<TraceEvent>> {
    let query = DatabaseQueryEnum::FetchTraceEvents.get(db);

    let rows = db
        .fetch_all(
            &query,
            &[
                &trace_id, // logs
                &trace_id, // ai_requests
                &trace_id, // agent_tasks
                &trace_id, // task_messages
                &trace_id, // mcp_tool_executions
            ],
        )
        .await?;

    let mut events = Vec::new();
    for row in rows {
        let timestamp = row
            .get("timestamp")
            .and_then(|v| systemprompt_core_database::parse_database_datetime(v))
            .unwrap_or_else(Utc::now);

        events.push(TraceEvent {
            event_type: row
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string(),
            timestamp,
            details: row
                .get("details")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_id: row
                .get("user_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            session_id: row
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            task_id: row
                .get("task_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            context_id: row
                .get("context_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            metadata: row
                .get("metadata")
                .and_then(|v| v.as_str())
                .map(String::from),
        });
    }

    Ok(events)
}

fn print_formatted(events: &[TraceEvent], trace_id: &str, verbose: bool) -> Result<()> {
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
        for event in events {
            print_event(event, verbose);
        }
    } else {
        let rows: Vec<TraceRow> = events
            .iter()
            .map(|e| {
                let time = e.timestamp.format("%H:%M:%S%.3f").to_string();
                let session = e
                    .session_id
                    .as_ref()
                    .map(|s| format!("{}...", &s[..s.len().min(8)]))
                    .unwrap_or_else(|| "-".to_string());
                let user = e
                    .user_id
                    .as_ref()
                    .map(|u| format!("{}...", &u[..u.len().min(8)]))
                    .unwrap_or_else(|| "-".to_string());

                TraceRow {
                    time,
                    event_type: e.event_type.clone(),
                    details: truncate_string(&e.details, 60),
                    session,
                    user,
                }
            })
            .collect();

        if !rows.is_empty() {
            let table = Table::new(rows).with(Style::modern()).to_string();
            println!("{}", table);
        }
    }

    println!();
    print_summary(events, first_timestamp, last_timestamp);

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn print_event(event: &TraceEvent, verbose: bool) {
    let timestamp = event
        .timestamp
        .format("%H:%M:%S%.3f")
        .to_string()
        .bright_black();

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
        "TASK" => ("[TASK]  ".bright_purple(), event.details.bright_purple()),
        "MESSAGE" => ("[MSG]   ".bright_cyan(), event.details.bright_cyan()),
        "MCP" => ("[MCP]   ".bright_yellow(), event.details.bright_yellow()),
        _ => ("[UNKNOWN]".normal(), event.details.normal()),
    };

    println!("{} {} {}", timestamp, type_label, details_colored);

    if verbose {
        let mut context_parts = Vec::new();

        if let Some(ref session_id) = event.session_id {
            context_parts.push(format!(
                "session: {}",
                &session_id[..session_id.len().min(12)]
            ));
        }
        if let Some(ref user_id) = event.user_id {
            context_parts.push(format!("user: {}", &user_id[..user_id.len().min(12)]));
        }
        if let Some(ref task_id) = event.task_id {
            context_parts.push(format!("task: {}", &task_id[..task_id.len().min(12)]));
        }
        if let Some(ref context_id) = event.context_id {
            context_parts.push(format!(
                "context: {}",
                &context_id[..context_id.len().min(12)]
            ));
        }

        if !context_parts.is_empty() {
            println!("           {}", context_parts.join(" | ").bright_black());
        }

        if let Some(ref metadata) = event.metadata {
            if let Ok(parsed) = serde_json::from_str::<Value>(metadata) {
                if let Some(obj) = parsed.as_object() {
                    for (key, value) in obj {
                        if !value.is_null() {
                            println!(
                                "           {}: {}",
                                key.bright_black(),
                                format!("{}", value).trim_matches('"').bright_black()
                            );
                        }
                    }
                }
            }
        }
        println!();
    }
}

fn print_summary(events: &[TraceEvent], first: Option<DateTime<Utc>>, last: Option<DateTime<Utc>>) {
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
        println!("  Status: {}", "✗ Errors detected".bright_red());
    } else {
        println!("  Status: {}", "✓ Success".bright_green());
    }

    println!();
}

fn print_json(events: &[TraceEvent]) -> Result<()> {
    let json_events: Vec<Value> = events
        .iter()
        .map(|e| {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), Value::String(e.event_type.clone()));
            obj.insert(
                "timestamp".to_string(),
                Value::String(e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
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
        "events": json_events,
        "count": events.len(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

pub async fn execute(trace_id: &str, options: TraceOptions) -> Result<()> {
    dotenvy::dotenv().ok();

    let ctx = AppContext::new().await?;
    let db_pool = ctx.db_pool();

    let events = fetch_trace_events(db_pool.as_ref(), trace_id).await?;

    if events.is_empty() {
        println!(
            "{}",
            format!("No events found for trace_id: {}", trace_id).yellow()
        );
        return Ok(());
    }

    if options.json {
        print_json(&events)?;
    } else {
        print_formatted(&events, trace_id, options.verbose)?;
    }

    Ok(())
}
