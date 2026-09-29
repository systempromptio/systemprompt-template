//! The filter ribbon of the Tools and Artifacts pages: one pill per
//! dimension holding the values the filtered set contains, and a chip per
//! active filter carrying the link that removes it and the export of what
//! it selects.

use systemprompt::identifiers::UserId;

use super::query::{Lens, ToolsQuery};
use crate::handlers::ssr::analysis::ribbon::{RibbonGroupView, RibbonView};
use crate::repositories::analysis::tools::{
    ArtifactKind, ToolActivityResult, ToolFacet, ToolState,
};

fn facet(items: &[ToolFacet]) -> impl Iterator<Item = (&str, String, i64)> {
    items.iter().map(|f| {
        (
            f.value.as_str(),
            f.label.clone().unwrap_or_else(|| f.value.clone()),
            f.calls,
        )
    })
}

// Why: the artifact-kind pill, drawn on both lenses under different labels.
fn kind_group(label: &'static str, q: &ToolsQuery, data: &ToolActivityResult) -> RibbonGroupView {
    RibbonGroupView::single(
        "artifact",
        label,
        "file",
        q.artifact.as_deref(),
        data.kinds.iter().map(|k| {
            let kind = ArtifactKind::parse_artifact_kind(Some(k.value.as_str()));
            (
                k.value.as_str(),
                kind.map_or_else(|| k.value.clone(), |k| k.label().to_owned()),
                k.calls,
            )
        }),
    )
}

// Why: state and decision only mean something for every call; the
// Artifacts lens is already the executed, successful subset.
fn tools_only_groups(q: &ToolsQuery, data: &ToolActivityResult) -> Vec<RibbonGroupView> {
    vec![
        RibbonGroupView::single(
            "state",
            "State",
            "check",
            q.state.as_deref(),
            ToolState::ALL
                .iter()
                .map(|(s, l)| (s.as_str(), (*l).to_owned(), 0)),
        ),
        RibbonGroupView::single(
            "decision",
            "Decision",
            "shield",
            q.decision.as_deref(),
            facet(&data.decisions),
        ),
        kind_group("Artifact", q, data),
    ]
}

pub(crate) fn ribbon(lens: Lens, q: &ToolsQuery, data: &ToolActivityResult) -> RibbonView {
    let user_id = q.user_id();
    let mut view = RibbonView::new(lens.base_url(), lens.base_url()).preserve(&q.preserved(&[
        "user_id", "tool", "server", "kind", "state", "decision", "skill", "client", "artifact",
        "q", "page", "ids",
    ]));
    if lens == Lens::Artifacts {
        view = view.group(kind_group("Kind", q, data));
    }
    view = view
        .group(RibbonGroupView::single(
            "tool",
            "Tool",
            "wrench",
            q.tool.as_deref(),
            facet(&data.tools),
        ))
        .group(RibbonGroupView::single(
            "server",
            "Server",
            "plug",
            q.server.as_deref(),
            facet(&data.servers),
        ))
        .group(RibbonGroupView::fixed(
            "kind",
            "Origin",
            "terminal",
            q.kind.as_deref(),
            &[
                ("builtin", "Builtin harness tool"),
                ("mcp", "MCP server tool"),
            ],
        ));
    if lens == Lens::Tools {
        for group in tools_only_groups(q, data) {
            view = view.group(group);
        }
    }
    view = view
        .group(RibbonGroupView::single(
            "user_id",
            "Person",
            "user",
            user_id.as_ref().map(UserId::as_str),
            facet(&data.users),
        ))
        .group(RibbonGroupView::single(
            "client",
            "Client",
            "model",
            q.client.as_deref(),
            facet(&data.clients),
        ))
        .group(RibbonGroupView::single(
            "skill",
            "Skill",
            "skill",
            q.skill.as_deref(),
            facet(&data.skills),
        ))
        .search("q", q.q.as_deref(), "tool, server, path, command or title");
    chips(lens, q, view)
}

fn chips(lens: Lens, q: &ToolsQuery, mut view: RibbonView) -> RibbonView {
    let user = q.user_id();
    let active: [(&'static str, &'static str, Option<&str>); 11] = [
        ("Kind", "artifact", q.artifact.as_deref()),
        ("Tool", "tool", q.tool.as_deref()),
        ("Server", "server", q.server.as_deref()),
        ("Origin", "kind", q.kind.as_deref()),
        ("State", "state", q.state.as_deref()),
        ("Decision", "decision", q.decision.as_deref()),
        ("Person", "user_id", user.as_ref().map(UserId::as_str)),
        ("Client", "client", q.client.as_deref()),
        ("Skill", "skill", q.skill.as_deref()),
        ("Conversation", "context", q.context.as_deref()),
        ("Session", "session", q.session.as_deref()),
    ];
    for (label, name, value) in active {
        if let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) {
            view = view.chip(
                label,
                value,
                q.without(lens, name),
                Some(q.export_href(lens, name, value)),
            );
        }
    }
    if let Some(text) = q.q.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        view = view.chip("Search", text, q.without(lens, "q"), None);
    }
    view
}
