//! URL and time-window builders for the Evals page.
//!
//! The tab bar, the time-range picker and the post-action redirect all rebuild
//! the same query string, so the reader keeps their window and their tab across
//! every navigation the page offers.

use urlencoding::encode as urlencode;

use crate::util::time_range::TimeRange;

use super::context::{EvalTabLinkView, EvalTimeRangeView, EvalsTab};
use super::{BASE_URL, EvalsQuery};


// Why: The tab bar. Every link carries the current window so switching tabs
// never silently resets the range the reader chose, but drops the Judge tab's
// verdict/model filters, which mean nothing anywhere else.
pub(super) fn tab_links(
    active: EvalsTab,
    range: &TimeRange,
    query: &EvalsQuery,
) -> Vec<EvalTabLinkView> {
    const TABS: [(EvalsTab, &str); 5] = [
        (EvalsTab::Overview, "Overview"),
        (EvalsTab::Traffic, "Traffic"),
        (EvalsTab::Judge, "Scored answers"),
        (EvalsTab::HeadToHead, "Head-to-head"),
        (EvalsTab::GoldenSet, "Golden set"),
    ];

    TABS.iter()
        .map(|&(tab, label)| EvalTabLinkView {
            slug: tab.as_str(),
            label,
            href: format!(
                "{BASE_URL}?tab={}{}",
                tab.as_str(),
                range_query(range, query)
            ),
            is_active: tab == active,
        })
        .collect()
}

fn range_query(range: &TimeRange, query: &EvalsQuery) -> String {
    match query.preset.as_deref() {
        Some(preset) if preset != "custom" => format!("&preset={}", urlencode(preset)),
        _ => format!(
            "&preset=custom&from={}&to={}",
            urlencode(&range.from.to_rfc3339()),
            urlencode(&range.to.to_rfc3339()),
        ),
    }
}

pub(super) fn time_range_context(
    query: &EvalsQuery,
    range: &TimeRange,
    auto_widened: Option<&'static str>,
    tab: EvalsTab,
) -> EvalTimeRangeView {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_owned()
        } else {
            auto_widened.unwrap_or("24h").to_owned()
        }
    });
    EvalTimeRangeView {
        preset,
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        // Why: Preserves the tab across the time-range picker's own links, which
        // would otherwise drop the reader back on Overview.
        query: format!("&tab={}", tab.as_str()),
        auto_widened,
    }
}

// Why: Where a POST action sends the browser back to. `tab` is the tab the form
// was fired from, so a run's notice appears above the table that run filled.
pub(super) fn redirect_url(range: &TimeRange, tab: &str, notice: &str, is_error: bool) -> String {
    format!(
        "{BASE_URL}?tab={}&preset=custom&from={}&to={}&notice={}&notice_error={}",
        urlencode(tab),
        urlencode(&range.from.to_rfc3339()),
        urlencode(&range.to.to_rfc3339()),
        urlencode(notice),
        if is_error { "1" } else { "0" },
    )
}
