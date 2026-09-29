//! Walks one thread's canonical history into turns and steps.
//!
//! The canonical history is the stored message array of the thread's latest
//! request that has one — the gateway persists the whole array on every
//! request, so the newest is the complete transcript and everything before it
//! is a prefix.
//!
//! A `user` row starts a turn unless the assistant row before it carried a
//! tool use — then it carries that row's results and folds into its tool
//! steps. A `tool` row is always a result. An `assistant` row becomes a text
//! step (when any text survives the marker strip) plus one tool step per
//! marker, whose arguments come from the attributed request's
//! `ai_request_tool_calls` rows when their count matches and from the marker's
//! own JSON otherwise.
//!
//! Results pair by `tool_use` id when both sides carry one, and by position
//! among the latest assistant row's still-open steps only when neither does.
//! A step whose ledger row holds a result payload keeps it and never takes
//! message text.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::conversation::{
    Counters, StepView, THREAD_LABEL_CHARS, ThreadView, TurnView, is_long, local_time, single_line,
};
use super::markers::{parse_assistant, pretty_json_text, strip_system_reminders, tidy_lines};
use super::steps::{tool_step, turn_view};
use super::{meta_view, preview, short_id};
use crate::handlers::ssr::entity_urls::request_detail_url;
use crate::repositories::analytics::context_detail::{ContextMessageRow, ContextRequestRow};
use crate::repositories::analytics::context_tool_calls::ContextToolCallRow;

type MessagesByRequest<'a> = HashMap<&'a str, Vec<&'a ContextMessageRow>>;
type ToolsByRequest<'a> = HashMap<&'a str, Vec<&'a ContextToolCallRow>>;

pub(super) struct ThreadBuilder<'a, 'c> {
    counters: &'c mut Counters,
    messages: &'c MessagesByRequest<'a>,
    tools: &'c ToolsByRequest<'a>,
    open: Vec<OpenTool>,
}

// Why: a tool step of the latest assistant row that may still take a result.
// `backed` means its ledger row holds a result payload; a client-side tool's
// row has none, so its result can only come from the message history.
struct OpenTool {
    step: usize,
    id: Option<String>,
    backed: bool,
}

struct TurnDraft {
    prompt: String,
    ts: DateTime<Utc>,
    steps: Vec<StepView>,
}

impl<'a, 'c> ThreadBuilder<'a, 'c> {
    pub(super) const fn new(
        counters: &'c mut Counters,
        messages: &'c MessagesByRequest<'a>,
        tools: &'c ToolsByRequest<'a>,
    ) -> Self {
        Self {
            counters,
            messages,
            tools,
            open: Vec::new(),
        }
    }

