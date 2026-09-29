//! The help modal behind each page's `?`: what every column means and where
//! each number comes from, as an icon-led glossary rather than a paragraph at
//! the page foot. Every page in the section and the Tools / Artifacts pages
//! build a [`HelpView`] here so the wording of a shared figure — a turn, a
//! tool call, an artifact, the completion score — is written once.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct HelpItem {
    pub icon: &'static str,
    pub term: &'static str,
    pub text: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct HelpSection {
    pub heading: &'static str,
    pub items: Vec<HelpItem>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HelpView {
    pub title: String,
    pub intro: &'static str,
    pub sections: Vec<HelpSection>,
    pub guide_href: &'static str,
}

pub(crate) const GUIDE: &str = "/documentation/analysis";

pub(crate) const fn item(icon: &'static str, term: &'static str, text: &'static str) -> HelpItem {
    HelpItem { icon, term, text }
}

impl HelpView {
    pub(crate) fn new(title: impl Into<String>, intro: &'static str) -> Self {
        Self {
            title: title.into(),
            intro,
            sections: Vec::new(),
            guide_href: GUIDE,
        }
    }

    pub(crate) fn section(mut self, heading: &'static str, items: Vec<HelpItem>) -> Self {
        self.sections.push(HelpSection { heading, items });
        self
    }
}

// Why: the figures the conversation record carries, worded once for the
// Conversations list, the conversation detail and the Skills rows.
pub(crate) fn record_items() -> Vec<HelpItem> {
    vec![
        item(
            "chat",
            "Conversation",
            "One gateway context. Turns, tokens, cost, latency and status come from the request log; the row appears once the gateway records a turn and is re-derived within a minute of any plane changing.",
        ),
        item(
            "bolt",
            "Turns",
            "Human prompts the model answered. Side calls the harness made (title generation, summaries) are counted separately and excluded.",
        ),
        item(
            "wrench",
            "Tools",
            "Every tool call — the model's intent from the request, the execution from the ledger. Shown as executed/asked when the two differ; red when one failed.",
        ),
        item(
            "file",
            "Artifacts",
            "The subset of tool calls a person can view or retrieve afterwards: a file edited, written or read; a UI resource; a typed card (table, chart, report); a retained body with a preview. A Bash, Grep or search result is a tool call, not an artifact.",
        ),
        item(
            "token",
            "Tokens",
            "Input plus output tokens; the bar beneath is the share of everything the models read that came from cache.",
        ),
        item(
            "coins",
            "Cost",
            "Priced spend on every request in the conversation, from the gateway's rate cards.",
        ),
        item(
            "alert",
            "Err · ⛔ · ⚑",
            "Failed requests, tool calls the governance chain denied, and safety scanner findings on the conversation's requests.",
        ),
        item(
            "gauge",
            "Latency",
            "The conversation's own p95 turn latency; the tile is the 95th percentile of those.",
        ),
        item("clock", "Duration", "First request to last activity."),
        item(
            "skill",
            "Skills",
            "Solid chips were reported by the harness hook and are evidence; outlined ones the judge read from the transcript.",
        ),
    ]
}

pub(crate) fn judge_items() -> Vec<HelpItem> {
    vec![
        item(
            "sparkle",
            "AI",
            "The last column: one circle for the judge's 0–100 completion score — did the assistant deliver what the person originally asked for. Green from 80, amber from 50, red below; a hollow ring means no verdict yet, a pulsing one that a verdict is on its way. Hover or focus it for the summary and rationale.",
        ),
        item(
            "sparkle",
            "Title · intent · outcome",
            "The judge's label: a title, one intent (development, business analysis, operations, admin & config, writing & comms, research & learning, other) and an outcome (achieved, partial, abandoned, unclear). The summary opens on hover.",
        ),
        item(
            "clock",
            "When it runs",
            "In production the judge runs by itself: the conversation_judge job reads a conversation once it has been quiet or its session has ended, sends the transcript (credentials redacted) to the configured model with the schema enforced by the provider, and records one verdict. Where the profile keeps it manual (development), the hollow circle is the button that asks, and Judge N unjudged / Judge selected ask for many at once.",
        ),
        item(
            "shield",
            "Its own spend",
            "Judge calls are audited under the job's identity and never counted as conversations.",
        ),
    ]
}

pub(crate) fn conversations_help() -> HelpView {
    HelpView::new(
        "Conversations — what the columns mean",
        "Every conversation is a gateway context. Nothing on this page is an opinion except the judge's one label; every other figure is a count, sum or percentile over rows the gateway, the hooks, the tool ledger and the governance spine wrote.",
    )
    .section("The record", record_items())
    .section("The judge", judge_items())
    .section(
        "Reading the page",
        vec![
            item("spark", "Trend slots", "The small line beside a tile's figure is that figure per bucket over the window; the row sparkline is tokens per turn over the conversation's last 24 turns."),
            item("layers", "Breakdown", "One dimension at a time over the same filtered set the table pages through, so the buckets add up to the tiles. Each row links into the list narrowed to it and can be downloaded."),
            item("filter", "Filters", "Each pill lists only the values the current set contains, with counts. Chips show what is applied; the download glyph on a chip exports exactly those rows."),
            item("export", "Export", "Tick rows for Export selected, or use the header's Export for the whole filtered set with your choice of columns."),
        ],
    )
}

pub(crate) fn conversation_detail_help() -> HelpView {
    HelpView::new(
        "This conversation — where each number comes from",
        "The header facts and tiles are the conversation's record row, refreshed as the page loads; the tables beneath are the raw rows on each plane.",
    )
    .section("The record", record_items())
    .section("The judge", judge_items())
    .section(
        "The planes",
        vec![
            item("layers", "Turn ledger", "Every gateway request in order — human turns and the harness's side calls (dimmed) — with its tokens by kind, cost, latency and status."),
            item("wrench", "Tools", "Every tool call with its state: intended (the model asked), executed (the ledger has a run), failed, or unattested (a hook reported a run no request asked for)."),
            item("file", "Artifacts", "The tool calls that produced something viewable: kind, title or path, and a preview where a body was retained."),
            item("shield", "Governance", "Every allow / warn / deny decision the chain recorded for the conversation or its harness session."),
            item("alert", "Safety", "Gateway scanner findings on the conversation's requests and responses."),
        ],
    )
}

pub(crate) fn skills_help() -> HelpView {
    HelpView::new(
        "Skills — what the views mean",
        "An invocation is a harness hook event — a slash command or the Skill tool — keyed plugin:skill; the plugin is the key's prefix, whichever plugin carries the hooks. Everything else is read from the conversations whose Claude Code session invoked the skill, so a number here is the same number on the Conversations page.",
    )
    .section(
        "Scope and views",
        vec![
            item("calendar", "Window · Marketplace", "The scope above the tabs. Every view honours both; pick a marketplace to see only its skills, its chart and its row."),
            item("layers", "Overview", "One row per marketplace: entitled → installed → active, skills used of those declared, the window's invocations, conversations, cost and judge mean."),
            item("bolt", "Activity", "Invocations over the window as one chart — per day, per week once a year is shown — and the skills it draws, each a link into the table."),
            item("skill", "Skills", "The table: one row per skill under its marketplace and plugin, with the client, sort and search filters."),
        ],
    )
    .section(
        "The skill row",
        vec![
            item("bolt", "Invocations", "Hook-reported invocations in the window, slash and tool. The sparkline is invocations per day over the last fourteen."),
            item("people", "People / entitled", "Distinct invokers over the people the access-control rules reach for the marketplaces carrying the skill."),
            item("download", "Installs", "Distinct consumers holding a verified installation receipt for the skill resource, on any host."),
            item("chat", "Conv.", "Gateway conversations whose harness session invoked the skill; the tokens, cost, tools, artifacts and errors beside it are theirs."),
            item("wrench", "Tools · Artifacts", "Tool calls in those conversations, and the subset that produced something viewable (files, cards, UI, bodies)."),
            item("sparkle", "AI", "The last column: mean judge completion over the judged conversations as one coloured circle with the score, then judged/total."),
            item("skill", "N unused", "The toolbar pill lists configured skills nobody invoked in the window, with the people entitled to each — the gap between what is shipped and what is used."),
        ],
    )
    .section(
        "Adoption",
        vec![
            item("layers", "Entitled → installed → active", "Per marketplace: people the rules reach, consumers with a verified receipt for any of its skills, and distinct people who invoked one in the window. The install rate counts only installed people the rules entitle; the rest are shown as \"outside\" so no rate can pass 100%."),
            item("calendar", "Versions", "The marketplace's current content hash and how many hashes it has served; each links to the version history."),
        ],
    )
    .section("The judge", judge_items())
}

pub(crate) fn skill_detail_help() -> HelpView {
    HelpView::new(
        "This skill — where each number comes from",
        "Invocations are hook events for this one skill; every other figure is read from the conversations whose session invoked it.",
    )
    .section(
        "The page",
        vec![
            item("calendar", "Per day", "Invocations, people and the conversations' tokens, cost and errors, one point per day of the window — zero days included."),
            item("layers", "Breakdown", "Invocations by model, client, group, project, person, marketplace version or outcome of the conversations that invoked it."),
            item("chat", "Conversations", "Every conversation in the window that invoked the skill, with the same columns and judge cell as the Conversations page."),
        ],
    )
    .section("The record", record_items())
    .section("The judge", judge_items())
}
