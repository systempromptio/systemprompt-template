//! `conversation_audit`: one request's audit record with a bounded page of
//! its transcript.

use super::{ConversationAuditInput, flag_value, page_limit, read_value};
use crate::cli::CliLocation;
use rmcp::ErrorData;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// JSON: the audit card's fields are the CLI's own values.
use serde_json::Value;
use std::collections::BTreeMap;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::{McpOutputSchema, McpToolHandler};
use systemprompt::models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy)]
pub struct ConversationAuditHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

/// The audit card folded back into an object, plus what to ask for next.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AuditOutput {
    pub command: String,
    pub fields: BTreeMap<String, Value>,
    pub has_more: bool,
    // Why: the next `offset` to request when `has_more`, the current one
    // otherwise, so the caller never has to add `limit` itself.
    pub next_offset: u16,
    pub hint: String,
}

impl McpOutputSchema for AuditOutput {
    fn artifact_type() -> &'static str {
        "presentation_card"
    }
    fn artifact_title(&self) -> Option<String> {
        Some(self.command.clone())
    }
}

pub fn command(input: &ConversationAuditInput) -> Result<String, ErrorData> {
    let id = flag_value("request_id", &input.request_id)?;
    if id.is_empty() {
        return Err(crate::reports::invalid("`request_id` is required"));
    }
    let mut command = format!(
        "infra logs audit {id} --offset {} --limit {} --max-content {}",
        input.offset,
        audit_page(input.limit),
        body_chars(input.max_chars)
    );
    if input.messages {
        command.push_str(" --messages");
    }
    if input.tools {
        command.push_str(" --tools");
    }
    Ok(command)
}

// Why: `--max-content 0` is the whole transcript body, which for one long
// agent turn is hundreds of KB; with the page cap a full page stays near
// 50 KB, which a model can read.
pub(super) const MAX_BODY_CHARS: u16 = 2000;
pub(super) const MAX_AUDIT_PAGE: u16 = 25;

fn audit_page(requested: u16) -> u16 {
    page_limit(requested, 20).min(MAX_AUDIT_PAGE)
}

fn body_chars(requested: u16) -> u16 {
    if requested == 0 {
        MAX_BODY_CHARS
    } else {
        requested.min(MAX_BODY_CHARS)
    }
}

impl McpToolHandler for ConversationAuditHandler<'_> {
    type Input = ConversationAuditInput;
    type Output = AuditOutput;
    fn tool_name(&self) -> &'static str {
        "conversation_audit"
    }
    fn description(&self) -> &'static str {
        "Read one AI request's audit record: user, model served vs requested, tokens, cost, latency, and a paged, length-bounded slice of its messages (and tool calls on request). `message_count` and `tool_call_count` are the totals; `messages` holds `limit` rows from `offset`, each body cut to `max_chars`. Use small pages to judge prompt quality without loading a whole transcript. Takes a `request_id` from `request_log`."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: ConversationAuditInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(AuditOutput, String), ErrorData> {
        let command = command(&input)?;
        let value = read_value(self.cli, self.token, &command).await?;
        if value.get("lines").is_some() {
            return Err(crate::reports::invalid(format!(
                "No AI request found for `{}`",
                input.request_id.trim()
            )));
        }
        let fields = super::card_object(&value);
        let has_more = fields
            .get("has_more")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let next_offset = if has_more {
            input.offset.saturating_add(audit_page(input.limit))
        } else {
            input.offset
        };
        let counts = format!(
            "{} messages, {} tool calls",
            fields.get("message_count").cloned().unwrap_or(Value::Null),
            fields
                .get("tool_call_count")
                .cloned()
                .unwrap_or(Value::Null)
        );
        let hint = if has_more {
            format!("{counts}; more rows: call again with `offset` = {next_offset}.")
        } else {
            format!("{counts}; this page reaches the end of what was requested.")
        };
        let summary = format!("Audit of `{}`: {hint}", input.request_id.trim());
        Ok((
            AuditOutput {
                command,
                fields,
                has_more,
                next_offset,
                hint,
            },
            summary,
        ))
    }
}
