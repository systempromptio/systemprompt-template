//! `usage_by_user`: spend, requests, tokens and conversations per user.

use super::{PagedOutput, UsageByUserInput, flag_value, page_limit, read_rows};
use crate::cli::CliLocation;
use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy)]
pub struct UsageByUserHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

pub fn command(input: &UsageByUserInput) -> Result<String, ErrorData> {
    let mut command = format!(
        "analytics costs breakdown --by user --since {} --limit {}",
        flag_value("since", &input.since)?,
        page_limit(input.limit, 50)
    );
    let until = flag_value("until", &input.until)?;
    if !until.is_empty() {
        command.push_str(&format!(" --until {until}"));
    }
    Ok(command)
}

impl McpToolHandler for UsageByUserHandler<'_> {
    type Input = UsageByUserInput;
    type Output = PagedOutput;
    fn tool_name(&self) -> &'static str {
        "usage_by_user"
    }
    fn description(&self) -> &'static str {
        "Rank this platform's users by AI spend over a window: cost, request count, tokens and distinct conversations per user, highest spend first. The `name` column is `<user_id> (<display name>)`; pass the id to `request_log` to see that user's requests. Costs are in `cost_usd`. Emails are not available here — use `users` for the roster. Spend only: for what people did, active days and skills use `user_activity`. It excludes deleted users, so its totals can be below a sum of `request_log` rows; report both figures rather than explaining the gap."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: UsageByUserInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(PagedOutput, String), ErrorData> {
        let command = command(&input)?;
        let rows = read_rows(self.cli, self.token, &command).await?;
        let limit = usize::from(page_limit(input.limit, 50));
        let mut output = PagedOutput::new(command, rows);
        let full = output.returned >= limit;
        output.fit_to_budget(super::MAX_OUTPUT_BYTES);
        output.truncated |= full;
        output.hint = if output.truncated {
            "The page is full; raise `limit` (max 100) or narrow `since`/`until` to see every user."
                .to_owned()
        } else {
            "Every user with spend in the window is listed.".to_owned()
        };
        let summary = output.summary();
        Ok((output, summary))
    }
}
