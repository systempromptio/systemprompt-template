//! The `?` glossaries of the Tools and Artifacts pages, worded once so the
//! two pages and the conversation record describe a call the same way.

use crate::handlers::ssr::analysis::help::{HelpItem, HelpView, item};

fn call_items() -> Vec<HelpItem> {
    vec![
        item(
            "wrench",
            "Tool call",
            "One row of the tool-call ledger: the model's intent from the gateway request, the execution the platform or a harness hook recorded, and the typed result, joined on the client tool_use_id.",
        ),
        item(
            "terminal",
            "Builtin",
            "A harness tool — Bash, Edit, Read, Grep, Skill … — reported by the Claude Code or OpenCode hook. Its server is the hook itself.",
        ),
        item(
            "plug",
            "MCP",
            "A tool an MCP server exposes, run in-process, through the proxy, or reported by a hook as mcp__<server>__<tool>.",
        ),
        item(
            "search",
            "Input",
            "What the call was about in one line: the file path, the command, the search pattern or query, else the first key of the input.",
        ),
        item(
            "check",
            "State",
            "Executed: a run joined to a request. Intended: the model asked, nothing ran. Unattested: something ran that no request asked for. Failed: the run errored, timed out or returned an error result.",
        ),
        item(
            "shield",
            "Decision",
            "The governance chain's allow / warn / deny keyed to this call by tool_use_id; a dash when the chain never saw it.",
        ),
        item(
            "gauge",
            "Duration",
            "Execution time from the run's own record; the tile is its 95th percentile.",
        ),
        item(
            "skill",
            "Skill",
            "The skill most recently invoked in the same harness session before the call.",
        ),
    ]
}

pub(crate) fn artifact_items() -> Vec<HelpItem> {
    vec![
        item(
            "file",
            "Artifact",
            "The subset of tool calls a person can view or retrieve afterwards. A shell command, a search, a listing or an untyped result is a tool call, not an artifact — the same rule counts them on every page.",
        ),
        item(
            "file",
            "File",
            "An Edit, Write, MultiEdit, NotebookEdit or Read naming a file path; the path is the title.",
        ),
        item(
            "card",
            "Card",
            "A result an MCP server declared with a type — table, chart, report, presentation card — rendered by core's renderers.",
        ),
        item(
            "ui",
            "UI",
            "A result carrying an MCP Apps ui:// resource the host renders.",
        ),
        item(
            "body",
            "Body",
            "A structured result whose body is retained, content-addressed, in the payload store and can be previewed.",
        ),
        item(
            "search",
            "Preview",
            "Opens the stored body rendered as a host would render it, in a sandboxed frame; a file shows its path chip instead.",
        ),
        item(
            "alert",
            "Findings · redactions",
            "Scanner findings recorded at ingestion and secret spans removed before the body was stored.",
        ),
    ]
}

pub(crate) fn tools_help() -> HelpView {
    HelpView::new(
        "Tools — what the columns mean",
        "Every tool call the platform saw in the window, whichever client made it. Nothing here is an opinion: each figure is a count or percentile over ledger rows.",
    )
    .section("The call", call_items())
    .section("Artifacts", artifact_items())
    .section(
        "Reading the page",
        vec![
            item("spark", "Trend slots", "The small line beside a tile is that figure per bucket over the window; a 24-hour window buckets by hour, longer ones by day, zero buckets included."),
            item("layers", "Breakdown", "One dimension at a time over the same filtered set the table pages through; each row links into the list narrowed to it and downloads as CSV."),
            item("filter", "Filters", "Each pill lists only the values the current set contains, with counts; the download glyph on a chip exports exactly those rows."),
            item("export", "Export", "Tick rows for Export selected, or use the header's Export for the whole filtered set."),
        ],
    )
}

pub(crate) fn artifacts_help() -> HelpView {
    HelpView::new(
        "Artifacts — what the columns mean",
        "Every tool result a person can view or retrieve, from any client, with where it was seen and how it was joined to the call that produced it.",
    )
    .section("Artifacts", artifact_items())
    .section("The call behind it", call_items())
    .section(
        "Reading the page",
        vec![
            item("spark", "Trend slots", "The small line beside a tile is that figure per bucket over the window, zero buckets included."),
            item("layers", "Breakdown", "By kind, tool, server, person or skill over the same filtered set; each row links into the list and downloads as CSV."),
            item("export", "Export", "Tick rows for Export selected, or the header's Export for the filtered set."),
        ],
    )
}
