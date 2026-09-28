//! `groups.yaml` against the groups, projects and mapping tables: the
//! loader's asymmetry — sets are only upserted, mappings are reconciled —
//! is what the drift rows and the export must say.

use systemprompt_web_admin::repositories::sync::groups_db::{
    parse_groups_doc, render_groups_export,
};
use systemprompt_web_admin::repositories::sync::groups_drift::{
    MappingRow, MemberSetRow, compute_groups_drift, declared_count, declared_hash,
};

const DECLARED: &str = r"
groups:
  - id: europe-devs
    name: Europe developers
    description: Commerce Cloud developers.
    ad_groups: [Systemprompt-Commerce]
  - id: uk
    name: UK
    ad_groups: []
projects:
  - id: commerce
    name: Commerce
    ad_groups: [Systemprompt-Commerce]
";

fn set(kind: &str, id: &str, name: &str, description: Option<&str>, source: &str) -> MemberSetRow {
    MemberSetRow {
        kind: kind.to_owned(),
        id: id.to_owned(),
        name: name.to_owned(),
        description: description.map(str::to_owned),
        source: source.to_owned(),
        is_system: source == "system",
    }
}

fn mapping(kind: &str, ad_group: &str, set_id: &str, source: &str) -> MappingRow {
    MappingRow {
        kind: kind.to_owned(),
        ad_group: ad_group.to_owned(),
        set_id: set_id.to_owned(),
        source: source.to_owned(),
    }
}

fn in_step() -> (Vec<MemberSetRow>, Vec<MappingRow>) {
    (
        vec![
            set(
                "group",
                "europe-devs",
                "Europe developers",
                Some("Commerce Cloud developers."),
                "yaml",
            ),
            set("group", "uk", "UK", None, "yaml"),
            set("group", "unassigned", "Unassigned", None, "system"),
            set("project", "commerce", "Commerce", None, "yaml"),
        ],
        vec![
            mapping("group", "Systemprompt-Commerce", "europe-devs", "yaml"),
            mapping("project", "Systemprompt-Commerce", "commerce", "yaml"),
        ],
    )
}

#[test]
fn a_database_in_step_with_the_file_has_no_drift_and_the_system_group_is_ignored() {
    let doc = parse_groups_doc(DECLARED).expect("parses");
    let (sets, mappings) = in_step();
    let drift = compute_groups_drift(&doc, &sets, &mappings);
    assert!(drift.is_clean(), "{:?}", drift.rows);
    assert_eq!(declared_count(&doc), 5, "three sets and two mappings");
}

#[test]
fn the_declared_hash_moves_with_a_mapping_and_not_with_formatting() {
    let a = declared_hash(&parse_groups_doc(DECLARED).expect("parses"));
    let b = declared_hash(
        &parse_groups_doc(&DECLARED.replace("name: UK", "name:    UK")).expect("parses"),
    );
    let c = declared_hash(
        &parse_groups_doc(&DECLARED.replace("ad_groups: []", "ad_groups: [Systemprompt-UK]"))
            .expect("parses"),
    );
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// Why: a group the console created must be reported and kept, and reach
// the file through export; a mapping code once declared and dropped must be
// deleted by overwrite, while a console-written mapping is kept.
#[test]
fn orphans_follow_the_loader_asymmetry() {
    let doc = parse_groups_doc(DECLARED).expect("parses");
    let (mut sets, mut mappings) = in_step();
    sets.push(set(
        "group",
        "india-devs",
        "India developers",
        None,
        "dashboard",
    ));
    mappings.push(mapping(
        "group",
        "Systemprompt-Core",
        "india-devs",
        "dashboard",
    ));
    mappings.push(mapping("group", "Systemprompt-Legacy", "uk", "yaml"));
    let drift = compute_groups_drift(&doc, &sets, &mappings);

    assert_eq!(drift.orphan_sets, 1);
    assert_eq!(drift.orphan_console_mappings, 1);
    assert_eq!(drift.orphan_yaml_mappings, 1);

    let group = drift
        .rows
        .iter()
        .find(|r| r.entity_id == "india-devs" && r.subject.is_empty())
        .expect("orphan set");
    assert_eq!(group.kind, "orphan");
    assert_eq!(group.origin, "console");
    assert!(!group.overwrite_applies, "sets are never deleted by sync");

    let console_mapping = drift
        .rows
        .iter()
        .find(|r| r.subject == "Systemprompt-Core")
        .expect("console mapping");
    assert!(!console_mapping.overwrite_applies);
    assert_eq!(console_mapping.kind_label, "Written in console");
    assert_eq!(console_mapping.resolve, "Export to keep");

    let code_mapping = drift
        .rows
        .iter()
        .find(|r| r.subject == "Systemprompt-Legacy")
        .expect("mapping code once declared");
    assert!(code_mapping.overwrite_applies);
    assert_eq!(code_mapping.overwrite_effect, "deleted");
    assert_eq!(code_mapping.kind_label, "Removed from code");
    assert_eq!(code_mapping.resolve, "Overwrite deletes");
}

#[test]
fn a_missing_set_and_a_changed_description_are_separate_rows() {
    let doc = parse_groups_doc(DECLARED).expect("parses");
    let (mut sets, mappings) = in_step();
    sets.retain(|s| s.id != "commerce");
    sets[0].description = Some("Renamed in the console.".to_owned());
    sets[0].source = "dashboard".to_owned();
    let drift = compute_groups_drift(&doc, &sets, &mappings);
    assert_eq!(drift.missing_sets, 1);
    assert_eq!(drift.changed_sets, 1);
    let changed = drift
        .rows
        .iter()
        .find(|r| r.kind == "changed")
        .expect("changed row");
    assert_eq!(changed.origin, "console");
    assert!(changed.in_db.contains("Renamed in the console."));
    let missing = drift
        .rows
        .iter()
        .find(|r| r.kind == "missing" && r.subject.is_empty())
        .expect("missing row");
    assert_eq!(missing.entity_id, "commerce");
    assert!(missing.insert_applies);
}

#[test]
fn the_export_carries_console_rows_and_round_trips_through_the_loader() {
    let (mut sets, mut mappings) = in_step();
    sets.push(set(
        "group",
        "india-devs",
        "India developers",
        None,
        "dashboard",
    ));
    mappings.push(mapping(
        "group",
        "Systemprompt-Core",
        "india-devs",
        "dashboard",
    ));
    let yaml = render_groups_export(&sets, &mappings);
    assert!(yaml.starts_with("# The groups and projects"));
    let doc = parse_groups_doc(&yaml).expect("the export parses as groups.yaml");
    assert_eq!(doc.groups.len(), 3, "the system group is not exported");
    let india = doc
        .groups
        .iter()
        .find(|g| g.id == "india-devs")
        .expect("console group exported");
    assert_eq!(india.ad_groups, vec!["Systemprompt-Core".to_owned()]);
    assert!(doc.groups.iter().all(|g| g.id != "unassigned"));
    assert_eq!(doc.projects.len(), 1);
    assert!(
        compute_groups_drift(&doc, &sets, &mappings)
            .rows
            .iter()
            .all(|r| r.kind != "orphan" || r.entity_id == "unassigned"),
        "after export nothing outside the system group is an orphan"
    );
}
