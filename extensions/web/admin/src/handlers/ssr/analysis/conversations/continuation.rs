//! Claude Code sessions resumed after compaction: each opens a new context
//! whose first prompt is the harness's resume preamble. The row names and
//! links the conversation it most likely continues instead.

use serde::Serialize;

use super::row_facts::RowFacts;
use crate::handlers::ssr::analysis_urls::ANALYSIS_CONVERSATIONS_URL;

#[derive(Debug, Serialize)]
pub(crate) struct ContinuationView {
    pub prev_href: Option<String>,
    pub prev_title: Option<String>,
}

impl ContinuationView {
    pub(crate) fn for_row(f: &RowFacts<'_>) -> Option<Self> {
        f.is_continuation.then(|| Self {
            prev_href: f
                .prev_context_id
                .map(|id| format!("{ANALYSIS_CONVERSATIONS_URL}/{}", urlencoding::encode(id))),
            prev_title: f.prev_title.map(str::to_owned),
        })
    }
}

// Why: the preamble is the same for every continuation, so it names none;
// the judge's title wins, then the previous conversation's.
pub(crate) fn row_title(f: &RowFacts<'_>) -> String {
    if let Some(judged) = f.judge_title.filter(|t| !t.is_empty()) {
        return judged.to_owned();
    }
    if !f.is_continuation {
        return f.title.to_owned();
    }
    f.prev_title.map_or_else(
        || {
            let short: String = f.context_id.as_str().chars().take(8).collect();
            format!("Continued session {short}…")
        },
        str::to_owned,
    )
}
