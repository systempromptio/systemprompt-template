//! Input contracts for the typed admin-analytics tools.
//!
//! Flat structs only: every field is a `String` (empty = unset) or an
//! integer with a default. See the module head of `typed` for why no field
//! is an `Option` and no enum is referenced through `$defs`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const fn zero() -> u16 {
    0
}

const fn fifty() -> u16 {
    50
}

const fn twenty() -> u16 {
    20
}

fn seven_days() -> String {
    "7d".to_owned()
}

fn thirty_days() -> String {
    "30d".to_owned()
}

const fn yes() -> bool {
    true
}

const fn eight_hundred() -> u16 {
    800
}

/// Spend, requests, tokens and distinct conversations per user.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageByUserInput {
    #[schemars(
        description = "Window start: a duration back from now (`1h`, `24h`, `7d`, `30d`) or an absolute `YYYY-MM-DD` / `YYYY-MM-DDTHH:MM:SS`. Defaults to `7d`."
    )]
    #[serde(default = "seven_days")]
    pub since: String,
    #[schemars(description = "Window end, same formats as `since`, exclusive. Empty means now.")]
    #[serde(default)]
    pub until: String,
    #[schemars(description = "Rows to return, highest spend first. 1-100; defaults to 50.")]
    #[serde(default = "fifty")]
    pub limit: u16,
}

/// Individual AI requests, newest first, filterable by user and pageable by
/// cursor.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RequestLogInput {
    #[schemars(
        description = "Window start: a duration back from now (`1h`, `24h`, `7d`) or an absolute `YYYY-MM-DD` / `YYYY-MM-DDTHH:MM:SS`. Defaults to `30d`."
    )]
    #[serde(default = "thirty_days")]
    pub since: String,
    #[schemars(description = "Window end, same formats as `since`, exclusive. Empty means now.")]
    #[serde(default)]
    pub until: String,
    #[schemars(
        description = "Exact user id (the `user_id` column of `usage_by_user` rows, before the display name). Empty means every user."
    )]
    #[serde(default)]
    pub user: String,
    #[schemars(description = "Model name substring, e.g. `opus`. Empty means every model.")]
    #[serde(default)]
    pub model: String,
    #[schemars(description = "Provider substring, e.g. `anthropic`. Empty means every provider.")]
    #[serde(default)]
    pub provider: String,
    #[schemars(description = "Rows per page. 1-100; defaults to 50.")]
    #[serde(default = "fifty")]
    pub limit: u16,
    #[schemars(
        description = "The `next_cursor` from the previous page; returns strictly older rows. Empty starts from the newest."
    )]
    #[serde(default)]
    pub cursor: String,
}

/// One request's audit record: identity, model, tokens, cost, and a bounded
/// page of its conversation.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConversationAuditInput {
    #[schemars(
        description = "AI request id, task id, or trace id (a `request_id` from `request_log`)."
    )]
    pub request_id: String,
    #[schemars(description = "Include the conversation messages. Defaults to true.")]
    #[serde(default = "yes")]
    pub messages: bool,
    #[schemars(description = "Include the tool calls. Defaults to false.")]
    #[serde(default)]
    pub tools: bool,
    #[schemars(description = "Messages / tool calls to skip before this page. Defaults to 0.")]
    #[serde(default = "zero")]
    pub offset: u16,
    #[schemars(description = "Messages / tool calls per page. 1-25; defaults to 20.")]
    #[serde(default = "twenty")]
    pub limit: u16,
    #[schemars(
        description = "Truncate each message body and tool input to this many characters, at most 2000. 0 means the maximum; defaults to 800."
    )]
    #[serde(default = "eight_hundred")]
    pub max_chars: u16,
}

const fn twenty_five() -> u16 {
    25
}

/// One row per person: conversations, active days, skills and titles.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UserActivityInput {
    #[schemars(
        description = "A user id, an email, or part of a display name (`jay`, `Meenakshi Ruia`). Empty means every user."
    )]
    #[serde(default)]
    pub user: String,
    #[schemars(
        description = "Window start: `24h`, `7d`, `4w`, `30d` back from now, or `YYYY-MM-DD[THH:MM:SS]`. Defaults to `30d`."
    )]
    #[serde(default = "thirty_days")]
    pub since: String,
    #[schemars(description = "Window end, same formats as `since`, exclusive. Empty means now.")]
    #[serde(default)]
    pub until: String,
    #[schemars(description = "People to return, most conversations first. 1-100; defaults to 20.")]
    #[serde(default = "twenty")]
    pub limit: u16,
}

/// Conversations newest first, filterable by person and skill.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConversationListInput {
    #[schemars(
        description = "A user id, an email, or part of a display name. Empty means every user."
    )]
    #[serde(default)]
    pub user: String,
    #[schemars(
        description = "Window start: `24h`, `7d`, `4w`, `30d` back from now, or `YYYY-MM-DD[THH:MM:SS]`. Defaults to `7d`."
    )]
    #[serde(default = "seven_days")]
    pub since: String,
    #[schemars(description = "Window end, same formats as `since`, exclusive. Empty means now.")]
    #[serde(default)]
    pub until: String,
    #[schemars(
        description = "Only conversations that used a skill whose name contains this, e.g. `deal-review`. Empty means any."
    )]
    #[serde(default)]
    pub skill: String,
    #[schemars(description = "Rows per page. 1-100; defaults to 25.")]
    #[serde(default = "twenty_five")]
    pub limit: u16,
    #[schemars(description = "Rows to skip (the previous page's `next_cursor`). Defaults to 0.")]
    #[serde(default = "zero")]
    pub offset: u16,
}

/// Registered users with role and status, offset-paged.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsersInput {
    #[schemars(description = "Rows per page. 1-100; defaults to 50.")]
    #[serde(default = "fifty")]
    pub limit: u16,
    #[schemars(description = "Rows to skip. Defaults to 0.")]
    #[serde(default = "zero")]
    pub offset: u16,
    #[schemars(
        description = "Role filter: `admin`, `user`, or `anonymous`. Empty means every role. With a role set the CLI returns every match and ignores limit/offset."
    )]
    #[serde(default)]
    pub role: String,
    #[schemars(
        description = "Status filter: `active`, `inactive`, `suspended`, `pending`, `deleted`, `temporary`. Empty means every status. The CLI applies it after paging, so a page can come back short."
    )]
    #[serde(default)]
    pub status: String,
}
