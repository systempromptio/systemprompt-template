//! Typed admin-analytics tools over the `systemprompt` CLI.
//!
//! The free-form `systemprompt` tool takes one command string, so a model has
//! to discover flags by trial: a production session spent fifteen calls on
//! `--help`, invented `--until` and `--export`, never found the `--user`
//! filter that existed, and then pulled a 400 KB audit it could not read.
//! These tools carry the flags in their JSON schema instead, page with
//! a cursor or offset, and bound what comes back. They shell out to the same
//! CLI as the passthrough so auth-token forwarding, remote routing, access
//! audit and the reporting-privacy projection are unchanged.
//!
//! Every input is a flat struct: `String` fields default to `""` and integer
//! fields to a number, never `Option`, because schemars renders an `Option`
//! as a type list and Gemini refuses a function declaration that holds one.

mod activity;
mod conversation_audit;
mod inputs;
mod request_log;
mod usage_by_user;
mod users;

pub use activity::{ConversationListHandler, UserActivityHandler};
pub use conversation_audit::ConversationAuditHandler;
pub use inputs::{
    ConversationAuditInput, ConversationListInput, RequestLogInput, UsageByUserInput,
    UserActivityInput, UsersInput,
};
pub use request_log::RequestLogHandler;
pub use usage_by_user::UsageByUserHandler;
pub use users::UsersHandler;

// Why: the argv each tool builds is its contract with the CLI; exposed behind
// `#[doc(hidden)]` so the external test workspace can pin it without a
// binary in the path. Not part of the public API.
#[doc(hidden)]
pub use activity::{conversation_list_command, user_activity_command, window_bound};
#[doc(hidden)]
pub use conversation_audit::command as conversation_audit_command;
#[doc(hidden)]
pub use request_log::command as request_log_command;
#[doc(hidden)]
pub use usage_by_user::command as usage_by_user_command;
#[doc(hidden)]
pub use users::command as users_command;

use crate::cli::{self, CliLocation};
use crate::reports::{failure, invalid, normalize_cli, with_dollar_siblings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// JSON: rows are the CLI's own row objects, heterogeneous by command.
use serde_json::Value;
use std::collections::BTreeMap;
use systemprompt::mcp::McpOutputSchema;

// Why: a request for more than this is clamped, not refused, and the output
// says so — a refusal is one more error the model has to reason around.
pub const MAX_PAGE: u16 = 100;

// Why: rows are ~1 KB of JSON each and a host folds the whole result into the
// model's context (pretty-printed for clients without structured content),
// so a row count alone is no bound: 200 wide request rows filled most of a
// 200k-token window. Past this serialized size trailing rows are dropped and
// the page ends early with a cursor to the rest.
pub const MAX_OUTPUT_BYTES: usize = 48 * 1024;

const MICRODOLLARS_SUFFIX: &str = "_microdollars";

/// One page of rows from a CLI list command, with what a caller needs to ask
/// for the next one.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PagedOutput {
    pub command: String,
    pub columns: Vec<String>,
    pub rows: Vec<BTreeMap<String, Value>>,
    pub returned: usize,
    pub truncated: bool,
    // Why: a cursor rather than a page number — the caller passes it back as
    // `cursor` (or as `offset` for offset-paged tools); empty means the tool
    // knows of no later page.
    pub next_cursor: String,
    pub hint: String,
}

