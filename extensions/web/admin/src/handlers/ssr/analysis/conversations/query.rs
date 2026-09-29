//! The query string `/admin/analysis/conversations` binds, and the links that
//! carry it: every filter, the breakdown dimension, sort and page.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use crate::repositories::analysis::conversations::{
    BreakdownBy, FactSort, FlagFilter, JudgedFilter,
};

pub(crate) const BASE_URL: &str = "/admin/analysis/conversations";
// Why: this page's `all` reaches past the live window contract, whose widest
// preset is 90 days; the export offers that nearest window.
pub(crate) fn export_preset(params: &ConversationAnalysisQuery) -> &'static str {
    match params.since_label() {
        "24h" => "24h",
        "7d" => "7d",
        "90d" | "all" => "90d",
        _ => "30d",
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ConversationAnalysisQuery {
    pub group: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<UserId>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub skill: Option<String>,
    pub judged: Option<String>,
    pub model: Option<String>,
    pub client: Option<String>,
    pub flag: Option<String>,
    pub q: Option<String>,
    pub since: Option<String>,
    // Why: the shell's scope bar emits `?preset=`; read as a fallback so it
    // is not silently ignored.
    pub preset: Option<String>,
    pub by: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    // Why: a comma-joined selection of context ids from the bulk bar; the
    // export dialog appends it so a download holds exactly the ticked rows.
    pub ids: Option<String>,
    // Why: `all` lists conversations with no turn too; hidden by default.
    pub show: Option<String>,
}

impl ConversationAnalysisQuery {
    fn trimmed(value: Option<&str>) -> Option<&str> {
        value.map(str::trim).filter(|v| !v.is_empty())
    }

    pub(crate) fn since_label(&self) -> &'static str {
        let raw =
            Self::trimmed(self.since.as_deref()).or_else(|| Self::trimmed(self.preset.as_deref()));
        match raw {
            Some("1d" | "24h") => "24h",
            Some("7d") => "7d",
            Some("90d") => "90d",
            Some("all") => "all",
            _ => "30d",
        }
    }

    pub(crate) fn since_datetime(&self) -> Option<DateTime<Utc>> {
        let preset = crate::util::time_range::TimeRangePreset::parse(self.since_label())?;
        Some(Utc::now() - preset.duration()?)
    }

    pub(crate) fn user_id(&self) -> Option<UserId> {
        self.user_id
            .clone()
            .filter(|u| !u.as_str().trim().is_empty())
    }

    pub(crate) fn category(&self) -> Option<String> {
        Self::trimmed(self.category.as_deref()).map(str::to_owned)
    }

    pub(crate) fn outcome(&self) -> Option<String> {
        Self::trimmed(self.outcome.as_deref()).map(str::to_owned)
    }

    pub(crate) fn skill(&self) -> Option<String> {
        Self::trimmed(self.skill.as_deref()).map(str::to_owned)
    }

    pub(crate) fn model(&self) -> Option<String> {
        Self::trimmed(self.model.as_deref()).map(str::to_owned)
    }

    pub(crate) fn client(&self) -> Option<String> {
        Self::trimmed(self.client.as_deref()).map(str::to_owned)
    }

    pub(crate) fn judged(&self) -> Option<JudgedFilter> {
        JudgedFilter::parse_judged_filter(Self::trimmed(self.judged.as_deref()))
    }

    pub(crate) fn show_all(&self) -> bool {
        Self::trimmed(self.show.as_deref()) == Some("all")
    }

    pub(crate) fn flag(&self) -> Option<FlagFilter> {
        FlagFilter::parse_flag_filter(Self::trimmed(self.flag.as_deref()))
    }

    pub(crate) fn free_text(&self) -> Option<String> {
        Self::trimmed(self.q.as_deref()).map(str::to_owned)
    }

    pub(crate) fn context_ids(&self) -> Option<Vec<String>> {
        let ids: Vec<String> = self
            .ids
            .as_deref()?
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        (!ids.is_empty()).then_some(ids)
    }

    pub(crate) fn breakdown(&self) -> BreakdownBy {
        BreakdownBy::from_breakdown_param(self.by.as_deref())
    }

    pub(crate) fn sort(&self) -> FactSort {
        FactSort::parse_fact_sort(self.sort.as_deref())
    }

    pub(crate) fn descending(&self) -> bool {
        self.dir.as_deref() != Some("asc")
    }

    pub(crate) fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }

    fn pairs(&self) -> [(&'static str, Option<&str>); 18] {
        [
            ("group", self.group.as_deref()),
            ("project", self.project.as_deref()),
            ("user_id", self.user_id.as_ref().map(UserId::as_str)),
            ("category", self.category.as_deref()),
            ("outcome", self.outcome.as_deref()),
            ("skill", self.skill.as_deref()),
            ("judged", self.judged.as_deref()),
            ("model", self.model.as_deref()),
            ("client", self.client.as_deref()),
            ("flag", self.flag.as_deref()),
            ("q", self.q.as_deref()),
            ("since", self.since.as_deref()),
            ("preset", self.preset.as_deref()),
            ("by", self.by.as_deref()),
            ("sort", self.sort.as_deref()),
            ("dir", self.dir.as_deref()),
            ("ids", self.ids.as_deref()),
            ("show", self.show.as_deref()),
        ]
    }

    // Why: the filters alone as a query string — what a judge-all or a
    // report request carries so the server re-reads the same set.
    pub(crate) fn query_string(&self) -> String {
        self.link_prefix(&["page", "ids"])
            .split_once('?')
            .map(|(_, q)| q.trim_end_matches('&').to_owned())
            .unwrap_or_default()
    }

    // Why: the page as the reader has it, page number included, for the
    // `back=` a judge POST returns to.
    pub(crate) fn current_url(&self) -> String {
        let mut url = self
            .link_prefix(&["ids"])
            .trim_end_matches(['?', '&'])
            .to_owned();
        if self.page() > 0 {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(&format!("page={}", self.page()));
        }
        url
    }

    // Why: the page's filters narrowed by one breakdown bucket or facet, as
    // the export dialog's query; `export_href` is its no-script CSV link. The
    // page's `preset` and ticked `ids` are dropped: the window is stated
    // once, and a bucket is every row in it, not the selection.
    pub(crate) fn export_query(&self, name: &str, value: &str) -> String {
        let prefix = self.link_prefix(&[name, "page", "preset", "ids"]);
        let kept = prefix.split_once('?').map_or("", |(_, q)| q);
        format!(
            "{kept}{name}={}&preset={}",
            urlencode(value),
            export_preset(self)
        )
    }

    pub(crate) fn export_href(&self, dataset: &str, name: &str, value: &str) -> String {
        format!(
            "/admin/export/{dataset}?{}&format=csv",
            self.export_query(name, value)
        )
    }

    // Why: the raw query value behind one filter name, for the chip that
    // removes it and the export that keeps only it.
    pub(crate) fn pair_value(&self, name: &str) -> Option<String> {
        self.pairs()
            .iter()
            .find(|(n, _)| *n == name)
            .and_then(|(_, v)| Self::trimmed(*v))
            .map(str::to_owned)
    }

    pub(crate) fn preserved(&self, drop: &[&str]) -> Vec<(String, String)> {
        self.pairs()
            .iter()
            .filter(|(name, _)| !drop.contains(name))
            .filter_map(|(name, val)| {
                val.map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(|v| ((*name).to_owned(), v.to_owned()))
            })
            .collect()
    }

    pub(crate) fn link_prefix(&self, drop: &[&str]) -> String {
        let qs = self
            .preserved(drop)
            .iter()
            .map(|(name, value)| format!("{name}={}", urlencode(value)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            format!("{BASE_URL}?")
        } else {
            format!("{BASE_URL}?{qs}&")
        }
    }

    // Why: the link a breakdown row or a facet offers — the current filters
    // narrowed by one more `name=value`, back on page one.
    pub(crate) fn narrowed(&self, name: &str, value: &str) -> String {
        format!(
            "{}{name}={}",
            self.link_prefix(&[name, "page"]),
            urlencode(value)
        )
    }
}
