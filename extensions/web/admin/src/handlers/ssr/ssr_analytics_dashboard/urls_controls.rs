//! The links that change how the page reads its own numbers, rather than
//! which rows it shows: the attribution toggle, the Cost tab's audience and
//! container axis, the CSV export, and the per-row drill into the request log.
//!
//! Split from `urls.rs` at the 300-line ceiling. Every builder here rebuilds
//! the current query string minus the parameter it is changing, so following
//! one never silently drops the scope or the window.

use urlencoding::encode as urlencode;

use crate::repositories::analytics::site::cost::ContainerAxis;

use super::context::AttributionLink;
use super::urls::{preserved_query_string, with_qs};
use super::{AnalyticsDashboardQuery, BASE_URL};

// Why: the request log is the one listing every drill lands on, and it reads
// `model`, `skill`, `tool` and `provider` beside the shared `group`,
// `project`, `user_id` and `preset` scope parameters. Carrying the scope is
// what stops a drill silently widening what the reader was looking at.
pub(super) fn drill_url(query: &AnalyticsDashboardQuery, dimension: &str, value: &str) -> String {
    let mut parts = vec![format!("{dimension}={}", urlencode(value))];
    let scope: [(&str, Option<&str>); 4] = [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        (
            "user_id",
            query
                .user_id
                .as_ref()
                .map(systemprompt::identifiers::UserId::as_str),
        ),
        ("preset", query.preset.as_deref()),
    ];
    for (name, v) in scope {
        if let Some(v) = v.filter(|s| !s.is_empty()) {
            parts.push(format!("{name}={}", urlencode(v)));
        }
    }
    format!("/admin/requests?{}", parts.join("&"))
}

// Why: attribution is a claim about the numbers, not a filter on them, so it
// is offered as two named readings rather than a checkbox. Exclusive is the
// default and the only one whose rows sum back to the instance total.
pub(super) fn attribution_links(
    query: &AnalyticsDashboardQuery,
    member: bool,
) -> Vec<AttributionLink> {
    let qs = preserved_query_string(query, &["attr", "page"]);
    vec![
        AttributionLink {
            label: "Exclusive",
            href: with_qs(format!("{BASE_URL}?attr=exclusive"), &qs),
            is_active: !member,
            hint: "Each person counts once, in their primary group and project, so container \
                   totals partition the instance.",
        },
        AttributionLink {
            label: "Member view",
            href: with_qs(format!("{BASE_URL}?attr=member"), &qs),
            is_active: member,
            hint: "Each person counts in every container they belong to, so the rows overlap \
                   and deliberately do not sum to the instance total.",
        },
    ]
}

pub(super) fn audience_links(
    query: &AnalyticsDashboardQuery,
    internal: bool,
) -> Vec<AttributionLink> {
    let qs = preserved_query_string(query, &["audience", "page"]);
    vec![
        AttributionLink {
            label: "Internal",
            href: with_qs(format!("{BASE_URL}?tab=cost&audience=internal"), &qs),
            is_active: internal,
            hint: "What inference cost us at the providers.",
        },
        AttributionLink {
            label: "Customer",
            href: with_qs(format!("{BASE_URL}?tab=cost&audience=customer"), &qs),
            is_active: !internal,
            hint: "What each container consumed. Carries no supplier cost, so it is safe to \
                   send outside the platform team.",
        },
    ]
}

pub(super) fn axis_links(
    query: &AnalyticsDashboardQuery,
    axis: ContainerAxis,
) -> Vec<AttributionLink> {
    let qs = preserved_query_string(query, &["axis", "page"]);
    [
        (ContainerAxis::Group, "By group"),
        (ContainerAxis::Project, "By project"),
    ]
    .into_iter()
    .map(|(value, label)| AttributionLink {
        label,
        href: with_qs(format!("{BASE_URL}?tab=cost&axis={}", value.as_str()), &qs),
        is_active: value == axis,
        hint: "Which column of the daily rollups the consumption is grouped on.",
    })
    .collect()
}

pub(super) fn cost_csv_url(query: &AnalyticsDashboardQuery, internal: bool) -> String {
    let qs = preserved_query_string(query, &["tab", "audience", "page"]);
    let audience = if internal { "internal" } else { "customer" };
    with_qs(
        format!("/admin/analytics/cost.csv?audience={audience}"),
        &qs,
    )
}
