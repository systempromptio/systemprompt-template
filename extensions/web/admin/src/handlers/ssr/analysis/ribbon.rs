//! The filter ribbon every list page in this section shares: one pill per
//! dimension holding the choices the filtered set actually contains, a
//! free-text box, and a chip per active filter with the link that removes
//! it and the link that downloads what it selects. Pages build a
//! [`RibbonView`] from the facets their repository already returns and hand
//! it to `components/filter-ribbon`; nothing here knows a page's query type.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct RibbonOptionView {
    pub id: String,
    pub label: String,
    pub count: i64,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RibbonGroupView {
    pub param: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub options: Vec<RibbonOptionView>,
    pub multi: bool,
    pub active_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct RibbonChipView {
    pub group_label: &'static str,
    pub label: String,
    pub remove_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_href: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RibbonSearchView {
    pub name: &'static str,
    pub value: String,
    pub placeholder: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct RibbonPreservedView {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RibbonView {
    pub base_url: String,
    pub preserved: Vec<RibbonPreservedView>,
    pub groups: Vec<RibbonGroupView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<RibbonSearchView>,
    pub chips: Vec<RibbonChipView>,
    pub clear_url: String,
}

impl RibbonGroupView {
    // Why: one group from a facet list — `(value, label, count)` triples —
    // with the current choice marked; `selected` is the raw query value so a
    // choice outside the facet list still shows as active on its chip.
    pub(crate) fn single<'a, I>(
        param: &'static str,
        label: &'static str,
        icon: &'static str,
        selected: Option<&str>,
        items: I,
    ) -> Self
    where
        I: IntoIterator<Item = (&'a str, String, i64)>,
    {
        let options: Vec<RibbonOptionView> = items
            .into_iter()
            .map(|(id, label, count)| RibbonOptionView {
                selected: selected == Some(id),
                id: id.to_owned(),
                label,
                count,
            })
            .collect();
        Self {
            param,
            label,
            icon,
            active_count: usize::from(selected.is_some_and(|s| !s.is_empty())),
            options,
            multi: false,
        }
    }

    pub(crate) fn fixed(
        param: &'static str,
        label: &'static str,
        icon: &'static str,
        selected: Option<&str>,
        items: &[(&str, &str)],
    ) -> Self {
        Self::single(
            param,
            label,
            icon,
            selected,
            items.iter().map(|(v, l)| (*v, (*l).to_owned(), 0)),
        )
    }
}

impl RibbonView {
    pub(crate) fn new(base_url: impl Into<String>, clear_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            preserved: Vec::new(),
            groups: Vec::new(),
            search: None,
            chips: Vec::new(),
            clear_url: clear_url.into(),
        }
    }

    pub(crate) fn preserve(mut self, pairs: &[(String, String)]) -> Self {
        self.preserved = pairs
            .iter()
            .map(|(name, value)| RibbonPreservedView {
                name: name.clone(),
                value: value.clone(),
            })
            .collect();
        self
    }

    pub(crate) fn group(mut self, group: RibbonGroupView) -> Self {
        self.groups.push(group);
        self
    }

    pub(crate) fn search(
        mut self,
        name: &'static str,
        value: Option<&str>,
        placeholder: &'static str,
    ) -> Self {
        self.search = Some(RibbonSearchView {
            name,
            value: value.unwrap_or_default().to_owned(),
            placeholder,
        });
        self
    }

    // Why: a chip per active filter, in the order the groups are drawn, each
    // with the page minus that one filter and, when the page offers one, the
    // export of exactly the rows it selects.
    pub(crate) fn chip(
        mut self,
        group_label: &'static str,
        label: impl Into<String>,
        remove_url: String,
        export_href: Option<String>,
    ) -> Self {
        self.chips.push(RibbonChipView {
            group_label,
            label: label.into(),
            remove_url,
            export_href,
        });
        self
    }
}
