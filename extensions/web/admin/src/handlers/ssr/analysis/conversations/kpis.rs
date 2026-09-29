//! The KPI tiles above the conversations table: seven figures from the
//! filtered set's totals, each toned, icon-led and carrying a sparkline of
//! the window in the tile's trend slot.

use serde::Serialize;

use crate::handlers::ssr::analysis::tone::{
    cache_tone, completion_tone, deny_tone, error_rate_tone, percent, safety_tone, score_display,
};
use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::handlers::ssr::types::{SparklineView, sparkline_toned};
use crate::repositories::analysis::conversations::{
    ConversationAnalysisTotals, ConversationSeriesPoint,
};

// Why: one KPI tile — value, supporting line, tone and the window's sparkline.
#[derive(Debug, Serialize)]
pub(crate) struct ConversationTileView {
    pub label: &'static str,
    pub icon: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
    pub hint: &'static str,
    pub spark: SparklineView,
}

// Why: the words of a tile before its sparkline is drawn.
struct Tile {
    label: &'static str,
    icon: &'static str,
    value: String,
    note: String,
    tone: &'static str,
    hint: &'static str,
}

impl Tile {
    fn with_spark(self, values: &[i64]) -> ConversationTileView {
        ConversationTileView {
            spark: sparkline_toned(
                values,
                self.tone,
                format!("{} per bucket over the window", self.label),
            ),
            label: self.label,
            icon: self.icon,
            value: self.value,
            note: self.note,
            tone: self.tone,
            hint: self.hint,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct KpisView {
    pub tiles: Vec<ConversationTileView>,
    pub conversations: i64,
    pub judged: i64,
    pub pending_judgement: i64,
    pub completion_display: String,
    pub completion_tone: &'static str,
}

fn column(series: &[ConversationSeriesPoint], f: fn(&ConversationSeriesPoint) -> i64) -> Vec<i64> {
    series.iter().map(f).collect()
}

fn volume_tiles(
    t: &ConversationAnalysisTotals,
    series: &[ConversationSeriesPoint],
) -> Vec<ConversationTileView> {
    let tokens = t.input_tokens + t.output_tokens;
    let cache_share = if tokens + t.cache_tokens == 0 {
        "no cache".to_owned()
    } else {
        format!(
            "{}% served from cache",
            t.cache_tokens * 100 / (t.input_tokens + t.cache_tokens).max(1)
        )
    };
    vec![
        Tile {
            label: "Conversations",
            icon: "chat",
            value: t.conversations.to_string(),
            note: format!("{} people · {} requests", t.users, t.requests),
            tone: "accent",
            hint: "Gateway conversations whose last activity falls in the window",
        }
        .with_spark(&column(series, |p| p.conversations)),
        Tile {
            label: "Turns",
            icon: "bolt",
            value: t.turns.to_string(),
            note: format!(
                "{} tool calls · {} artifacts ({} files)",
                t.tool_calls, t.artifacts, t.artifact_files
            ),
            tone: "accent",
            hint: "Human prompts answered; side calls the harness made are excluded",
        }
        .with_spark(&column(series, |p| p.turns)),
        Tile {
            label: "Tokens",
            icon: "token",
            value: format_token_total(tokens),
            note: cache_share,
            tone: cache_tone(t.cache_tokens, t.input_tokens),
            hint: "Input plus output tokens; the note is the cache-read share of everything the models read",
        }
        .with_spark(&column(series, |p| p.tokens)),
        Tile {
            label: "Cost",
            icon: "coins",
            value: format_cost(t.total_cost_microdollars),
            note: format!(
                "{} per conversation",
                format_cost(t.total_cost_microdollars / t.conversations.max(1))
            ),
            tone: "accent",
            hint: "Priced spend on every request in these conversations",
        }
        .with_spark(&column(series, |p| p.cost_microdollars)),
    ]
}

fn health_tiles(
    t: &ConversationAnalysisTotals,
    series: &[ConversationSeriesPoint],
) -> Vec<ConversationTileView> {
    vec![
        Tile {
            label: "Errors",
            icon: "alert",
            value: percent(t.errors, t.requests),
            note: format!("{} failed · {} rejected", t.errors, t.rejected),
            tone: error_rate_tone(t.errors, t.requests),
            hint: "Failed requests as a share of all requests",
        }
        .with_spark(&column(series, |p| p.errors)),
        Tile {
            label: "Denied",
            icon: "shield",
            value: t.denied.to_string(),
            note: format!("{} warned · {} skills invoked", t.warned, t.skill_invocations),
            tone: deny_tone(t.denied),
            hint: "Tool calls the governance chain denied",
        }
        .with_spark(&column(series, |p| p.denied)),
        Tile {
            label: "Active time",
            icon: "gauge",
            value: format_duration_ms(t.active_ms / t.conversations.max(1)),
            note: format!(
                "per conversation · {} safety findings · {} blocked",
                t.safety_findings, t.safety_blocked
            ),
            tone: if t.safety_blocked > 0 {
                safety_tone(t.safety_findings, t.safety_blocked)
            } else {
                "accent"
            },
            hint: "Mean time the model spent answering per conversation: the summed latency of its turns, idle time excluded",
        }
        .with_spark(&column(series, |p| p.users)),
    ]
}

impl KpisView {
    pub(crate) fn new(t: &ConversationAnalysisTotals, series: &[ConversationSeriesPoint]) -> Self {
        let mut tiles = volume_tiles(t, series);
        tiles.extend(health_tiles(t, series));
        Self {
            tiles,
            conversations: t.conversations,
            judged: t.judged,
            pending_judgement: t.pending_judgement,
            completion_display: score_display(t.completion_avg),
            completion_tone: completion_tone(t.completion_avg),
        }
    }
}
