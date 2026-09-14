//! The skills page groups entries into a curated category order rather than an
//! alphabetical one, so Consultancy leads and unrecognised categories sort
//! last. `display_category` overrides `category`, and an entry with neither
//! still has to land somewhere — "General" — instead of vanishing from the
//! page.

use serde_json::Value;
use systemprompt_web_site::skills_page::SkillEntry;
use systemprompt_web_site::skills_page::prerenderer::{category_rank, group_by_category};

fn entry(id: &str, category: Option<&str>, display_category: Option<&str>) -> SkillEntry {
    SkillEntry {
        id: id.to_owned(),
        name: id.to_owned(),
        description: "A skill".to_owned(),
        enabled: true,
        tags: vec![],
        category: category.map(str::to_owned),
        display_category: display_category.map(str::to_owned),
        href: None,
    }
}

fn category_names(grouped: &[Value]) -> Vec<String> {
    grouped
        .iter()
        .map(|g| g["name"].as_str().expect("category name").to_owned())
        .collect()
}

#[test]
fn the_curated_categories_rank_ahead_of_everything_else() {
    assert_eq!(category_rank("Consultancy Workflows"), 0);
    assert!(category_rank("Consultancy Workflows") < category_rank("Brand & Workspace"));
    assert!(category_rank("Platform & Operations") < category_rank("General"));
    assert_eq!(category_rank("General"), category_rank("Anything Unlisted"));
}

#[test]
fn categories_render_in_curated_order_with_unlisted_ones_last() {
    let grouped = group_by_category(&[
        entry("z", Some("Zebra Tools"), None),
        entry("p", Some("Platform & Operations"), None),
        entry("s", Some("Consultancy Workflows"), None),
    ]);

    assert_eq!(
        category_names(&grouped),
        vec![
            "Consultancy Workflows",
            "Platform & Operations",
            "Zebra Tools"
        ]
    );
}

#[test]
fn unlisted_categories_tie_break_alphabetically() {
    let grouped = group_by_category(&[
        entry("b", Some("Beta"), None),
        entry("a", Some("Alpha"), None),
        entry("c", Some("Charlie"), None),
    ]);

    assert_eq!(category_names(&grouped), vec!["Alpha", "Beta", "Charlie"]);
}

#[test]
fn display_category_wins_and_a_categoryless_skill_lands_in_general() {
    let grouped = group_by_category(&[
        entry(
            "override",
            Some("Consultancy Workflows"),
            Some("Brand & Workspace"),
        ),
        entry("bare", None, None),
    ]);

    assert_eq!(
        category_names(&grouped),
        vec!["Brand & Workspace", "General"]
    );
    assert_eq!(grouped[0]["skills"].as_array().expect("skills").len(), 1);
    assert_eq!(grouped[1]["skills"][0]["id"], "bare");
}
