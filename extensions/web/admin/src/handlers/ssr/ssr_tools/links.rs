//! The navigation the Tools and Artifacts pages hang off their query: the
//! window tabs, the breakdown tabs, the pager and the sortable headers.

use super::query::{Lens, ToolsQuery};
use crate::handlers::ssr::list_view::{PageWindow, Pagination, paginate_prefixed};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::tools::{ToolBreakdownBy, ToolSort};
use crate::util::time_range::TimeRange;

impl ToolsQuery {
    pub(crate) fn range_links(&self, lens: Lens, range: TimeRange) -> Vec<TabLinkView> {
        let prefix = self.link_prefix(lens, &["preset", "from", "to", "page"]);
        let active = self.preset_label(range);
        [
            ("1h", "1h"),
            ("24h", "24h"),
            ("7d", "7 days"),
            ("30d", "30 days"),
        ]
        .iter()
        .map(|(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("{prefix}preset={slug}"),
            is_active: *slug == active,
            count: None,
        })
        .collect()
    }

    pub(crate) fn breakdown_tabs(&self, lens: Lens) -> Vec<TabLinkView> {
        let prefix = self.link_prefix(lens, &["by", "page"]);
        let active = self.breakdown(lens);
        ToolBreakdownBy::ALL
            .iter()
            .map(|by| TabLinkView {
                slug: by.as_str(),
                label: by.label(),
                href: format!("{prefix}by={}", by.as_str()),
                is_active: *by == active,
                count: None,
            })
            .collect()
    }

    pub(crate) fn pagination(&self, lens: Lens, window: PageWindow) -> Pagination {
        paginate_prefixed(window, &self.link_prefix(lens, &["page"]))
    }

    pub(crate) fn sort_header(
        &self,
        lens: Lens,
        col: ToolSort,
        (label, class, hint): (&'static str, &'static str, &'static str),
    ) -> SortHeaderView {
        let prefix = self.link_prefix(lens, &["sort", "dir", "page"]);
        let active = col == self.sort();
        let is_desc = self.descending();
        let next_dir = SortHeaderView::next_dir(active, is_desc);
        SortHeaderView::new(
            (label, class, hint),
            format!("{prefix}sort={}&dir={next_dir}", col.as_str()),
            active,
            is_desc,
        )
    }
}
