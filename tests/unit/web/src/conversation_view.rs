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
use systemprompt::identifiers::{AiRequestId, AiToolCallId, GatewayConversationId};
use systemprompt_web_admin::repositories::analytics::context_detail::{
    ContextMessageRow, ContextRequestRow,
};
use systemprompt_web_admin::repositories::analytics::context_tool_calls::ContextToolCallRow;
use systemprompt_web_admin::test_support::{
    ConversationView, EmptyReason, TranscriptOptions, build_conversation, parse_assistant, short_id,
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
        cache_read_tokens: None,
        cache_creation_tokens: None,
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
            tool_call_id: None,
            name: None,
            created_at: at(secs),
        })
        .collect()
}

fn tool(id: &str, seq: i32, name: &str, input: &str) -> ContextToolCallRow {
    ContextToolCallRow {
        request_id: AiRequestId::new(id),
        tool_name: name.to_owned(),
        sequence_number: seq,
        ai_tool_call_id: None,
        tool_input: serde_json::from_str(input).expect("valid json"),
        tool_result_payload: None,
        artifact_id: None,
        artifact_structured: false,
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
    let body = "<system-reminder>\nhousekeeping\n</system-reminder>\n\n//acme-admin:demonstrate-rag <system-reminder>more</system-reminder>";
    assert_eq!(strip_system_reminders(body), "//acme-admin:demonstrate-rag");
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

// The reader used to render an empty page for any conversation whose stored
// message rows exceeded a context-wide cap, because the cap was taken
// oldest-first and the transcript is built from the newest request. The
// bodies are now fetched by request id, and these four tests pin the
// behaviour that made that fix possible — plus the two other ways a
// conversation could go blank while its KPI tiles reported real numbers.

#[test]
fn only_the_tail_of_each_thread_needs_its_message_bodies() {
    use systemprompt_web_admin::test_support::transcript_request_ids;
    let requests: Vec<ContextRequestRow> = (0..8)
        .map(|i| req(&format!("r{i}"), i, "turn", Some(MAIN), "completed", 2))
        .collect();
    let ids = transcript_request_ids(&requests);
    assert_eq!(
        ids.messages,
        ["r7", "r6", "r5", "r4", "r3"],
        "the newest five of the thread, so a request that never stored its \
         history does not blank the page"
    );
    assert_eq!(
        ids.tool_calls.len(),
        8,
        "every turn request may own tool-call rows, whoever rendered the text"
    );
}

#[test]
fn a_final_request_with_no_stored_history_falls_back_to_the_one_before_it() {
    let mut messages = history("r1", 0, &[("user", "one"), ("assistant", "first")]);
    messages.extend(history(
        "r2",
        1,
        &[
            ("user", "one"),
            ("assistant", "first"),
            ("user", "two"),
            ("assistant", "second"),
        ],
    ));
    let requests = vec![
        req("r1", 0, "turn", Some(MAIN), "completed", 2),
        req("r2", 1, "turn", Some(MAIN), "completed", 4),
        // Why: this one failed before its best-effort message inserts ran.
        req("r3", 2, "turn", Some(MAIN), "failed", 0),
    ];
    let view = build(&messages, &[], &requests);
    assert!(
        view.has_content,
        "an unwritten final request must not blank a thread whose earlier \
         requests hold the whole transcript"
    );
    assert_eq!(view.turn_count, 2);
    assert_eq!(view.empty_reason, EmptyReason::None);
}

#[test]
fn a_conversation_with_no_turn_classified_requests_still_renders() {
    let messages = history("r1", 0, &[("user", "hello"), ("assistant", "hi")]);
    // Why: `conversation_request_kind` calls a request a utility call when it
    // offered no tools, and whether tools were recorded is a best-effort
    // write. The transcript is real either way.
    let requests = vec![req("r1", 0, "utility", None, "completed", 2)];
    let view = build(&messages, &[], &requests);
    assert!(view.has_content);
    assert_eq!(view.turn_count, 1);
    assert_eq!(
        view.side_calls.count, 0,
        "these rows are the transcript, so they are not also side calls"
    );
}

#[test]
fn an_empty_transcript_says_which_of_the_two_reasons_it_was() {
    let none = build(&[], &[], &[]);
    assert!(!none.has_content);
    assert_eq!(none.empty_reason, EmptyReason::NoRequests);

    let requests = vec![req("r1", 0, "turn", Some(MAIN), "completed", 4)];
    let bodiless = build(&[], &[], &requests);
    assert!(!bodiless.has_content);
    assert_eq!(
        bodiless.empty_reason,
        EmptyReason::NoBodies,
        "requests exist and the KPI tiles will report them, so the page must \
         not claim nothing was recorded"
    );
}

#[test]
fn a_slash_command_row_is_named_by_its_command_not_its_xml_envelope() {
    use systemprompt_web_admin::test_support::history_command_name;
    assert_eq!(
        history_command_name(
            "<command-message>acme-commons:cowork-setup</command-message> \
             <command-name>/acme-commons:cowork-setup</command-name> \
             <command-args></command-args> Base directory for this skill: C:\\Use"
        )
        .as_deref(),
        Some("/acme-commons:cowork-setup")
    );
    assert_eq!(
        history_command_name("how much did we spend on ai today"),
        None
    );
    assert_eq!(history_command_name("<command-name></command-name>"), None);
    assert_eq!(history_command_name("<command-name>unterminated"), None);
}

#[test]
fn a_negative_missing_history_count_cannot_steal_an_assistant_attribution() {
    let messages = history("retained", 1, &[("assistant", "retained reply")]);
    let requests = vec![
        req("retained", 1, "turn", Some(MAIN), "completed", 1),
        req("corrupt", 2, "turn", Some(MAIN), "completed", -1),
    ];
    let view = build(&messages, &[], &requests);
    let reply = &view.threads[0].turns[0].steps[0];
    assert_eq!(reply.text.as_deref(), Some("retained reply"));
    assert_eq!(
        reply.request_id_short.as_deref(),
        Some(short_id("retained").as_str())
    );
    assert_eq!(
        reply.meta.as_ref().map(|meta| meta.model.as_str()),
        Some("model-retained")
    );
}

#[test]
fn retained_message_rows_supply_attribution_even_when_the_summary_count_is_invalid() {
    let messages = history("retained", 1, &[("assistant", "retained reply")]);
    let requests = vec![req("retained", 1, "turn", Some(MAIN), "completed", -1)];
    let view = build(&messages, &[], &requests);
    let reply = &view.threads[0].turns[0].steps[0];
    assert_eq!(
        reply.request_id_short.as_deref(),
        Some(short_id("retained").as_str())
    );
}

fn parallel_call() -> &'static str {
    "[tool_use:Read {\"path\":\"a.rs\"}]\n[tool_use:request_log {\"id\":\"q\"}]"
}

fn parallel_thread(results: &[(&str, &str)]) -> (Vec<ContextMessageRow>, Vec<ContextRequestRow>) {
    let mut messages = history("r1", 1, &[("user", "go"), ("assistant", parallel_call())]);
    let mut rows = vec![("user", "go"), ("assistant", parallel_call())];
    rows.extend_from_slice(results);
    rows.push(("assistant", "done"));
    let count = rows.len() as i64;
    messages.extend(history("r2", 2, &rows));
    let requests = vec![
        req("r1", 1, "turn", Some(MAIN), "completed", 2),
        req("r2", 2, "turn", Some(MAIN), "completed", count),
    ];
    (messages, requests)
}

fn with_id(mut t: ContextToolCallRow, id: &str) -> ContextToolCallRow {
    t.ai_tool_call_id = Some(AiToolCallId::new(id));
    t
}

#[test]
fn a_ledger_payload_is_kept_and_message_text_goes_to_the_step_without_one() {
    let (messages, requests) = parallel_thread(&[("user", "file body")]);
    let mut log = with_id(tool("r1", 1, "request_log", "{\"id\":\"q\"}"), "toolu_b");
    log.tool_result_payload = Some(serde_json::json!({"rows": 3}));
    let tools = vec![
        with_id(tool("r1", 0, "Read", "{\"path\":\"a.rs\"}"), "toolu_a"),
        log,
    ];
    let view = build(&messages, &tools, &requests);
    let steps = &view.threads[0].turns[0].steps;
    let read = steps
        .iter()
        .find(|s| s.tool_name.as_deref() == Some("Read"))
        .expect("read");
    let log = steps
        .iter()
        .find(|s| s.tool_name.as_deref() == Some("request_log"))
        .expect("log");
    assert_eq!(read.tool_result_pretty.as_deref(), Some("file body"));
    assert!(
        log.tool_result_pretty
            .as_deref()
            .expect("payload")
            .contains("\"rows\": 3")
    );
}

#[test]
fn tool_rows_pair_by_id_not_by_position() {
    let (mut messages, requests) =
        parallel_thread(&[("tool", "log result"), ("tool", "file body")]);
    for m in messages.iter_mut().filter(|m| m.role == "tool") {
        let id = if m.content == "log result" {
            "toolu_b"
        } else {
            "toolu_a"
        };
        m.tool_call_id = Some(AiToolCallId::new(id));
    }
    let tools = vec![
        with_id(tool("r1", 0, "Read", "{\"path\":\"a.rs\"}"), "toolu_a"),
        with_id(tool("r1", 1, "request_log", "{\"id\":\"q\"}"), "toolu_b"),
    ];
    let view = build(&messages, &tools, &requests);
    assert_eq!(view.turn_count, 1, "tool rows never start a turn");
    let steps = &view.threads[0].turns[0].steps;
    assert_eq!(steps[0].tool_result_pretty.as_deref(), Some("file body"));
    assert_eq!(steps[1].tool_result_pretty.as_deref(), Some("log result"));
}

#[test]
fn an_unsplittable_parallel_block_is_said_to_be_one_and_never_leaks_forward() {
    let (mut messages, mut requests) = parallel_thread(&[("user", "both results")]);
    let second = "[tool_use:Bash {\"command\":\"ls\"}]";
    let mut rows: Vec<(&str, &str)> = messages
        .iter()
        .filter(|m| m.request_id.as_str() == "r2")
        .map(|m| (m.role.as_str(), m.content.as_str()))
        .collect();
    rows.pop();
    rows.extend([
        ("assistant", second),
        ("user", "listing"),
        ("assistant", "done"),
    ]);
    let r3 = history("r3", 3, &rows);
    requests.push(req(
        "r3",
        3,
        "turn",
        Some(MAIN),
        "completed",
        r3.len() as i64,
    ));
    messages.extend(r3);
    let view = build(&messages, &[], &requests);
    let steps = &view.threads[0].turns[0].steps;
    let first = steps[0].tool_result_pretty.as_deref().expect("combined");
    assert!(first.starts_with("Results of 2 parallel calls"));
    assert!(first.contains("both results"));
    assert!(steps[1].tool_result_pretty.is_none());
    let bash = steps
        .iter()
        .find(|s| s.tool_name.as_deref() == Some("Bash"))
        .expect("bash");
    assert_eq!(bash.tool_result_pretty.as_deref(), Some("listing"));
}