    pub(super) fn build(mut self, index: usize, reqs: &[&'a ContextRequestRow]) -> ThreadView {
        let empty: Vec<&ContextMessageRow> = Vec::new();
        // Why: the latest request that actually retained a history, not simply
        // the latest request. Message inserts are best-effort and warn-only, so
        // a failed or half-written final request would otherwise blank a thread
        // whose earlier requests hold the whole transcript.
        let canonical = reqs
            .iter()
            .rev()
            .find_map(|r| self.messages.get(r.id.as_str()))
            .unwrap_or(&empty);
        let attributed = self.attribute(reqs, canonical);

        let mut turns = Vec::new();
        let mut draft: Option<TurnDraft> = None;
        let mut system_prompt = None;
        let mut system_prompt_chars = 0;
        let mut prev_tool_use = false;
        for (idx, m) in canonical.iter().enumerate() {
            match m.role.as_str() {
                "system" => {
                    if system_prompt.is_none() {
                        let body = self.counters.body(&m.content);
                        system_prompt_chars = body.chars().count();
                        system_prompt = Some(preview(&body));
                    }
                },
                "assistant" => {
                    let turn = draft.get_or_insert_with(|| TurnDraft {
                        prompt: String::new(),
                        ts: m.created_at,
                        steps: Vec::new(),
                    });
                    prev_tool_use = self.assistant_steps(turn, m, attributed.get(&idx).copied());
                },
                role => {
                    let is_result = role == "tool" || prev_tool_use;
                    if is_result && let Some(turn) = draft.as_mut() {
                        self.fold_result(turn, m);
                        // Why: a `tool` row answers one call and more may
                        // follow; a `user` row carries every result at once.
                        prev_tool_use &= role == "tool";
                        continue;
                    }
                    if let Some(done) = draft.take() {
                        turns.push(self.finish(done));
                    }
                    let ts = attributed
                        .iter()
                        .filter(|(i, _)| **i > idx)
                        .min_by_key(|(i, _)| *i)
                        .map_or(m.created_at, |(_, r)| r.created_at);
                    draft = Some(TurnDraft {
                        prompt: self
                            .counters
                            .body(&tidy_lines(&strip_system_reminders(&m.content))),
                        ts,
                        steps: Vec::new(),
                    });
                    prev_tool_use = false;
                },
            }
        }
        if let Some(done) = draft.take() {
            turns.push(self.finish(done));
        }

        let label = turns
            .first()
            .map_or_else(|| format!("Thread {index}"), thread_label);
        ThreadView {
            index,
            is_main: index == 1,
            anchor: format!("thread-{index}"),
            label,
            started_local: reqs
                .first()
                .map_or_else(String::new, |r| local_time(r.created_at)),
            request_count: reqs.len(),
            model: reqs.iter().rev().find_map(|r| r.model.clone()),
            system_prompt,
            system_prompt_chars,
            turns,
        }
    }

    // Why: request i is attributed to canonical index `n_i`: its own history
    // length minus the reply it appended on completion.
    fn attribute(
        &self,
        reqs: &[&'a ContextRequestRow],
        canonical: &[&ContextMessageRow],
    ) -> HashMap<usize, &'a ContextRequestRow> {
        let mut out = HashMap::new();
        for r in reqs {
            let rows = self.messages.get(r.id.as_str());
            let len = match rows {
                Some(rows) => rows.len(),
                None => match usize::try_from(r.message_count) {
                    Ok(count) => count,
                    Err(error) => {
                        tracing::warn!(request_id = %r.id, message_count = r.message_count, %error,
                            "Skipping transcript attribution with invalid message count");
                        continue;
                    },
                },
            };
            let appended_reply = r.status == "completed"
                && rows.is_none_or(|rows| rows.last().is_some_and(|m| m.role == "assistant"));
            let n = if appended_reply {
                len.saturating_sub(1)
            } else {
                len
            };
            if canonical.get(n).is_some_and(|m| m.role == "assistant") {
                out.insert(n, *r);
            }
        }
        out
    }

    fn assistant_steps(
        &mut self,
        turn: &mut TurnDraft,
        m: &ContextMessageRow,
        request: Option<&ContextRequestRow>,
    ) -> bool {
        let parsed = parse_assistant(&m.content);
        let (id_short, url) = request.map_or((None, None), |r| {
            (
                Some(short_id(r.id.as_str())),
                Some(request_detail_url(&r.id)),
            )
        });
        if !parsed.text.is_empty() {
            let text = self.counters.body(&tidy_lines(&parsed.text));
            turn.steps.push(StepView {
                is_assistant: true,
                is_tool: false,
                text_is_long: is_long(&text),
                text: Some(text),
                tool_name: None,
                tool_input_pretty: None,
                tool_result_pretty: None,
                artifact_url: None,
                artifact_structured: false,
                meta: request.map(meta_view),
                request_id_short: id_short.clone(),
                request_url: url.clone(),
            });
        }
        // Why: markers carry no tool_use id, so ledger rows can only follow
        // marker order, and only when every marker has one.
        let rows = request
            .and_then(|r| self.tools.get(r.id.as_str()))
            .filter(|rows| rows.len() == parsed.tool_uses.len());
        self.open.clear();
        for (i, marker) in parsed.tool_uses.iter().enumerate() {
            let row = rows.and_then(|rows| rows.get(i).copied());
            self.open.push(OpenTool {
                step: turn.steps.len(),
                id: row
                    .and_then(|r| r.ai_tool_call_id.as_ref())
                    .map(|id| id.as_str().to_owned()),
                backed: row.is_some_and(|r| r.tool_result_payload.is_some()),
            });
            turn.steps
                .push(tool_step(marker, row, id_short.clone(), url.clone()));
            self.counters.tool_calls += 1;
        }
        !parsed.tool_uses.is_empty()
    }

    fn fold_result(&mut self, turn: &mut TurnDraft, m: &ContextMessageRow) {
        let target = match m.tool_call_id.as_ref() {
            Some(id) => self
                .open
                .iter()
                .position(|o| o.id.as_deref() == Some(id.as_str()))
                .or_else(|| self.first_unbacked(true)),
            None => self.first_unbacked(false),
        };
        let unclaimed = self.open.iter().filter(|o| !o.backed).count();
        let Some(pos) = target else {
            if m.role != "tool" {
                self.open.clear();
            }
            return;
        };
        let open = self.open.remove(pos);
        if !open.backed
            && let Some(step) = turn.steps.get_mut(open.step)
        {
            let body = self.counters.body(&m.content);
            // Why: the gateway flattens a row's results into one text block
            // with no boundary between them, so without ids the parallel
            // results cannot be split and are shown together, said as such.
            step.tool_result_pretty = Some(if m.tool_call_id.is_none() && unclaimed > 1 {
                format!("Results of {unclaimed} parallel calls, stored as one block:\n\n{body}")
            } else {
                pretty_json_text(&body)
            });
        }
        if m.role != "tool" {
            self.open.clear();
        }
    }

    // Why: position is the fallback only for steps no id could ever reach;
    // `id_less_only` keeps an id-carrying result off a step with its own id.
    fn first_unbacked(&self, id_less_only: bool) -> Option<usize> {
        self.open
            .iter()
            .position(|o| !o.backed && (!id_less_only || o.id.is_none()))
    }

    fn finish(&mut self, draft: TurnDraft) -> TurnView {
        self.counters.turn_number += 1;
        turn_view(
            self.counters.turn_number,
            draft.prompt,
            draft.ts,
            draft.steps,
        )
    }
}

fn thread_label(turn: &TurnView) -> String {
    single_line(&turn.prompt, THREAD_LABEL_CHARS)
}
