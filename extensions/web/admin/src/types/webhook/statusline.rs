//! The statusline ingest: Claude Code's statusline JSON as a typed wire shape,
//! and the validated record the handler stores.
//!
//! Claude Code pipes one JSON document to the configured `statusLine` command
//! on every refresh. Its documented shape is
//!
//! ```json
//! { "session_id": "…",
//!   "model": { "id": "claude-opus-4-1", "display_name": "Opus" },
//!   "cost": { "total_cost_usd": 0.42, "total_duration_ms": 45000, … },
//!   "context_window": { "context_window_size": 200000,
//!                       "current_usage": { "input_tokens": …, "output_tokens": …,
//!                                          "cache_creation_input_tokens": …,
//!                                          "cache_read_input_tokens": … } },
//!   "workspace": …, "version": …, "output_style": …, "exceeds_200k_tokens": … }
//! ```
//!
//! [`StatusLinePayload`] names the fields this ingest stores. The rest are
//! Claude Code's to add and rename: they are ignored by name, never captured
//! as an untyped blob, so a new client field can neither break ingestion nor
//! leak into storage unreviewed.
//!
//! [`StatusLineIngest`] is the only thing the handler stores. It exists iff the
//! request identified a session and every number was in range; the
//! conversion is the single place those rules live.

use serde::Deserialize;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

use crate::repositories::dashboard::usage_aggregations::SessionCostSnapshot;

use super::validate_session_key;

const MICRODOLLARS_PER_DOLLAR: f64 = 1_000_000.0;

#[derive(Debug, Deserialize)]
pub struct StatusLineQuery {
    pub plugin_id: Option<PluginId>,
    // Why: the bridge may name the session on the URL; the payload names it
    // in the body. Both may be present only if they agree — a request that
    // names two sessions is reporting for neither.
    pub session_id: Option<SessionId>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLinePayload {
    pub session_id: Option<SessionId>,
    pub model: Option<StatusLineModel>,
    pub cost: Option<StatusLineCost>,
    pub context_window: Option<ContextWindow>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineModel {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct StatusLineCost {
    pub total_cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ContextWindow {
    pub context_window_size: Option<i64>,
    pub current_usage: Option<ContextWindowUsage>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ContextWindowUsage {
    #[serde(rename = "input_tokens")]
    pub input: Option<i64>,
    #[serde(rename = "output_tokens")]
    pub output: Option<i64>,
    #[serde(rename = "cache_creation_input_tokens")]
    pub cache_creation_input: Option<i64>,
    #[serde(rename = "cache_read_input_tokens")]
    pub cache_read_input: Option<i64>,
}

/// Why a statusline request was refused. Each variant is one rule of
/// [`StatusLineIngest::try_from`], so a client reads the exact field at fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusLineRejection {
    MissingSession,
    SessionMismatch,
    InvalidSession(String),
    InvalidCost,
    NegativeContextWindow,
    NegativeTokenCount,
}

impl std::fmt::Display for StatusLineRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSession => f.write_str("session_id is required"),
            Self::SessionMismatch => f.write_str("Query and payload session_id must match"),
            Self::InvalidSession(reason) => f.write_str(reason),
            Self::InvalidCost => {
                f.write_str("cost.total_cost_usd must be a finite, nonnegative amount")
            },
            Self::NegativeContextWindow => {
                f.write_str("context_window.context_window_size must be nonnegative")
            },
            Self::NegativeTokenCount => f.write_str("token counts must be nonnegative"),
        }
    }
}

/// A validated statusline report, ready to store: the session it belongs to
/// and every measurement in the units the tables keep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLineIngest {
    pub session_id: SessionId,
    pub model_id: Option<String>,
    pub total_cost_microdollars: Option<i64>,
    pub context_window_size: Option<i64>,
    pub usage: Option<TokenUsage>,
}

/// Token counts from the session's current context, each proven nonnegative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenUsage {
    pub input: Option<i64>,
    pub output: Option<i64>,
    pub cache_creation_input: Option<i64>,
    pub cache_read_input: Option<i64>,
}

impl TryFrom<(StatusLineQuery, StatusLinePayload)> for StatusLineIngest {
    type Error = StatusLineRejection;

    fn try_from(
        (query, payload): (StatusLineQuery, StatusLinePayload),
    ) -> Result<Self, Self::Error> {
        let session_id = match (query.session_id, payload.session_id) {
            (Some(from_query), Some(from_body)) if from_query != from_body => {
                return Err(StatusLineRejection::SessionMismatch);
            },
            (Some(from_query), _) => from_query,
            (None, Some(from_body)) => from_body,
            (None, None) => return Err(StatusLineRejection::MissingSession),
        };
        validate_session_key(session_id.as_str()).map_err(StatusLineRejection::InvalidSession)?;

        let total_cost_microdollars = payload
            .cost
            .and_then(|c| c.total_cost_usd)
            .map(usd_to_microdollars)
            .transpose()?;

        let window = payload.context_window;
        let context_window_size = window.and_then(|w| w.context_window_size);
        if context_window_size.is_some_and(|v| v < 0) {
            return Err(StatusLineRejection::NegativeContextWindow);
        }
        let usage = window
            .and_then(|w| w.current_usage)
            .map(TokenUsage::try_from)
            .transpose()?;

        Ok(Self {
            session_id,
            model_id: payload.model.and_then(|m| m.id).filter(|id| !id.is_empty()),
            total_cost_microdollars,
            context_window_size,
            usage,
        })
    }
}

impl TryFrom<ContextWindowUsage> for TokenUsage {
    type Error = StatusLineRejection;

    fn try_from(usage: ContextWindowUsage) -> Result<Self, Self::Error> {
        let counts = [
            usage.input,
            usage.output,
            usage.cache_creation_input,
            usage.cache_read_input,
        ];
        if counts.into_iter().flatten().any(|v| v < 0) {
            return Err(StatusLineRejection::NegativeTokenCount);
        }
        Ok(Self {
            input: usage.input,
            output: usage.output,
            cache_creation_input: usage.cache_creation_input,
            cache_read_input: usage.cache_read_input,
        })
    }
}

impl StatusLineIngest {
    pub fn snapshot<'a>(&'a self, user_id: &'a UserId) -> SessionCostSnapshot<'a> {
        SessionCostSnapshot {
            session_id: &self.session_id,
            user_id,
            model: self.model_id.as_deref(),
            total_cost_microdollars: self.total_cost_microdollars,
            context_window_size: self.context_window_size,
            input_tokens: self.usage.and_then(|u| u.input),
            output_tokens: self.usage.and_then(|u| u.output),
            cache_creation_input_tokens: self.usage.and_then(|u| u.cache_creation_input),
            cache_read_input_tokens: self.usage.and_then(|u| u.cache_read_input),
        }
    }
}

// Why: the tables keep cost as integer microdollars. Round-to-nearest keeps
// sub-cent amounts; anything that is not a finite, nonnegative dollar figure
// representable in i64 is the client's error, not a value to clamp.
pub fn usd_to_microdollars(usd: f64) -> Result<i64, StatusLineRejection> {
    let micro = (usd * MICRODOLLARS_PER_DOLLAR).round();
    if usd < 0.0 || !micro.is_finite() || micro >= i64::MAX as f64 {
        return Err(StatusLineRejection::InvalidCost);
    }
    Ok(micro as i64)
}
