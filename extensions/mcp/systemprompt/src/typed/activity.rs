//! `user_activity` and `conversation_list`: what people did, by conversation.
//!
//! Requests are the wrong unit for "what has X been working on": a production
//! session paged one day of `request_log`, called it the busiest day, and
//! estimated hours per day from timestamps. These tools read the conversation
//! rollup (`conversation_facts`, `conversation_skill_facts`) instead, so one
//! call answers active days, turns per day, skills and titles for a window.
//! They run fixed read-only SQL through `infra db query`, so remote routing
//! and the CLI's `READ ONLY` transaction apply exactly as for the passthrough.

use super::{ConversationListInput, PagedOutput, UserActivityInput, page_limit, read_rows};
use crate::cli::CliLocation;
use crate::reports::invalid;
use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy)]
pub struct UserActivityHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct ConversationListHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

// Why: values are spliced into SQL text, so the accepted alphabet excludes
// every quote, backslash and statement separator rather than escaping them.
fn literal(name: &str, value: &str) -> Result<String, ErrorData> {
    let value = value.trim();
    let allowed = |c: char| c.is_alphanumeric() || " @._+-:".contains(c);
    if value.len() > 80 || !value.chars().all(allowed) {
        return Err(invalid(format!(
            "`{name}` may hold letters, digits, spaces and @._+-: only (got {value:?})"
        )));
    }
    Ok(value.to_owned())
}

pub fn window_bound(name: &str, value: &str, default: &str) -> Result<String, ErrorData> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(default.to_owned());
    }
    for (suffix, unit) in [("d", "days"), ("h", "hours"), ("w", "weeks")] {
        if let Some(digits) = value.strip_suffix(suffix)
            && !digits.is_empty()
            && digits.len() <= 4
            && digits.chars().all(|c| c.is_ascii_digit())
        {
            return Ok(format!("now() - interval '{digits} {unit}'"));
        }
    }
    let date = value.get(..10).unwrap_or_default();
    let shaped = date.len() == 10
        && date.char_indices().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        })
        && value[10..]
            .chars()
            .all(|c| c.is_ascii_digit() || "T:".contains(c));
    if !shaped {
        return Err(invalid(format!(
            "`{name}` must be `24h`, `7d`, `4w` or `YYYY-MM-DD[THH:MM:SS]` (got {value:?})"
        )));
    }
    Ok(format!("'{value}'::timestamptz"))
}

// Why: a person is named however the asker names them — id, email, or part
// of a name — and one question can match several people, who then each get
// a row rather than a guess.
fn person_match(alias: &str, user: &str) -> Result<String, ErrorData> {
    let v = literal("user", user)?;
    if v.is_empty() {
        return Ok("TRUE".to_owned());
    }
    Ok(format!(
        "({alias}.id = '{v}' OR {alias}.email = lower('{v}') \
         OR COALESCE({alias}.display_name, '') ILIKE '%{v}%' \
         OR COALESCE({alias}.full_name, '') ILIKE '%{v}%' OR {alias}.name ILIKE '%{v}%')"
    ))
}

// Why: the SQL is ~2 KB; echoing it back as `command` is width the model
// reads on every call and learns nothing from.
fn label(tool: &str, user: &str, since: &str) -> String {
    format!("{tool} user={:?} since={:?}", user.trim(), since.trim())
}

fn sql_command(sql: &str, limit: u16, offset: u16) -> String {
    let sql = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut command = format!(
        "infra db query {} --limit {limit}",
        shell_words::quote(&sql)
    );
    if offset > 0 {
        command.push_str(&format!(" --offset {offset}"));
    }
    command
}

pub fn user_activity_command(input: &UserActivityInput) -> Result<String, ErrorData> {
    let person = person_match("u", &input.user)?;
    let since = window_bound("since", &input.since, "now() - interval '30 days'")?;
    let until = window_bound("until", &input.until, "now()")?;
    let sql = format!(
        "WITH u AS (SELECT u.id, COALESCE(NULLIF(u.display_name, ''), u.full_name, u.name) AS person, \
           u.email, u.status FROM users u WHERE {person}), \
         f AS (SELECT f.* FROM conversation_facts f JOIN u ON u.id = f.user_id \
           WHERE f.first_at >= {since} AND f.first_at < {until}), \
         d AS (SELECT user_id, first_at::date AS day, COUNT(*) AS n, SUM(turn_count) AS turns \
           FROM f GROUP BY 1, 2), \
         s AS (SELECT sf.user_id, sf.skill, SUM(sf.invocations) AS n FROM conversation_skill_facts sf \
           JOIN f ON f.context_id = sf.context_id GROUP BY 1, 2), \
         t AS (SELECT user_id, first_at, LEFT(conversation_title(context_id, client_session_id), 120) AS title, \
           row_number() OVER (PARTITION BY user_id ORDER BY turn_count DESC, first_at DESC) AS rn FROM f) \
         SELECT u.id AS user_id, u.person AS name, u.email, u.status, \
           COUNT(f.context_id) AS conversations, \
           (SELECT COUNT(*) FROM d WHERE d.user_id = u.id) AS active_days, \
           MIN(f.first_at) AS first_seen, MAX(f.last_at) AS last_seen, \
           COALESCE(SUM(f.turn_count), 0) AS turns, COALESCE(SUM(f.request_count), 0) AS requests, \
           COALESCE(SUM(f.cost_microdollars), 0) AS cost_microdollars, \
           COALESCE(SUM(f.error_count), 0) AS errors, \
           COALESCE(SUM(f.gov_deny + f.rejected_count), 0) AS denied, \
           (SELECT string_agg(to_char(day, 'YYYY-MM-DD') || ': ' || n || ' conv, ' || turns || ' turns', \
             '; ' ORDER BY day) FROM d WHERE d.user_id = u.id) AS per_day, \
           (SELECT string_agg(skill || ' x' || n, ', ' ORDER BY n DESC) FROM s WHERE s.user_id = u.id) AS skills, \
           (SELECT string_agg(to_char(first_at, 'MM-DD') || ' ' || title, ' | ' ORDER BY first_at DESC) \
             FROM t WHERE t.user_id = u.id AND t.rn <= 10) AS main_conversations, \
           (SELECT MAX(refreshed_at) FROM conversation_facts) AS facts_as_of \
         FROM u LEFT JOIN f ON f.user_id = u.id \
         GROUP BY u.id, u.person, u.email, u.status \
         ORDER BY COUNT(f.context_id) DESC, u.person"
    );
    Ok(sql_command(&sql, page_limit(input.limit, 20), 0))
}

