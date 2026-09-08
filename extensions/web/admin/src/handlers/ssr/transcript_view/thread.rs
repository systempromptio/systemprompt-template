//! Walks one thread's canonical history into turns and steps.
//!
//! A `user` row starts a turn unless the assistant row before it carried a
//! tool use — then it is that tool's result and folds into the pending tool
//! step. An `assistant` row becomes a text step (when any text survives the
//! marker strip) plus one tool step per marker, whose arguments come from the
//! attributed request's `ai_request_tool_calls` rows when their count matches
//! and from the marker's own JSON otherwise.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::conversation::{
    Counters, PROMPT_SHORT_CHARS, StepView, THREAD_LABEL_CHARS, ThreadView, ToolChipView, TurnView,
    is_long, local_time, single_line,
};
use super::markers::{
    ToolUseMarker, parse_assistant, pretty_json_text, strip_system_reminders, tidy_lines,
};
use super::{meta_view, pretty_json, preview, short_id};
use crate::handlers::ssr::entity_urls::request_detail_url;
use crate::repositories::analytics::context_detail::{
    ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};

type MessagesByRequest<'a> = HashMap<&'a str, Vec<&'a ContextMessageRow>>;
type ToolsByRequest<'a> = HashMap<&'a str, Vec<&'a ContextToolCallRow>>;

pub(super) struct ThreadBuilder<'a, 'c> {
    counters: &'c mut Counters,
    messages: &'c MessagesByRequest<'a>,
    tools: &'c ToolsByRequest<'a>,
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
        }
    }

    pub(super) fn build(mut self, index: usize, reqs: &[&'a ContextRequestRow]) -> ThreadView {
        let empty: Vec<&ContextMessageRow> = Vec::new();
        let canonical = reqs
            .last()
            .and_then(|last| self.messages.get(last.id.as_str()))
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
                _ => {
                    if prev_tool_use && let Some(turn) = draft.as_mut() {
                        self.fold_result(turn, m);
                    } else {
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
                    }
                    prev_tool_use = false;
                },
            }
        }
        if let Some(done) = draft.take() {
            turns.push(self.finish(done));
        }

        let label = turns.first().map_or_else(
            || format!("Thread {index}"),
            |t| single_line(&t.prompt, THREAD_LABEL_CHARS),
        );
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
            let len = rows.map_or_else(
                || usize::try_from(r.message_count).unwrap_or_default(),
                Vec::len,
            );
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
                meta: request.map(meta_view),
                request_id_short: id_short.clone(),
                request_url: url.clone(),
            });
        }
        let rows = request
            .and_then(|r| self.tools.get(r.id.as_str()))
            .filter(|rows| rows.len() == parsed.tool_uses.len());
        for (i, marker) in parsed.tool_uses.iter().enumerate() {
            let row = rows.and_then(|rows| rows.get(i).copied());
            turn.steps
                .push(tool_step(marker, row, id_short.clone(), url.clone()));
            self.counters.tool_calls += 1;
        }
        !parsed.tool_uses.is_empty()
    }

    fn fold_result(&mut self, turn: &mut TurnDraft, m: &ContextMessageRow) {
        let body = self.counters.body(&m.content);
        if let Some(step) = turn
            .steps
            .iter_mut()
            .find(|s| s.is_tool && s.tool_result_pretty.is_none())
        {
            step.tool_result_pretty = Some(pretty_json_text(&body));
        }
    }

    fn finish(&mut self, draft: TurnDraft) -> TurnView {
        self.counters.turn_number += 1;
        let number = self.counters.turn_number;
        let mut chips: Vec<ToolChipView> = Vec::new();
        for name in draft.steps.iter().filter_map(|s| s.tool_name.as_deref()) {
            if let Some(chip) = chips.iter_mut().find(|c| c.name == name) {
                chip.count += 1;
                continue;
            }
            chips.push(ToolChipView {
                name: name.to_owned(),
                count: 1,
            });
        }
        TurnView {
            number,
            anchor: format!("turn-{number}"),
            prompt_short: single_line(&draft.prompt, PROMPT_SHORT_CHARS),
            prompt_is_long: is_long(&draft.prompt),
            prompt: draft.prompt,
            ts_local: local_time(draft.ts),
            ts_full: draft.ts.to_rfc3339(),
            steps: draft.steps,
            tool_names: chips,
        }
    }
}

fn tool_step(
    marker: &ToolUseMarker,
    row: Option<&ContextToolCallRow>,
    request_id_short: Option<String>,
    request_url: Option<String>,
) -> StepView {
    StepView {
        is_assistant: false,
        is_tool: true,
        text: None,
        text_is_long: false,
        tool_name: Some(row.map_or_else(|| marker.name.clone(), |r| r.tool_name.clone())),
        tool_input_pretty: Some(row.map_or_else(
            || pretty_json_text(&marker.input_json),
            |r| pretty_json(&r.tool_input),
        )),
        tool_result_pretty: row
            .and_then(|r| r.tool_result_payload.as_ref())
            .map(pretty_json),
        meta: None,
        request_id_short,
        request_url,
    }
}
