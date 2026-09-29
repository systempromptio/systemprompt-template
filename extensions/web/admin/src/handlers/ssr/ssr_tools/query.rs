//! The query string `/admin/tools` and `/admin/artifacts` bind, and the
//! links that carry it: the window, every filter, the breakdown dimension,
//! sort and page. Both pages share the type; `Lens` names which one is
//! asking so links land back on the right page.

use serde::Deserialize;
use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use crate::repositories::analysis::tools::{
    ArtifactKind, ToolActivityFilter, ToolBreakdownBy, ToolSort, ToolState,
};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

// Why: which page is asking — the Artifacts page is the Tools page narrowed
// to rows with an artifact kind, so one query type serves both and only the
// base URL and the default breakdown differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lens {
    Tools,
    Artifacts,
}

impl Lens {
    pub(crate) const fn base_url(self) -> &'static str {
        match self {
            Self::Tools => "/admin/tools",
            Self::Artifacts => "/admin/artifacts",
        }
    }

    pub(crate) const fn dataset(self) -> &'static str {
        match self {
            Self::Tools => "tools",
            Self::Artifacts => "artifacts",
        }
    }

    const fn default_breakdown(self) -> ToolBreakdownBy {
        match self {
            Self::Tools => ToolBreakdownBy::Tool,
            Self::Artifacts => ToolBreakdownBy::Kind,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ToolsQuery {
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<UserId>,
    pub tool: Option<String>,
    pub server: Option<String>,
    pub kind: Option<String>,
    pub state: Option<String>,
    pub decision: Option<String>,
    pub context: Option<String>,
    pub session: Option<String>,
    pub skill: Option<String>,
    pub client: Option<String>,
    pub artifact: Option<String>,
    pub q: Option<String>,
    pub by: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    pub ids: Option<String>,
}

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

fn owned(value: Option<&str>) -> Option<String> {
    trimmed(value).map(str::to_owned)
}

impl ToolsQuery {
    pub(crate) fn range(&self) -> TimeRange {
        parse_time_range(&TimeRangeQuery {
            from: self.from.clone(),
            to: self.to.clone(),
            preset: self.preset.clone(),
        })
    }

    pub(crate) const fn preset_label(&self, range: TimeRange) -> &'static str {
        if self.from.is_some() && self.to.is_some() {
            return "custom";
        }
        range.preset.as_str()
    }

    pub(crate) fn user_id(&self) -> Option<UserId> {
        self.user_id
            .clone()
            .filter(|u| !u.as_str().trim().is_empty())
    }

    pub(crate) fn builtin(&self) -> Option<bool> {
        match trimmed(self.kind.as_deref()) {
            Some("builtin") => Some(true),
            Some("mcp") => Some(false),
            _ => None,
        }
    }

    pub(crate) fn state(&self) -> Option<ToolState> {
        ToolState::parse_tool_state(trimmed(self.state.as_deref()))
    }

    pub(crate) fn artifact_kind(&self) -> Option<ArtifactKind> {
        ArtifactKind::parse_artifact_kind(trimmed(self.artifact.as_deref()))
    }

    pub(crate) fn ids(&self) -> Option<Vec<String>> {
        let ids: Vec<String> = self
            .ids
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        (!ids.is_empty()).then_some(ids)
    }

    pub(crate) fn breakdown(&self, lens: Lens) -> ToolBreakdownBy {
        match trimmed(self.by.as_deref()) {
            None => lens.default_breakdown(),
            some => ToolBreakdownBy::parse_tool_breakdown(some),
        }
    }

    pub(crate) fn sort(&self) -> ToolSort {
        ToolSort::parse_tool_sort(trimmed(self.sort.as_deref()))
    }

    pub(crate) fn descending(&self) -> bool {
        self.dir.as_deref() != Some("asc")
    }

    pub(crate) fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }

    // Why: the repository filter from the query, the resolved window and the
    // caller's scope; the Artifacts lens adds the one narrowing that makes
    // it that page.
    pub(crate) fn filter(
        &self,
        lens: Lens,
        range: TimeRange,
        subject_ids: Option<&[String]>,
    ) -> ToolActivityFilter {
        ToolActivityFilter {
            since: Some(range.from),
            until: Some(range.to),
            subject_ids: subject_ids.map(<[String]>::to_vec),
            user_id: self.user_id(),
            tool: owned(self.tool.as_deref()),
            server: owned(self.server.as_deref()),
            builtin: self.builtin(),
            state: self.state(),
            decision: owned(self.decision.as_deref()),
            context: owned(self.context.as_deref()),
            session: owned(self.session.as_deref()),
            skill: owned(self.skill.as_deref()),
            client_kind: owned(self.client.as_deref()),
            artifact_kind: self.artifact_kind(),
            artifacts_only: lens == Lens::Artifacts,
            free_text: owned(self.q.as_deref()),
            ids: self.ids(),
        }
    }

    pub(crate) fn pairs(&self) -> [(&'static str, Option<&str>); 21] {
        [
            ("preset", self.preset.as_deref()),
            ("from", self.from.as_deref()),
            ("to", self.to.as_deref()),
            ("group", self.group.as_deref()),
            ("project", self.project.as_deref()),
            ("user_id", self.user_id.as_ref().map(UserId::as_str)),
            ("tool", self.tool.as_deref()),
            ("server", self.server.as_deref()),
            ("kind", self.kind.as_deref()),
            ("state", self.state.as_deref()),
            ("decision", self.decision.as_deref()),
            ("context", self.context.as_deref()),
            ("session", self.session.as_deref()),
            ("skill", self.skill.as_deref()),
            ("client", self.client.as_deref()),
            ("artifact", self.artifact.as_deref()),
            ("q", self.q.as_deref()),
            ("by", self.by.as_deref()),
            ("sort", self.sort.as_deref()),
            ("dir", self.dir.as_deref()),
            ("ids", self.ids.as_deref()),
        ]
    }

    pub(crate) fn preserved(&self, drop: &[&str]) -> Vec<(String, String)> {
        self.pairs()
            .iter()
            .filter(|(name, _)| !drop.contains(name))
            .filter_map(|(name, val)| trimmed(*val).map(|v| ((*name).to_owned(), v.to_owned())))
            .collect()
    }

    pub(crate) fn query_string(&self, drop: &[&str]) -> String {
        self.preserved(drop)
            .iter()
            .map(|(name, value)| format!("{name}={}", urlencode(value)))
            .collect::<Vec<_>>()
            .join("&")
    }

    pub(crate) fn link_prefix(&self, lens: Lens, drop: &[&str]) -> String {
        let qs = self.query_string(drop);
        if qs.is_empty() {
            format!("{}?", lens.base_url())
        } else {
            format!("{}?{qs}&", lens.base_url())
        }
    }

    // Why: the current filters narrowed by one more `name=value`, back on
    // page one — what a breakdown row, a facet and a chip's export offer.
    pub(crate) fn narrowed(&self, lens: Lens, name: &str, value: &str) -> String {
        format!(
            "{}{name}={}",
            self.link_prefix(lens, &[name, "page"]),
            urlencode(value)
        )
    }

    pub(crate) fn without(&self, lens: Lens, name: &str) -> String {
        self.link_prefix(lens, &[name, "page"])
            .trim_end_matches(['?', '&'])
            .to_owned()
    }

    pub(crate) fn export_href(&self, lens: Lens, name: &str, value: &str) -> String {
        let mut qs = self.query_string(&[name, "page", "ids"]);
        if !qs.is_empty() {
            qs.push('&');
        }
        format!(
            "/admin/export/{}?{qs}{name}={}&format=csv",
            lens.dataset(),
            urlencode(value)
        )
    }
}
