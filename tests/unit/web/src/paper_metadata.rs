//! Front-matter for long-form paper content.
//!
//! Every field but `id` and `title` is optional, and `image_position` defaults
//! to `right` rather than to the empty string — a paper section that omits it
//! must still lay out, not render against an unset CSS class.

use systemprompt_web_shared::models::{PaperMetadata, PaperSection};

#[test]
fn a_section_omitting_its_image_position_defaults_to_right() {
    let section: PaperSection =
        serde_yaml::from_str("id: intro\ntitle: Introduction\n").expect("a minimal section parses");

    assert_eq!(section.id, "intro");
    assert_eq!(section.title, "Introduction");
    assert_eq!(section.image_position, "right");
    assert!(section.file.is_none());
    assert!(section.image.is_none());
    assert!(section.image_alt.is_none());
}

#[test]
fn a_declared_image_position_is_kept() {
    let section: PaperSection = serde_yaml::from_str(
        "id: governance\ntitle: Governance\nfile: governance.md\nimage: /files/g.png\nimage_alt: A \
         pipeline\nimage_position: left\n",
    )
    .expect("a fully specified section parses");

    assert_eq!(section.image_position, "left");
    assert_eq!(section.file.as_deref(), Some("governance.md"));
    assert_eq!(section.image_alt.as_deref(), Some("A pipeline"));
}

#[test]
fn paper_metadata_defaults_to_no_hero_no_toc_and_no_sections() {
    let metadata = PaperMetadata::default();

    assert!(metadata.hero_image.is_none());
    assert!(metadata.hero_alt.is_none());
    assert!(metadata.sections.is_empty());
    assert!(!metadata.toc, "the table of contents is opt-in");
    assert!(metadata.chapters_path.is_none());
}

#[test]
fn empty_front_matter_deserialises_into_those_same_defaults() {
    let metadata: PaperMetadata = serde_yaml::from_str("{}\n").expect("every field has a default");

    assert!(metadata.sections.is_empty());
    assert!(!metadata.toc);
}

#[test]
fn sections_round_trip_through_serialisation() {
    let metadata: PaperMetadata = serde_yaml::from_str(
        "hero_image: /files/hero.png\nhero_alt: Hero\ntoc: true\nchapters_path: \
         chapters/\nsections:\n  - id: one\n    title: One\n  - id: two\n    title: Two\n    \
         image_position: left\n",
    )
    .expect("the fixture parses");

    assert_eq!(metadata.sections.len(), 2);
    assert!(metadata.toc);
    assert_eq!(metadata.chapters_path.as_deref(), Some("chapters/"));

    let encoded = serde_json::to_value(&metadata).expect("metadata serialises");
    assert_eq!(encoded["sections"][0]["image_position"], "right");
    assert_eq!(encoded["sections"][1]["image_position"], "left");
    assert_eq!(encoded["hero_alt"], "Hero");
}
