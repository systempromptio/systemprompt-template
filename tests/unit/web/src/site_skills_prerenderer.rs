//! The skills-page prerenderer's render spec.
//!
//! `group_by_category` and `category_rank` are asserted on their own in
//! `site_skills_grouping`; what this file pins is the spec the publish
//! pipeline actually consumes — the template name, the output path, and the
//! shape of the data handed to that template. The `count` is the flat skill
//! total rather than the number of categories, which is the mistake the
//! grouped structure invites.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::extension::prelude::{PagePrepareContext, PagePrerenderer};
use systemprompt::models::services::WebConfig;
use systemprompt_web_site::skills_page::{SkillsPageConfig, SkillsPagePrerenderer};

const WEB_CONFIG_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../services/web/config.yaml"
);

fn web_config() -> WebConfig {
    let raw = std::fs::read_to_string(WEB_CONFIG_PATH).expect("the deployment ships a web config");
    serde_yaml::from_str(&raw).expect("services/web/config.yaml deserialises into a WebConfig")
}

fn config(yaml: &str) -> Arc<SkillsPageConfig> {
    Arc::new(serde_yaml::from_str(yaml).expect("the skills fixture matches SkillsPageConfig"))
}

// Build a skills fixture from `(id, category)` pairs, so the YAML stays
// readable however rustfmt wraps the call.
fn skills(entries: &[(&str, Option<&str>)]) -> Arc<SkillsPageConfig> {
    let mut yaml = String::from("skills:\n");
    for (id, category) in entries {
        yaml.push_str(&format!(
            "  - id: {id}\n    name: {id}\n    description: A skill.\n"
        ));
        if let Some(category) = category {
            yaml.push_str(&format!("    category: \"{category}\"\n"));
        }
    }
    config(&yaml)
}

fn prepare(config: Arc<SkillsPageConfig>) -> systemprompt::extension::prelude::PageRenderSpec {
    let web = web_config();
    let erased = ();
    let dist = std::path::Path::new("/nonexistent-dist");
    let ctx = PagePrepareContext::new(&web, &erased, &erased, dist);

    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(SkillsPagePrerenderer::new(config).prepare(&ctx))
        .expect("the prerenderer succeeds")
        .expect("the skills page always has a render spec")
}

#[test]
fn the_prerenderer_claims_the_skills_page_at_the_section_priority() {
    let prerenderer = SkillsPagePrerenderer::new(config("skills: []\n"));

    assert_eq!(prerenderer.page_type(), "skills-page");
    assert_eq!(
        prerenderer.priority(),
        50,
        "section pages are prepared after the homepage"
    );
}

#[test]
fn the_spec_names_the_skills_template_and_its_output_path() {
    let spec = prepare(config("skills: []\n"));

    assert_eq!(spec.template_name, "skills");
    assert_eq!(spec.output_path, PathBuf::from("skills/index.html"));
}

#[test]
fn an_empty_skill_list_still_renders_a_page() {
    let spec = prepare(config("skills: []\n"));

    assert_eq!(spec.base_data["skills"]["count"], 0);
    assert_eq!(
        spec.base_data["skills"]["categories"]
            .as_array()
            .expect("categories is an array")
            .len(),
        0
    );
    assert!(
        spec.base_data["site"].is_object(),
        "the web config is nested under site for the template"
    );
}

#[test]
fn the_count_is_the_flat_skill_total_not_the_category_total() {
    let spec = prepare(skills(&[
        ("a", Some("Salesforce")),
        ("b", Some("Salesforce")),
        ("c", Some("Governance & Analytics")),
    ]));

    assert_eq!(spec.base_data["skills"]["count"], 3);
    assert_eq!(
        spec.base_data["skills"]["categories"]
            .as_array()
            .expect("categories is an array")
            .len(),
        2,
        "three skills group into two categories"
    );
}

#[test]
fn items_carries_every_skill_in_its_declared_order() {
    let spec = prepare(skills(&[("zebra", None), ("alpha", None)]));

    let items = spec.base_data["skills"]["items"]
        .as_array()
        .expect("items is an array");
    assert_eq!(items[0]["id"], "zebra");
    assert_eq!(
        items[1]["id"], "alpha",
        "items is the unsorted source list; only categories are ordered"
    );
}

#[test]
fn the_curated_category_order_survives_into_the_render_data() {
    let spec = prepare(skills(&[
        ("ops", Some("Platform & Operations")),
        ("sf", Some("Salesforce")),
    ]));

    let names: Vec<&str> = spec.base_data["skills"]["categories"]
        .as_array()
        .expect("categories is an array")
        .iter()
        .map(|c| c["name"].as_str().expect("each category is named"))
        .collect();

    assert_eq!(
        names,
        vec!["Salesforce", "Platform & Operations"],
        "the curated order wins over the alphabetical grouping order"
    );
}
