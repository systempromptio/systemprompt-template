//! The transcript model and its rows: whose history it holds, rows that keep
//! their identity while a call or a reply changes, and what a deleted session
//! or an update the model cannot place does to it.

use serde_json::json;
use systemprompt_engine::store::{Change, SessionId};
use systemprompt_harness_views::state::session_model::ModelEffect;
use systemprompt_harness_views::transcript::rows::{changed_range, row_keys};
use systemprompt_harness_views::transcript::tool_info::{Phase, summarize};

use crate::TestResult;
use crate::fixtures::{assistant, entry, loaded, state, text, tool, user};

#[test]
fn a_transcript_holds_only_the_history_of_its_own_session() {
    let id = SessionId::mint(&crate::IDS);
    let other = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let foreign = user(&other);
    let part = text(&other, &foreign.id, "not mine");
    assert_eq!(
        model.apply_change(&Change::MessageUpdated(foreign.into())),
        ModelEffect::Unchanged
    );
    assert_eq!(
        model.apply_change(&Change::PartUpdated(part)),
        ModelEffect::Unchanged
    );
    assert!(model.messages.is_empty());
}

#[test]
fn a_deleted_session_is_marked_gone_and_stops_waiting() -> TestResult {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let project_id = model
        .session
        .as_ref()
        .map(|session| session.project_id.clone())
        .ok_or("a loaded session")?;
    let effect = model.apply_change(&Change::SessionDeleted {
        id: id.clone(),
        project_id,
        parent_id: None,
    });
    assert_eq!(effect, ModelEffect::Changed);
    assert!(model.deleted);
    assert!(!model.is_busy());
    assert!(model.permissions.is_empty() && model.questions.is_empty());
    Ok(())
}

#[test]
fn an_update_the_model_cannot_place_keeps_the_transcript_and_asks_for_a_reload() -> TestResult {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let prompt = user(&id);
    let first = text(&id, &prompt.id, "hello");
    model.apply_change(&Change::MessageUpdated(prompt.into()));
    model.apply_change(&Change::PartUpdated(first));
    let unknown = user(&id);
    let orphan = text(&id, &unknown.id, "arrived before its message");
    assert_eq!(
        model.apply_change(&Change::PartUpdated(orphan)),
        ModelEffect::Reload
    );
    assert_eq!(model.messages.len(), 1);
    assert_eq!(model.messages[0].parts.len(), 1);
    Ok(())
}

#[test]
fn a_tool_call_keeps_one_row_from_pending_to_completed() -> TestResult {
    let id = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let prompt = user(&id);
    let reply = assistant(&id, &prompt.id);
    let input = json!({ "path": "src/lib.rs" });
    let pending = tool(
        &id,
        &reply.id,
        "read",
        state(json!({ "status": "pending", "input": input }))?,
    )?;
    model.apply_change(&Change::MessageUpdated(prompt.into()));
    model.apply_change(&Change::MessageUpdated(reply.into()));
    model.apply_change(&Change::PartUpdated(pending.clone()));
    let before = row_keys(&model.messages, true);

    let steps = [
        json!({ "status": "running", "input": input, "started": 1 }),
        json!({ "status": "completed", "input": input, "output": "x", "title": "lib.rs", "started": 1, "ended": 2 }),
    ];
    for (offset, step) in (1_i64..).zip(steps) {
        let mut next = pending.clone();
        next.updated_at = pending.updated_at + offset;
        if let systemprompt_engine::store::PartBody::Tool { state: slot, .. } = &mut next.body {
            **slot = state(step)?;
        }
        assert_eq!(
            model.apply_change(&Change::PartUpdated(next)),
            ModelEffect::Changed
        );
    }
    let after = row_keys(&model.messages, true);
    assert_eq!(model.messages[1].parts.len(), 1);
    assert_eq!(before.len(), after.len());
    assert_eq!(changed_range(&before, &after), Some((1..2, 1)));
    let systemprompt_engine::store::PartBody::Tool {
        tool: name,
        state: done,
        ..
    } = &model.messages[1].parts[0].body
    else {
        return Err("a tool part".into());
    };
    assert_eq!(summarize(name, done).phase, Phase::Done);
    Ok(())
}

#[test]
fn linking_a_child_session_updates_the_task_row_in_place() -> TestResult {
    let id = SessionId::mint(&crate::IDS);
    let child = SessionId::mint(&crate::IDS);
    let mut model = loaded(&id);
    let prompt = user(&id);
    let reply = assistant(&id, &prompt.id);
    let input = json!({ "description": "look around" });
    let task = tool(
        &id,
        &reply.id,
        "task",
        state(json!({ "status": "running", "input": input, "started": 1 }))?,
    )?;
    model.apply_change(&Change::MessageUpdated(prompt.into()));
    model.apply_change(&Change::MessageUpdated(reply.into()));
    model.apply_change(&Change::PartUpdated(task.clone()));

    let mut linked = task.clone();
    linked.updated_at = task.updated_at + 1;
    linked.body = systemprompt_engine::store::PartBody::Tool {
        call_id: systemprompt_engine::run::CallId::try_new("call-1")?,
        tool: "task".to_owned(),
        state: Box::new(state(json!({
            "status": "running",
            "input": input,
            "metadata": { "session_id": child.to_string() },
            "started": 1
        }))?),
    };
    assert_eq!(
        model.apply_change(&Change::PartUpdated(linked)),
        ModelEffect::Changed
    );
    assert_eq!(model.messages[1].parts.len(), 1);
    assert_eq!(model.messages[1].parts[0].id, task.id);
    Ok(())
}

#[test]
fn streamed_text_keeps_one_row_through_growth_and_completion() {
    let id = SessionId::mint(&crate::IDS);
    let prompt = user(&id);
    let reply = assistant(&id, &prompt.id);
    let prompt_text = text(&id, &prompt.id, "go");
    let mut part = text(&id, &reply.id, "Wor");
    let mut messages = vec![
        entry(prompt, vec![prompt_text]),
        entry(reply, vec![part.clone()]),
    ];
    let start = row_keys(&messages, true);
    part.body = systemprompt_engine::store::PartBody::Text {
        text: "Working on it, done.".to_owned(),
        synthetic: false,
    };
    messages[1].parts = vec![part];
    let grown = row_keys(&messages, true);
    let settled = row_keys(&messages, false);
    assert_eq!(start.len(), 2);
    assert_eq!(grown.len(), 2);
    assert_eq!(settled.len(), 2);
    assert_eq!(changed_range(&start, &grown), Some((1..2, 1)));
    assert_eq!(changed_range(&grown, &settled), Some((1..2, 1)));
}

#[test]
fn an_unchanged_transcript_reuses_every_row() {
    let id = SessionId::mint(&crate::IDS);
    let prompt = user(&id);
    let reply = assistant(&id, &prompt.id);
    let messages = vec![
        entry(prompt.clone(), vec![text(&id, &prompt.id, "go")]),
        entry(reply.clone(), vec![text(&id, &reply.id, "done")]),
    ];
    assert_eq!(
        changed_range(&row_keys(&messages, false), &row_keys(&messages, false)),
        None
    );
}
