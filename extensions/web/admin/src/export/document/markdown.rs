//! A conversation bundle rendered for reading: the transcript turn by turn
//! with each tool call's input and result, then the planes around it as
//! tables. The JSON document is the record; this is the same record laid out
//! for a person.

use super::bundle::ConversationBundle;
use crate::export::format::usd;
use crate::handlers::ssr::transcript_view::{StepView, ThreadView};

const TABLE_TEXT_CHARS: usize = 160;

// Why: writing into a `String` cannot fail, and the `fmt::Write` contract
// would make every line a discarded `Result`; one line, one call.
macro_rules! line {
    ($out:expr, $($arg:tt)*) => {{
        $out.push_str(&format!($($arg)*));
        $out.push('\n');
    }};
}

pub(crate) fn render(b: &ConversationBundle) -> String {
    let mut out = String::new();
    let title = b.title.as_deref().unwrap_or("Conversation");
    line!(out, "# {title}\n");
    line!(out, "- Context: `{}`", b.context_id);
    if let Some(name) = &b.header.display_name {
        line!(out, "- Person: {name}");
    }
    if let Some(session) = &b.header.client_session_id {
        line!(out, "- Harness session: `{session}`");
    }
    if let Some(f) = &b.facts {
        line!(
            out,
            "- Client: {} · Models: {} · {} → {}",
            f.client_kind,
            f.models.join(", "),
            f.first_at.to_rfc3339(),
            f.last_at.to_rfc3339()
        );
        line!(
            out,
            "- Requests: {} ({} turns) · Tokens: {} in / {} out · Cost: ${} · Tool calls: {} · Errors: {}",
            f.request_count,
            f.turn_count,
            f.input_tokens,
            f.output_tokens,
            usd(f.cost_microdollars),
            f.tool_calls_intended,
            f.error_count
        );
        if let Some(score) = f.completion {
            line!(
                out,
                "- Judge: {score}/100 · {} · {}",
                f.outcome.as_deref().unwrap_or("—"),
                f.summary.as_deref().unwrap_or("")
            );
        }
    }
    line!(
        out,
        "- Exported: {}{}",
        b.exported_at.to_rfc3339(),
        if b.redacted { " (redacted)" } else { "" }
    );
    line!(out, "\n## Transcript\n");
    for thread in &b.transcript.threads {
        thread_section(&mut out, thread);
    }
    if b.transcript.threads.is_empty() {
        line!(
            out,
            "_No message bodies were stored for this conversation._\n"
        );
    }
    planes(&mut out, b);
    out
}

fn thread_section(out: &mut String, thread: &ThreadView) {
    if !thread.is_main {
        line!(out, "### Thread {} — {}\n", thread.index, thread.label);
    }
    if let Some(system) = &thread.system_prompt {
        line!(
            out,
            "<details><summary>System prompt</summary>\n\n```text\n{system}\n```\n\n</details>\n"
        );
    }
    for turn in &thread.turns {
        line!(out, "### Turn {} — {}\n", turn.number, turn.ts_full);
        line!(out, "**User**\n\n{}\n", quoted(&turn.prompt));
        for step in &turn.steps {
            step_block(out, step);
        }
    }
}

fn step_block(out: &mut String, step: &StepView) {
    if step.is_tool {
        line!(
            out,
            "**Tool** `{}`\n",
            step.tool_name.as_deref().unwrap_or("?")
        );
        if let Some(input) = &step.tool_input_pretty {
            line!(out, "```json\n{input}\n```\n");
        }
        if let Some(result) = &step.tool_result_pretty {
            line!(out, "Result:\n\n```json\n{result}\n```\n");
        }
        return;
    }
    if step.is_assistant {
        let meta = step
            .meta
            .as_ref()
            .map(|m| {
                format!(
                    " _({} · {} · {} · {})_",
                    m.model, m.status, m.latency_display, m.cost_display
                )
            })
            .unwrap_or_default();
        line!(out, "**Assistant**{meta}\n");
    }
    if let Some(text) = &step.text {
        line!(out, "{text}\n");
    }
}

fn planes(out: &mut String, b: &ConversationBundle) {
    requests_table(out, b);
    ledger_table(out, b);
    governance_tables(out, b);
    hook_table(out, b);
}

