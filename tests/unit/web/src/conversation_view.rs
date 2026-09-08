//! The conversation builder's contract: a thread renders the LATEST request's
//! history once, assistant rows are attributed to the request that produced
//! them, tool results fold into their tool step, probes go to side calls, and
//! a failed final request appends no reply.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    reason = "test code: panics are the assertion mechanism"
)]

use chrono::{DateTime, TimeZone, Utc};
use systemprompt::identifiers::{AiRequestId, GatewayConversationId};
use systemprompt_web_admin::repositories::analytics::context_detail::{
    ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};
use systemprompt_web_admin::test_support::{
    ConversationView, TranscriptOptions, build_conversation, parse_assistant, short_id,
};

const MAIN: &str = "ctx_00000000000000aa";
const SUB: &str = "ctx_00000000000000bb";

fn at(secs: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000 + secs, 0)
        .single()
        .expect("valid timestamp")
}

#[expect(clippy::too_many_arguments, reason = "one fixture row per call")]
fn req(
    id: &str,
    secs: i64,
    kind: &str,
    thread: Option<&str>,
    status: &str,
    message_count: i64,
) -> ContextRequestRow {
    ContextRequestRow {
        id: AiRequestId::new(id),
        trace_id: None,
        model: Some(format!("model-{id}")),
        status: status.to_owned(),
        latency_ms: Some(10),
        input_tokens: Some(1),
        output_tokens: Some(2),
        cost_microdollars: 5,
        created_at: at(secs),
        effective_kind: kind.to_owned(),
        gateway_conversation_id: thread
            .map(|t| GatewayConversationId::try_new(t).expect("valid thread id")),
        max_tokens: Some(1000),
        message_count,
    }
}

fn history(id: &str, secs: i64, rows: &[(&str, &str)]) -> Vec<ContextMessageRow> {
    rows.iter()
        .enumerate()
        .map(|(seq, (role, content))| ContextMessageRow {
            request_id: AiRequestId::new(id),
            role: (*role).to_owned(),
            sequence_number: seq as i32,
            content: (*content).to_owned(),
            created_at: at(secs),
        })
        .collect()
}

fn tool(id: &str, seq: i32, name: &str, input: &str) -> ContextToolCallRow {
    ContextToolCallRow {
        request_id: AiRequestId::new(id),
        tool_name: name.to_owned(),
        sequence_number: seq,
        tool_input: serde_json::from_str(input).expect("valid json"),
        tool_result_payload: None,
        created_at: at(0),
    }
}

fn three_turn_thread() -> (Vec<ContextMessageRow>, Vec<ContextRequestRow>) {
    let mut messages = history("r1", 1, &[("user", "one"), ("assistant", "a1")]);
    messages.extend(history(
        "r2",
        2,
        &[
            ("user", "one"),
            ("assistant", "a1"),
            ("user", "two"),
            ("assistant", "a2"),
        ],
    ));
    messages.extend(history(
        "r3",
        3,
        &[
            ("user", "one"),
            ("assistant", "a1"),
            ("user", "two"),
            ("assistant", "a2"),
            ("user", "three"),
            ("assistant", "a3"),
        ],
    ));
    let requests = vec![
        req("r1", 1, "turn", Some(MAIN), "completed", 2),
        req("r2", 2, "turn", Some(MAIN), "completed", 4),
        req("r3", 3, "turn", Some(MAIN), "completed", 6),
    ];
    (messages, requests)
}

fn build(
    messages: &[ContextMessageRow],
    tools: &[ContextToolCallRow],
    requests: &[ContextRequestRow],
) -> ConversationView {
    build_conversation(messages, tools, requests, TranscriptOptions::default())
}

#[test]
fn the_latest_request_wins_so_three_requests_yield_three_turns() {
    let (messages, requests) = three_turn_thread();
    let view = build(&messages, &[], &requests);
    assert_eq!(view.threads.len(), 1);
    assert_eq!(view.turn_count, 3);
    let turns = &view.threads[0].turns;
    assert_eq!(
        turns.iter().map(|t| t.prompt.as_str()).collect::<Vec<_>>(),
        ["one", "two", "three"]
    );
    for t in turns {
        assert_eq!(
            t.steps.len(),
            1,
            "one reply per turn, never the replayed history"
        );
    }
    assert!(view.has_content);
    assert!(view.threads[0].is_main);
}

