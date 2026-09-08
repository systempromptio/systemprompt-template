//! The breadcrumb trail every detail page renders.
//!
//! One type, one field name, one partial: a page puts a `breadcrumbs` field on
//! its context and `{{> components/breadcrumbs crumbs=breadcrumbs}}` draws it.
//! The last entry is the page itself and carries no `href`, which is how the
//! partial knows to render it as text rather than a link.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BreadcrumbView {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

impl BreadcrumbView {
    pub(crate) fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
        }
    }

    // Why: the page the reader is on — the last crumb, which is never a link.
    pub(crate) fn current(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
        }
    }
}
