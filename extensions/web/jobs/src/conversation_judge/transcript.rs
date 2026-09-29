//! Flattens a rendered conversation into the text the judge reads, inside a
//! character budget: the opening turns say what was wanted, the closing turns
//! say how it ended, and the middle is elided when it does not fit.

use systemprompt_web_admin::test_support::{ConversationView, StepView, TurnView};

// Why: roughly four characters per token. The budget is on characters because
// that is what the transcript is made of; the parameter is in tokens because
// that is how the model is priced.
const CHARS_PER_TOKEN: usize = 4;
const HEAD_TURNS: usize = 6;
const TAIL_TURNS: usize = 10;
const ASSISTANT_CHARS: usize = 1_500;
const TOOL_INPUT_CHARS: usize = 200;
const PROMPT_CHARS: usize = 4_000;

/// What the judge is told about the conversation before the transcript.
#[derive(Debug, Clone, Default)]
pub struct TranscriptMeta {
    pub client_kind: String,
    pub model: Option<String>,
    pub turn_count: i64,
    pub tool_call_count: i64,
    pub error_count: i64,
    pub duration_minutes: i64,
    pub hooked_skills: Vec<String>,
}

#[must_use]
pub fn render_transcript(
    meta: &TranscriptMeta,
    view: &ConversationView,
    token_budget: usize,
) -> String {
    let budget = token_budget.saturating_mul(CHARS_PER_TOKEN).max(2_000);
    let mut out = String::new();
    write_meta(&mut out, meta);

    let turns: Vec<&TurnView> = view.threads.iter().flat_map(|t| t.turns.iter()).collect();
    let rendered: Vec<String> = turns.iter().map(|t| render_turn(t)).collect();
    let body = fit_turns(&rendered, budget.saturating_sub(out.len()));
    out.push_str("\nTRANSCRIPT\n");
    out.push_str(&body);
    out
}

fn write_meta(out: &mut String, meta: &TranscriptMeta) {
    out.push_str("CONVERSATION METADATA\n");
    out.push_str(&format!("client: {}\n", meta.client_kind));
    if let Some(model) = &meta.model {
        out.push_str(&format!("model: {model}\n"));
    }
    out.push_str(&format!(
        "turns: {} · tool calls: {} · failed requests: {} · duration: {} min\n",
        meta.turn_count, meta.tool_call_count, meta.error_count, meta.duration_minutes
    ));
    if meta.hooked_skills.is_empty() {
        out.push_str("skills reported by the harness: none\n");
    } else {
        out.push_str(&format!(
            "skills reported by the harness: {}\n",
            meta.hooked_skills.join(", ")
        ));
    }
}

fn render_turn(turn: &TurnView) -> String {
    let mut s = format!("USER: {}\n", clip(&turn.prompt, PROMPT_CHARS));
    for step in &turn.steps {
        render_step(&mut s, step);
    }
    s
}

fn render_step(s: &mut String, step: &StepView) {
    if step.is_tool {
        let name = step.tool_name.as_deref().unwrap_or("tool");
        let input = step.tool_input_pretty.as_deref().unwrap_or("");
        s.push_str(&format!(
            "TOOL {name}: {}\n",
            clip(&input.replace('\n', " "), TOOL_INPUT_CHARS)
        ));
    } else if let Some(text) = step
        .text
        .as_deref()
        .filter(|t| step.is_assistant && !t.trim().is_empty())
    {
        s.push_str(&format!("ASSISTANT: {}\n", clip(text, ASSISTANT_CHARS)));
    }
}

// Why: keeps every turn when they fit, otherwise the first `HEAD_TURNS` and
// last `TAIL_TURNS` with an elision marker, shrinking the tail first if even
// that is too long.
fn fit_turns(rendered: &[String], budget: usize) -> String {
    let total: usize = rendered.iter().map(String::len).sum();
    if total <= budget {
        return rendered.concat();
    }
    let n = rendered.len();
    let head = HEAD_TURNS.min(n);
    let mut tail = TAIL_TURNS.min(n.saturating_sub(head));
    loop {
        let omitted = n.saturating_sub(head + tail);
        let marker = format!("[... {omitted} turns omitted ...]\n");
        let size: usize = rendered[..head].iter().map(String::len).sum::<usize>()
            + marker.len()
            + rendered[n - tail..].iter().map(String::len).sum::<usize>();
        if size <= budget || tail == 0 {
            let mut out = rendered[..head].concat();
            if omitted > 0 {
                out.push_str(&marker);
            }
            out.push_str(&rendered[n - tail..].concat());
            return hard_cap(out, budget);
        }
        tail -= 1;
    }
}

fn hard_cap(mut text: String, budget: usize) -> String {
    if text.len() > budget {
        let cut = text
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i <= budget.saturating_sub(24))
            .last()
            .unwrap_or(0);
        text.truncate(cut);
        text.push_str("\n[... truncated ...]\n");
    }
    text
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut s: String = text.chars().take(max).collect();
    s.push('…');
    s
}
