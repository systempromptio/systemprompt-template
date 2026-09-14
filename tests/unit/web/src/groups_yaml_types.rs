//! The bootstrap group/project file: id validity and the duplicate check.
//!
//! Both rules exist so a bad `services/web/config/groups.yaml` fails the boot
//! with a sentence a person can act on, rather than as a Postgres constraint
//! violation or — worse for a duplicate — as the later definition silently
//! winning.

use systemprompt_web_admin::repositories::config::groups_yaml_types::{GroupsDoc, is_valid_id};

fn doc(yaml: &str) -> GroupsDoc {
    serde_yaml::from_str(yaml).expect("document parses")
}

#[test]
fn the_id_rule_matches_the_database_constraint() {
    assert!(is_valid_id("europe-devs"));
    assert!(is_valid_id("europe_devs"));
    assert!(is_valid_id("a"));
    assert!(is_valid_id("9lives"));
    assert!(!is_valid_id(""));
    assert!(!is_valid_id("-leading"));
    assert!(!is_valid_id("_leading"));
    assert!(!is_valid_id("Commerce"), "uppercase is rejected");
    assert!(!is_valid_id("has space"));
    assert!(!is_valid_id("has.dot"));
    assert!(!is_valid_id(&"a".repeat(65)));
    assert!(is_valid_id(&"a".repeat(64)));
}

#[test]
fn a_well_formed_document_validates() {
    let doc = doc(
        "groups:\n  - {id: europe-devs, name: Europe developers, ad_groups: \
         [Systemprompt-Commerce]}\nprojects:\n  - {id: core, name: Core}\n",
    );
    assert_eq!(doc.validate(), Ok(()));
    assert_eq!(doc.groups[0].ad_groups, vec!["Systemprompt-Commerce"]);
    assert!(
        doc.projects[0].ad_groups.is_empty(),
        "mappings are optional"
    );
}

#[test]
fn an_empty_document_is_legitimate() {
    assert_eq!(GroupsDoc::default().validate(), Ok(()));
}

#[test]
fn a_malformed_id_is_named_in_the_error() {
    let err = doc("groups:\n  - {id: Commerce, name: Commerce}\n")
        .validate()
        .expect_err("uppercase id is refused");
    assert!(err.contains("Commerce"), "{err}");
    assert!(err.contains("groups"), "{err}");
}

#[test]
fn a_duplicate_id_is_refused_rather_than_letting_the_later_one_win() {
    let err = doc("projects:\n  - {id: core, name: Core}\n  - {id: core, name: Core platform}\n")
        .validate()
        .expect_err("duplicate id is refused");
    assert!(err.contains("declared twice"), "{err}");
}

#[test]
fn a_group_must_carry_a_name() {
    let err = doc("groups:\n  - {id: core, name: '  '}\n")
        .validate()
        .expect_err("a blank name is refused");
    assert!(err.contains("name"), "{err}");
}

#[test]
fn an_unknown_key_is_refused() {
    assert!(
        serde_yaml::from_str::<GroupsDoc>("groups:\n  - {id: a, name: A, project: commerce}\n")
            .is_err(),
        "a stale key must not be silently ignored"
    );
}
