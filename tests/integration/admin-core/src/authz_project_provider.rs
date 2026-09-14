//! The `project` subject dimension: the provider that reads a user's project
//! memberships and the band it occupies.
//!
//! Unlike `group` there is no derived fallback. Work attribution is optional,
//! so a user on no project holds no value here and matches no project rule —
//! which is what makes a project grant the narrowest statement an operator can
//! write.

use std::sync::Arc;

use systemprompt_security::authz::SubjectAttributeProvider;
use systemprompt_web_admin::authz::project::{
    ProjectAttributeProvider, invalidate, project_rule_type,
};

use crate::fixtures::{
    insert_project, insert_project_member, insert_user, unclaimed_email, unique, unique_project,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn the_provider_resolves_a_users_projects() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("pdim")).await;
    let project = unique_project("proj");
    insert_project(&db.pool, &project, "Storefront").await;
    insert_project_member(&db.pool, &project, &user, "manual").await;
    let provider = ProjectAttributeProvider::new(Arc::new((*db.pool).clone()));
    invalidate(&user).await;

    assert_eq!(
        provider
            .values_for(&user)
            .await
            .expect("project membership lookup"),
        vec![project.as_str().to_owned()]
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_user_on_no_project_holds_no_value_at_all() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("pnone")).await;
    let provider = ProjectAttributeProvider::new(Arc::new((*db.pool).clone()));
    invalidate(&user).await;

    assert!(
        provider
            .values_for(&user)
            .await
            .expect("project membership lookup")
            .is_empty(),
        "no derived fallback: they match no project rule and the entity default closes"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_user_may_hold_several_projects() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("pmany")).await;
    let mut expected = vec![unique("proj"), unique("proj")];
    expected.sort();
    for project in &expected {
        insert_project(&db.pool, project, "Work").await;
        insert_project_member(&db.pool, project, &user, "manual").await;
    }
    let provider = ProjectAttributeProvider::new(Arc::new((*db.pool).clone()));
    invalidate(&user).await;

    assert_eq!(
        provider
            .values_for(&user)
            .await
            .expect("project membership lookup"),
        expected
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_project_band_is_narrower_than_the_group_band() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let provider = ProjectAttributeProvider::new(Arc::new((*db.pool).clone()));

    let dimension = provider.dimension();

    assert_eq!(dimension.rule_type, project_rule_type());
    assert_eq!(dimension.rule_type.as_str(), "project");
    assert_eq!(dimension.precedence, 140);
    db.cleanup().await;
}