#[test]
fn assistant_rows_are_attributed_to_the_request_that_produced_them() {
    let (messages, requests) = three_turn_thread();
    let view = build(&messages, &[], &requests);
    let turns = &view.threads[0].turns;
    for (i, (turn, id)) in turns.iter().zip(["r1", "r2", "r3"]).enumerate() {
        let step = &turn.steps[0];
        assert!(step.is_assistant);
        assert_eq!(
            step.request_id_short.as_deref(),
            Some(short_id(id).as_str())
        );
        assert_eq!(
            step.meta.as_ref().map(|m| m.model.as_str()),
            Some(format!("model-{id}").as_str())
        );
        assert_eq!(turn.number, i + 1);
        assert_eq!(turn.anchor, format!("turn-{}", i + 1));
    }
}

#[test]
fn tool_results_fold_into_the_tool_step_and_do_not_start_a_turn() {
    let call = "Let me look.\n[tool_use:Read {\"path\":\"x.rs\"}]";
    let mut messages = history("r1", 1, &[("user", "read it"), ("assistant", call)]);
    messages.extend(history(
        "r2",
        2,
        &[
            ("user", "read it"),
            ("assistant", call),
            ("user", "{\"content\":\"fn main() {}\"}"),
            ("assistant", "done"),
        ],
    ));
    let requests = vec![
        req("r1", 1, "turn", Some(MAIN), "completed", 2),
        req("r2", 2, "turn", Some(MAIN), "completed", 4),
    ];
    let tools = vec![tool("r1", 0, "Read", "{\"path\":\"x.rs\"}")];
    let view = build(&messages, &tools, &requests);
    assert_eq!(view.turn_count, 1);
    assert_eq!(view.tool_call_count, 1);
    let turn = &view.threads[0].turns[0];
    assert_eq!(
        turn.steps.len(),
        3,
        "assistant text, tool step, final reply"
    );
    assert_eq!(turn.steps[0].text.as_deref(), Some("Let me look."));
    let tool_step = &turn.steps[1];
    assert!(tool_step.is_tool);
    assert_eq!(tool_step.tool_name.as_deref(), Some("Read"));
    assert!(
        tool_step
            .tool_input_pretty
            .as_deref()
            .expect("input")
            .contains("x.rs")
    );
    assert!(
        tool_step
            .tool_result_pretty
            .as_deref()
            .expect("result")
            .contains("fn main() {}")
    );
    assert_eq!(turn.steps[2].text.as_deref(), Some("done"));
    assert_eq!(turn.tool_names.len(), 1);
    assert_eq!(
        (turn.tool_names[0].name.as_str(), turn.tool_names[0].count),
        ("Read", 1)
    );
}

#[test]
fn markers_are_stripped_from_prompts_and_assistant_text() {
    let call = "Working.\n[tool_use:Bash {\"command\":\"ls\"}]";
    let messages = history(
        "r1",
        1,
        &[
            ("user", "=== USER PROMPT ===\nhello there"),
            ("assistant", call),
        ],
    );
    let requests = vec![req("r1", 1, "turn", Some(MAIN), "completed", 2)];
    let view = build_conversation(&messages, &[], &requests, TranscriptOptions::owner_facing());
    let turn = &view.threads[0].turns[0];
    assert_eq!(turn.prompt, "hello there");
    assert_eq!(turn.steps[0].text.as_deref(), Some("Working."));
    assert!(
        turn.steps
            .iter()
            .all(|s| !s.text.as_deref().unwrap_or_default().contains("[tool_use:"))
    );
    assert_eq!(turn.steps[1].tool_name.as_deref(), Some("Bash"));
}