impl PagedOutput {
    #[must_use]
    pub fn new(command: String, rows: Vec<Value>) -> Self {
        let rows: Vec<BTreeMap<String, Value>> = rows
            .into_iter()
            .map(|row| match row {
                Value::Object(map) => without_raw_costs(map.into_iter().collect()),
                other => BTreeMap::from([("value".to_owned(), other)]),
            })
            .collect();
        let columns = rows
            .iter()
            .flat_map(|row| row.keys().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        Self {
            command,
            columns,
            returned: rows.len(),
            rows,
            truncated: false,
            next_cursor: String::new(),
            hint: String::new(),
        }
    }

    // Why: a page must fit a model's context whatever its row width, so
    // trailing rows are dropped until the serialized output fits.
    pub fn fit_to_budget(&mut self, max_bytes: usize) -> usize {
        let received = self.rows.len();
        while !self.rows.is_empty() && self.wire_bytes() > max_bytes {
            let keep = self.rows.len() * 3 / 4;
            self.rows.truncate(keep);
            self.returned = self.rows.len();
        }
        let dropped = received - self.rows.len();
        if dropped > 0 {
            self.truncated = true;
        }
        dropped
    }

    fn wire_bytes(&self) -> usize {
        serde_json::to_vec(self).map_or(0, |bytes| bytes.len())
    }

    pub(crate) fn summary(&self) -> String {
        let mut text = format!("{} rows from `{}`", self.returned, self.command);
        if !self.next_cursor.is_empty() {
            text.push_str(&format!("; next_cursor={}", self.next_cursor));
        }
        if self.truncated {
            text.push_str("; truncated");
        }
        text
    }
}

impl McpOutputSchema for PagedOutput {
    // Why: rows of pre-rendered cells plus a cursor is not the core `table`
    // model (`items` + typed `columns`); see `ReportOutput::artifact_type`.
    fn artifact_type() -> &'static str {
        "paged_table"
    }
    fn artifact_title(&self) -> Option<String> {
        Some(self.command.clone())
    }
}

// Why: `with_dollar_siblings` adds `<x>_usd` beside every `<x>_microdollars`
// and the tools tell the model to quote the dollars, so the raw integer is
// only width.
fn without_raw_costs(mut row: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    let raw: Vec<String> = row
        .keys()
        .filter(|key| {
            key.strip_suffix(MICRODOLLARS_SUFFIX)
                .is_some_and(|prefix| row.contains_key(&format!("{prefix}_usd")))
        })
        .cloned()
        .collect();
    for key in raw {
        row.remove(&key);
    }
    row
}

pub(crate) fn page_limit(requested: u16, default: u16) -> u16 {
    if requested == 0 {
        default
    } else {
        requested.min(MAX_PAGE)
    }
}

// Why: a value with whitespace or a leading dash would become a second flag
// once the command string is split again; refusing it keeps one input field
// equal to one argv token.
pub(crate) fn flag_value(name: &str, value: &str) -> Result<String, rmcp::ErrorData> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    if value.chars().any(|c| c.is_whitespace() || c.is_control()) || value.starts_with('-') {
        return Err(invalid(format!(
            "`{name}` must be a single value without spaces (got {value:?})"
        )));
    }
    Ok(shell_words::quote(value).into_owned())
}

pub(crate) async fn read_rows(
    location: &CliLocation,
    token: &str,
    command: &str,
) -> Result<Vec<Value>, rmcp::ErrorData> {
    let value = read_value(location, token, command).await?;
    // Why: an empty list is a message artifact ("No AI requests found"), and
    // zero rows is the right answer to it, not an envelope error.
    if value.get("lines").is_some() {
        return Ok(Vec::new());
    }
    Ok(with_dollar_siblings(normalize_cli(&value)?))
}

pub(crate) async fn read_value(
    location: &CliLocation,
    token: &str,
    command: &str,
) -> Result<Value, rmcp::ErrorData> {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        cli::execute(location, command, token),
    )
    .await
    .map_err(|_error| failure(format!("`{command}` timed out")))??;
    if !result.success {
        return Err(failure(format!(
            "`{command}` failed with exit {}: {}",
            result.exit_code,
            result.stderr.trim()
        )));
    }
    serde_json::from_str(&result.stdout)
        .map_err(|_error| failure(format!("`{command}` returned malformed JSON")))
}

// Why: `card_value` splits an object into heading/content sections for the
// terminal; folding them back gives the caller `message_count`, not a section
// whose heading is `message_count`.
pub(crate) fn card_object(value: &Value) -> BTreeMap<String, Value> {
    value
        .get("sections")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|section| {
            let heading = section.get("heading")?.as_str()?;
            Some((heading.to_owned(), section.get("content")?.clone()))
        })
        .collect()
}
