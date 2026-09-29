//! Which column the projects listing is ordered by, and the header links
//! that change it.
//!
//! The sort key is validated against a fixed list rather than passed through,
//! so a query string can only choose an ordering the page actually offers.

use super::super::types::{ProjectSortHeaders, SortHeaderView};
use super::BASE_URL;
use crate::repositories::projects::usage::ProjectRollup;

use super::list::ListQuery;
use super::pct;
use crate::handlers::ssr::types::table::SortColumn;

const SORT_KEYS: [&str; 11] = [
    "name",
    "members",
    "groups",
    "requests",
    "tokens",
    "cost",
    "models",
    "clients",
    "tools",
    "skills",
    "artifacts",
];

pub(super) fn sort_key(requested: Option<&str>) -> &'static str {
    requested
        .and_then(|r| SORT_KEYS.iter().find(|k| **k == r).copied())
        .unwrap_or("cost")
}

pub(super) fn sort_rows(rows: &mut [ProjectRollup], key: &str, descending: bool) {
    match key {
        "name" => rows.sort_by_key(|a| a.name.to_lowercase()),
        "members" => rows.sort_by_key(|r| r.member_count),
        "groups" => rows.sort_by_key(|r| r.group_count),
        "requests" => rows.sort_by_key(|r| r.requests),
        "tokens" => rows.sort_by_key(|r| r.tokens),
        "models" => rows.sort_by_key(|r| r.models_used),
        "clients" => rows.sort_by_key(|r| r.clients_used),
        "tools" => rows.sort_by_key(|r| pct(r.tool_success, r.tool_calls)),
        "skills" => rows.sort_by_key(|r| r.skills_used),
        "artifacts" => rows.sort_by_key(|r| r.artifacts),
        _ => rows.sort_by_key(|r| r.cost_microdollars),
    }
    if descending {
        rows.reverse();
    }
}

fn preserved(query: &ListQuery) -> String {
    query
        .q
        .as_ref()
        .filter(|q| !q.is_empty())
        .map_or_else(String::new, |q| format!("&q={}", urlencoding::encode(q)))
}

pub(super) fn page_url(query: &ListQuery) -> String {
    let dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    format!(
        "{BASE_URL}?sort={}&dir={dir}{}",
        sort_key(query.sort.as_deref()),
        preserved(query)
    )
}

// Why: one column's static half — the key the URL carries, the words on the
// header, the class that sizes it, and the sentence its title explains it
// with. The dynamic half (url, active, indicator) is computed per request.
const NUM: &str = "sp-table__cell--num";
const MIX: &str = "sp-p-projects__col-mix";

const fn spec(
    key: &'static str,
    label: &'static str,
    class: &'static str,
    hint: &'static str,
) -> SortColumn {
    SortColumn {
        key,
        label,
        class,
        hint,
    }
}

const NAME: SortColumn = spec(
    "name",
    "Project",
    "sp-p-projects__col-name",
    "The project id and what it is for",
);
const MEMBERS: SortColumn = spec(
    "members",
    "Members",
    NUM,
    "People holding membership, and how many were active in the window",
);
const GROUPS: SortColumn = spec(
    "groups",
    "Groups",
    NUM,
    "Distinct groups the members belong to — who feeds this project",
);
const REQUESTS: SortColumn = spec(
    "requests",
    "Requests",
    NUM,
    "Gateway requests attributed to this project, exclusively",
);
const TOKENS: SortColumn = spec(
    "tokens",
    "Tokens",
    NUM,
    "Every token the window billed — input, output, cache and reasoning",
);
const COST: SortColumn = spec("cost", "Cost", NUM, "Billed cost over the window");
const MODELS: SortColumn = spec(
    "models",
    "Models",
    MIX,
    "The busiest model, and how many distinct models the project used",
);
const CLIENTS: SortColumn = spec(
    "clients",
    "Agents",
    MIX,
    "The coding agent behind most requests (Claude Code, Codex, desktop…), and how many distinct ones",
);
const TOOLS: SortColumn = spec(
    "tools",
    "Tools",
    NUM,
    "Share of MCP tool calls that returned success, over the calls made",
);
const SKILLS: SortColumn = spec(
    "skills",
    "Skills",
    NUM,
    "Distinct skills the project's people invoked",
);
const ARTIFACTS: SortColumn = spec(
    "artifacts",
    "Artifacts",
    NUM,
    "Artifacts the project's people produced through MCP tools",
);

pub(super) fn sort_headers(
    query: &ListQuery,
    active: &str,
    descending: bool,
) -> ProjectSortHeaders {
    let tail = preserved(query);
    let header = |c: &SortColumn| {
        let is_active = c.key == active;
        let next = SortHeaderView::next_dir(is_active, descending);
        SortHeaderView::new(
            (c.label, c.class, c.hint),
            format!("{BASE_URL}?sort={}&dir={next}{tail}", c.key),
            is_active,
            descending,
        )
    };
    ProjectSortHeaders {
        name: header(&NAME),
        members: header(&MEMBERS),
        groups: header(&GROUPS),
        requests: header(&REQUESTS),
        tokens: header(&TOKENS),
        cost: header(&COST),
        models: header(&MODELS),
        clients: header(&CLIENTS),
        tools: header(&TOOLS),
        skills: header(&SKILLS),
        artifacts: header(&ARTIFACTS),
    }
}