#[test]
fn probes_are_excluded_from_threads_and_listed_as_side_calls() {
    let (mut messages, mut requests) = three_turn_thread();
    messages.extend(history("probe", 4, &[("user", "count")]));
    requests.push(req("probe", 4, "probe", None, "completed", 1));
    requests.push(req("util", 0, "utility", None, "completed", 1));
    let view = build(&messages, &[], &requests);
    assert_eq!(view.threads.len(), 1);
    assert_eq!(view.turn_count, 3);
    assert_eq!(view.side_calls.count, 2);
    assert_eq!(
        view.side_calls.rows[0].kind, "utility",
        "side calls are ordered by time"
    );
    assert_eq!(view.side_calls.rows[1].kind, "probe");
}

#[test]
fn threads_are_ordered_by_their_first_request_and_turns_number_globally() {
    let mut messages = history("s1", 5, &[("user", "sub task"), ("assistant", "sub done")]);
    messages.extend(history(
        "m1",
        10,
        &[
            ("system", "you are"),
            ("user", "main"),
            ("assistant", "main done"),
        ],
    ));
    let requests = vec![
        req("m1", 10, "turn", Some(MAIN), "completed", 3),
        req("s1", 5, "turn", Some(SUB), "completed", 2),
    ];
    let view = build(&messages, &[], &requests);
    assert_eq!(view.threads.len(), 2);
    assert_eq!(view.threads[0].label, "sub task");
    assert!(view.threads[0].is_main);
    assert!(!view.threads[1].is_main);
    assert_eq!(view.threads[1].anchor, "thread-2");
    assert_eq!(view.threads[1].system_prompt.as_deref(), Some("you are"));
    assert_eq!(view.threads[0].turns[0].number, 1);
    assert_eq!(view.threads[1].turns[0].number, 2);
    assert_eq!(view.threads[1].turns[0].anchor, "turn-2");
    assert_eq!(view.turn_count, 2);
}

#[test]
fn a_failed_last_request_appends_no_reply() {
    let mut messages = history("r1", 1, &[("user", "one"), ("assistant", "a1")]);
    messages.extend(history(
        "r2",
        2,
        &[("user", "one"), ("assistant", "a1"), ("user", "two")],
    ));
    let requests = vec![
        req("r1", 1, "turn", Some(MAIN), "completed", 2),
        req("r2", 2, "turn", Some(MAIN), "failed", 3),
    ];
    let view = build(&messages, &[], &requests);
    let turns = &view.threads[0].turns;
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].steps.len(), 1);
    assert!(
        turns[1].steps.is_empty(),
        "the failed request produced nothing to show"
    );
}

#[test]
fn marker_parsing_handles_nested_brackets_inside_strings() {
    let parsed =
        parse_assistant("text [tool_use:Edit {\"old\":\"a]b\",\"n\":[1,2]}] more [tool_use:Noop]");
    assert_eq!(parsed.text, "text  more");
    assert_eq!(parsed.tool_uses.len(), 2);
    assert_eq!(parsed.tool_uses[0].name, "Edit");
    assert_eq!(
        parsed.tool_uses[0].input_json,
        "{\"old\":\"a]b\",\"n\":[1,2]}"
    );
    assert_eq!(parsed.tool_uses[1].input_json, "{}");
    let broken = parse_assistant("[tool_use:Read {\"unterminated\"");
    assert!(broken.tool_uses.is_empty());
    assert!(broken.text.contains("[tool_use:Read"));
}

#[test]
fn system_reminder_blocks_are_cut_from_a_prompt() {
    use systemprompt_web_admin::test_support::strip_system_reminders;
    let body = "<system-reminder>\nhousekeeping\n</system-reminder>\n\n//astound-admin:demonstrate-rag <system-reminder>more</system-reminder>";
    assert_eq!(
        strip_system_reminders(body),
        "//astound-admin:demonstrate-rag"
    );
    assert_eq!(
        strip_system_reminders("plain <system-reminder>unterminated"),
        "plain <system-reminder>unterminated"
    );
}

#[test]
fn tidy_lines_drops_indentation_and_collapses_blank_runs() {
    use systemprompt_web_admin::test_support::tidy_lines;
    assert_eq!(tidy_lines("  a\n\n\n\n      b\n   \n"), "a\n\nb");
}
