//! Structural gates on the analytics dashboard's templates.
//!
//! The page's honesty rules are expressed in markup, so they are checked in
//! markup: the customer cost view must carry no supplier figure, every table
//! must name itself for a screen reader, and every numeric cell must be
//! right-aligned and tabular. A rendering test could not see any of these —
//! a leaked cost column renders perfectly well.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use std::path::PathBuf;

use crate::support::repo_root;

const TABS: [&str; 5] = ["overview", "models", "tools", "sessions", "cost"];

fn partial(name: &str) -> String {
    let path: PathBuf =
        repo_root().join(format!("storage/files/admin/partials/analytics/{name}.hbs"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn page_template() -> String {
    let path = repo_root().join("storage/files/admin/templates/analytics-dashboard.hbs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn every_tab_has_a_partial_the_page_includes() {
    let page = page_template();
    for tab in TABS {
        assert!(
            page.contains(&format!("{{{{> analytics/{tab}}}}}")),
            "the page template never includes the {tab} tab's partial"
        );
        assert!(!partial(tab).is_empty(), "the {tab} partial is empty");
    }
}

// The customer view is sent outside the platform team. The repository behind
// it selects no cost column, and the markup must not print one either — the
// two guards are independent on purpose.
#[test]
fn the_customer_cost_view_prints_no_supplier_figure() {
    let cost = partial("cost");
    let start = cost
        .find("Consumption by container")
        .expect("the cost partial no longer has a container section");
    let customer_half = &cost[start..];
    assert!(
        !customer_half.contains("cost_display"),
        "the container table prints a cost cell; the customer export must carry none"
    );
}

#[test]
fn every_table_names_itself_for_a_screen_reader() {
    for tab in TABS {
        let body = partial(tab);
        for (n, _) in body.match_indices("{{#> components/table") {
            let head = &body[n..(n + 200).min(body.len())];
            assert!(
                head.contains("caption="),
                "a table on the {tab} tab has no caption"
            );
        }
    }
}

// A number the eye has to scan a column of must be right-aligned and tabular,
// or the column cannot be compared at a glance — which is the only reason to
// put numbers in a column.
#[test]
fn every_numeric_cell_is_right_aligned_and_tabular() {
    for tab in TABS {
        for line in partial(tab).lines() {
            if !line.contains("<td class=") || !line.contains("sp-table__cell--num") {
                continue;
            }
            assert!(
                line.contains("sp-u-num") || line.contains("sp-u-mono"),
                "a numeric cell on the {tab} tab is neither sp-u-num nor sp-u-mono: {}",
                line.trim()
            );
        }
    }
}

// Every tab must answer with something when its query returns nothing. A tab
// that renders a bare heading over no table reads as a broken page rather than
// as an empty window.
#[test]
fn every_tab_answers_an_empty_result() {
    for tab in TABS {
        assert!(
            partial(tab).contains("components/empty-state"),
            "the {tab} tab has no empty state"
        );
    }
}

// The page CSS budget: one block, one file, sixty lines. The shared components
// carry everything else, and a page stylesheet that grows past this is a
// component that was never extracted.
#[test]
fn the_page_stylesheet_stays_within_its_budget() {
    let path = repo_root().join("storage/files/css/admin/20-page-analytics.css");
    let css = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    assert!(
        css.lines().count() <= 60,
        "20-page-analytics.css is {} lines; the budget is 60",
        css.lines().count()
    );
    for line in css.lines() {
        assert!(
            !line.contains('#') && !line.contains("rgb(") && !line.contains("oklch("),
            "a literal colour in the page stylesheet: {}",
            line.trim()
        );
    }
}