pub fn conversation_list_command(input: &ConversationListInput) -> Result<String, ErrorData> {
    let person = person_match("u", &input.user)?;
    let since = window_bound("since", &input.since, "now() - interval '7 days'")?;
    let until = window_bound("until", &input.until, "now()")?;
    let skill = literal("skill", &input.skill)?;
    let skill_filter = if skill.is_empty() {
        String::new()
    } else {
        format!(
            " AND array_to_string(f.skills, ',') ILIKE '%{}%'",
            skill.replace('_', "-")
        )
    };
    let sql = format!(
        "SELECT f.context_id, \
           (SELECT r.id FROM ai_requests r WHERE r.context_id = f.context_id \
             ORDER BY r.created_at DESC LIMIT 1) AS request_id, \
           COALESCE(NULLIF(u.display_name, ''), u.full_name, u.name) AS name, \
           LEFT(conversation_title(f.context_id, f.client_session_id), 160) AS title, \
           f.first_at AS started, ROUND(f.duration_seconds / 60.0) AS minutes, \
           f.turn_count AS turns, f.request_count AS requests, \
           array_to_string(f.skills, ', ') AS skills, f.cost_microdollars, \
           f.error_count AS errors, f.gov_deny + f.rejected_count AS denied, f.client_kind \
         FROM conversation_facts f JOIN users u ON u.id = f.user_id \
         WHERE {person} AND f.first_at >= {since} AND f.first_at < {until}{skill_filter} \
         ORDER BY f.first_at DESC, f.context_id"
    );
    Ok(sql_command(&sql, page_limit(input.limit, 25), input.offset))
}

impl McpToolHandler for UserActivityHandler<'_> {
    type Input = UserActivityInput;
    type Output = PagedOutput;
    fn tool_name(&self) -> &'static str {
        "user_activity"
    }
    fn description(&self) -> &'static str {
        "Who did what, per person, from conversations (not requests): one row per matching user with conversations, active_days, first/last seen, per_day (conversations and turns each day), skills used, main_conversations (date + title), turns, requests, cost_usd, errors and denied. `user` is an id, an email or part of a name (`jay`, `Meenakshi`); empty lists every user with or without activity. Start here for \"what has X been working on\", team usage and \"how intensively\". Nothing here measures hours worked: a conversation can stay open for days. `facts_as_of` is when the rollup last ran; newer activity is not in it yet."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: UserActivityInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(PagedOutput, String), ErrorData> {
        let command = user_activity_command(&input)?;
        let rows = read_rows(self.cli, self.token, &command).await?;
        let full = rows.len() >= usize::from(page_limit(input.limit, 20));
        let mut output = PagedOutput::new(label("user_activity", &input.user, &input.since), rows);
        output.fit_to_budget(super::MAX_OUTPUT_BYTES);
        output.truncated |= full;
        output.hint = if output.truncated {
            "More people match: narrow `user` or raise `limit`.".to_owned()
        } else {
            "Every matching user is listed, active or not. Use conversation_list to see a person's conversations one by one.".to_owned()
        };
        let summary = output.summary();
        Ok((output, summary))
    }
}

impl McpToolHandler for ConversationListHandler<'_> {
    type Input = ConversationListInput;
    type Output = PagedOutput;
    fn tool_name(&self) -> &'static str {
        "conversation_list"
    }
    fn description(&self) -> &'static str {
        "One row per conversation, newest first: title (the AI title or opening prompt), who, started, minutes, turns, requests, skills, cost_usd, errors, denied, context_id and request_id. Filter by `user` (id, email or part of a name), `skill` and a `since`/`until` window; page with `offset`. Pass a row's `request_id` to `conversation_audit` to read that conversation."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: ConversationListInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(PagedOutput, String), ErrorData> {
        let command = conversation_list_command(&input)?;
        let rows = read_rows(self.cli, self.token, &command).await?;
        let limit = page_limit(input.limit, 25);
        let mut output =
            PagedOutput::new(label("conversation_list", &input.user, &input.since), rows);
        let full = output.returned >= usize::from(limit);
        let dropped = output.fit_to_budget(super::MAX_OUTPUT_BYTES);
        if full || dropped > 0 {
            let kept = u16::try_from(output.returned).unwrap_or(limit);
            output.next_cursor = input.offset.saturating_add(kept).to_string();
            "More conversations exist: call again with `offset` set to `next_cursor`, or use user_activity for totals."
                .clone_into(&mut output.hint);
        } else {
            "Last page for these filters.".clone_into(&mut output.hint);
        }
        let summary = output.summary();
        Ok((output, summary))
    }
}