fn requests_table(out: &mut String, b: &ConversationBundle) {
    line!(out, "## Requests\n");
    line!(
        out,
        "| # | At | Kind | Model | Status | In | Out | Cost | Latency | Tools |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for (i, r) in b.requests.iter().enumerate() {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            i + 1,
            r.created_at.to_rfc3339(),
            r.kind,
            r.model.as_deref().unwrap_or("—"),
            r.status,
            r.input_tokens.unwrap_or_default(),
            r.output_tokens.unwrap_or_default(),
            usd(r.cost_microdollars),
            r.latency_ms
                .map_or_else(|| "—".to_owned(), |ms| format!("{ms} ms")),
            r.tool_names.join(", ")
        );
    }
}

fn ledger_table(out: &mut String, b: &ConversationBundle) {
    line!(out, "\n## Tool ledger\n");
    line!(
        out,
        "| At | Tool | State | Status | Duration | Result | Error |\n|---|---|---|---|---|---|---|"
    );
    for t in &b.tool_ledger {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            t.occurred_at.map(|a| a.to_rfc3339()).unwrap_or_default(),
            cell(t.tool_name.as_deref().unwrap_or("—")),
            t.state,
            t.execution_status.as_deref().unwrap_or("—"),
            t.execution_time_ms
                .map_or_else(|| "—".to_owned(), |ms| format!("{ms} ms")),
            t.artifact_kind.as_deref().unwrap_or("—"),
            cell(t.error_message.as_deref().unwrap_or(""))
        );
    }
}

fn governance_tables(out: &mut String, b: &ConversationBundle) {
    line!(out, "\n## Governance decisions\n");
    line!(
        out,
        "| At | Tool | Decision | Policy | Reason |\n|---|---|---|---|---|"
    );
    for d in &b.decisions {
        line!(
            out,
            "| {} | {} | {} | {} | {} |",
            d.created_at.to_rfc3339(),
            cell(&d.tool_name),
            d.decision,
            d.policy,
            cell(&d.reason)
        );
    }
    line!(out, "\n## Skills\n");
    line!(
        out,
        "| Skill | Plugin | Marketplace | Version | Invocations | First |\n|---|---|---|---|---|---|"
    );
    for s in &b.skills {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            s.skill,
            s.plugin_id.as_ref().map_or("—", |p| p.as_str()),
            s.marketplace_id.as_ref().map_or("—", |m| m.as_str()),
            s.marketplace_hash
                .as_deref()
                .map_or("—", |h| &h[..h.len().min(12)]),
            s.invocations,
            s.first_invoked_at.to_rfc3339()
        );
    }
    line!(out, "\n## Safety findings\n");
    line!(
        out,
        "| At | Phase | Category | Severity | Scanner | Blocked | Excerpt |\n|---|---|---|---|---|---|---|"
    );
    for s in &b.safety_findings {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            s.created_at.to_rfc3339(),
            s.phase,
            s.category,
            s.severity,
            s.scanner,
            s.blocked,
            cell(s.excerpt.as_deref().unwrap_or(""))
        );
    }
}

fn hook_table(out: &mut String, b: &ConversationBundle) {
    line!(out, "\n## Hook events\n");
    line!(
        out,
        "| At | Event | Tool | Description |\n|---|---|---|---|"
    );
    for e in &b.hook_events {
        line!(
            out,
            "| {} | {} | {} | {} |",
            e.created_at.to_rfc3339(),
            e.event_type,
            cell(e.tool_name.as_deref().unwrap_or("")),
            cell(
                e.description
                    .as_deref()
                    .or(e.prompt_preview.as_deref())
                    .unwrap_or("")
            )
        );
    }
}

// Why: a prompt is often a skill body with headings of its own; quoted, it
// stays inside the turn instead of joining the document's outline.
fn quoted(text: &str) -> String {
    text.lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// Why: one line per table cell — pipes and newlines would break the row, and
// a long reason is in the JSON document in full.
fn cell(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| {
            if c == '\n' || c == '\r' {
                ' '
            } else if c == '|' {
                '¦'
            } else {
                c
            }
        })
        .collect();
    if flat.chars().count() > TABLE_TEXT_CHARS {
        let head: String = flat.chars().take(TABLE_TEXT_CHARS).collect();
        format!("{head}…")
    } else {
        flat
    }
}
