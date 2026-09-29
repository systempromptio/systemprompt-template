//! `users`: the registered roster, offset-paged.

use super::{PagedOutput, UsersInput, flag_value, page_limit, read_rows};
use crate::cli::CliLocation;
use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy)]
pub struct UsersHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

pub fn command(input: &UsersInput) -> Result<String, ErrorData> {
    let mut command = format!(
        "admin users list --limit {} --offset {}",
        page_limit(input.limit, 50),
        input.offset
    );
    for (flag, value) in [("role", &input.role), ("status", &input.status)] {
        let value = flag_value(flag, value)?;
        if !value.is_empty() {
            command.push_str(&format!(" --{flag} {value}"));
        }
    }
    Ok(command)
}

impl McpToolHandler for UsersHandler<'_> {
    type Input = UsersInput;
    type Output = PagedOutput;
    fn tool_name(&self) -> &'static str {
        "users"
    }
    fn description(&self) -> &'static str {
        "List registered users with id, name, email, roles, status and creation date, offset-paged. Use it to map a `user_id` from `usage_by_user` or `request_log` to a person. Read-only; role changes go through the `systemprompt` tool (`admin users role promote|demote <id>`)."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: UsersInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(PagedOutput, String), ErrorData> {
        let command = command(&input)?;
        let rows = read_rows(self.cli, self.token, &command).await?;
        let limit = page_limit(input.limit, 50);
        let mut output = PagedOutput::new(command, rows);
        let full = output.returned >= usize::from(limit);
        let dropped = output.fit_to_budget(super::MAX_OUTPUT_BYTES);
        let paged = input.role.trim().is_empty() && (full || dropped > 0);
        if paged {
            let kept = u16::try_from(output.returned).unwrap_or(limit);
            output.next_cursor = input.offset.saturating_add(kept).to_string();
        }
        output.hint.push_str(if paged {
            "More users may follow; call again with `offset` set to `next_cursor`."
        } else {
            "Last page."
        });
        let summary = output.summary();
        Ok((output, summary))
    }
}
