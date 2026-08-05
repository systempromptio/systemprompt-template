//! `repositories::users::access_control::matrix` — the multi-section shape of
//! the grid and how it grades rows core's entity vocabulary does not know.
//!
//! The `department` dimension is deliberately NOT covered here. It reaches the
//! resolver through `authz::subject_attributes_for`, which memoises its
//! provider set — and the pool those providers hold — in a process-global
//! `OnceLock`. This suite gives every test its own throwaway database in one
//! process, so whichever test resolves first pins the registry to *its*
//! database and every later department lookup reads the wrong one. Core's
//! `user` and `role` dimensions take the pool per call and are unaffected, so
//! the tests below exercise those plus the multi-section shape of the grid and
//! an entity type core's `EntityKind` does not know.

use systemprompt_web_admin::repositories::users::access_control::{
    filter_catalog_for_user, resolve_user_matrix,
};

use crate::fixtures::{AclRuleSpec, insert_acl_rule, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;
use crate::users_access_matrix::one_skill;

#[tokio::test]
async fn resolve_user_matrix_falls_back_to_the_default_for_an_unrecognised_entity_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("badkind")).await;
    let sections = vec![(
        "warp_drive".to_owned(),
        "Warp Drives".to_owned(),
        vec![(unique("entity"), "A Drive".to_owned(), None)],
    )];

    let matrix = resolve_user_matrix(&db.pool, &user, sections)
        .await
        .expect("resolve matrix")
        .expect("user found");

    let row = &matrix.sections[0].rows[0];
    assert_eq!(row.effective, "deny");
    assert_eq!(row.source.layer, "default");
    assert!(row.source.detail.contains("unknown entity type"));
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_grades_every_row_of_every_section() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sections")).await;
    let allowed = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &allowed, "user", user.as_str()),
    )
    .await;
    let server = unique("server");
    let sections = vec![
        (
            "skill".to_owned(),
            "Skills".to_owned(),
            vec![
                (
                    allowed.clone(),
                    "Allowed".to_owned(),
                    Some("desc".to_owned()),
                ),
                (unique("skill"), "Blocked".to_owned(), None),
            ],
        ),
        (
            "mcp_server".to_owned(),
            "MCP Servers".to_owned(),
            vec![(server, "A Server".to_owned(), None)],
        ),
    ];

    let matrix = resolve_user_matrix(&db.pool, &user, sections)
        .await
        .expect("resolve matrix")
        .expect("user found");

    assert_eq!(matrix.sections.len(), 2);
    assert_eq!(matrix.sections[0].label, "Skills");
    assert_eq!(matrix.sections[0].rows.len(), 2);
    assert_eq!(matrix.sections[0].rows[0].effective, "allow");
    assert_eq!(
        matrix.sections[0].rows[0].description.as_deref(),
        Some("desc")
    );
    assert_eq!(matrix.sections[0].rows[1].effective, "deny");
    assert_eq!(matrix.sections[1].entity_type, "mcp_server");
    assert_eq!(matrix.sections[1].rows[0].effective, "deny");
    db.cleanup().await;
}

#[tokio::test]
async fn filter_catalog_for_user_is_the_same_grading_as_resolve_user_matrix() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("filter")).await;
    let skill = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &skill, "user", user.as_str()),
    )
    .await;

    let matrix = filter_catalog_for_user(&db.pool, &user, one_skill(&skill))
        .await
        .expect("filter catalog")
        .expect("user found");

    assert_eq!(matrix.sections[0].rows[0].effective, "allow");
    assert_eq!(matrix.sections[0].rows[0].source.layer, "user");
    db.cleanup().await;
}
