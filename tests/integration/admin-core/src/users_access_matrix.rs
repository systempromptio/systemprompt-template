//! `repositories::users::access_control::matrix` — the per-user effective
//! access grid.
//!
//! The matrix does not implement its own precedence: it loads the rules and
//! the catalog defaults, gathers the subject's dimension values, and hands all
//! of it to the same `systemprompt_security::authz::resolve` the enforcement
//! webhook calls. These tests therefore assert on the *reported layer* as much
//! as on allow/deny, because the layer is what proves which band decided and
//! that the extension dimensions (department, organization) reached the
//! resolver at all.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::users::access_control::{
    MatrixRow, SectionInput, resolve_user_matrix,
};

use crate::fixtures::{
    AclRuleSpec, insert_acl_rule, insert_user, insert_user_full, set_department, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

// One section holding a single skill, which is what every test below grades.
pub fn one_skill(entity_id: &str) -> Vec<SectionInput> {
    vec![(
        "skill".to_owned(),
        "Skills".to_owned(),
        vec![(entity_id.to_owned(), "A Skill".to_owned(), None)],
    )]
}

async fn set_default_included(pool: &PgPool, entity_type: &str, entity_id: &str, included: bool) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ($1, $2, $3, 'fixture')
         ON CONFLICT (entity_type, entity_id) DO UPDATE
            SET default_included = EXCLUDED.default_included",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(included)
    .execute(pool)
    .await
    .expect("set entity default");
}

pub async fn grade(pool: &PgPool, user: &UserId, entity_id: &str) -> MatrixRow {
    let matrix = resolve_user_matrix(pool, user, one_skill(entity_id))
        .await
        .expect("resolve matrix")
        .expect("user found");
    let mut sections = matrix.sections;
    let section = sections.pop().expect("one section");
    section.rows.into_iter().next().expect("one row")
}

#[tokio::test]
async fn resolve_user_matrix_returns_none_for_an_unknown_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let matrix = resolve_user_matrix(&db.pool, &UserId::new(unique("nobody")), Vec::new())
        .await
        .expect("resolve matrix");

    assert!(matrix.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_reports_the_users_identity_and_department() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("matrix");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    set_department(&db.pool, &user, "Platform").await;

    let matrix = resolve_user_matrix(&db.pool, &user, Vec::new())
        .await
        .expect("resolve matrix")
        .expect("user found");

    assert_eq!(matrix.user.id, user.as_str());
    assert_eq!(matrix.user.email.as_deref(), Some(email.as_str()));
    assert_eq!(matrix.user.department.as_deref(), Some("Platform"));
    assert_eq!(matrix.user.roles, ["user"]);
    assert!(
        matrix.sections.is_empty(),
        "no sections in means no sections out"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_defaults_a_user_with_no_profile_row_to_the_default_department() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nodept")).await;

    let matrix = resolve_user_matrix(&db.pool, &user, Vec::new())
        .await
        .expect("resolve matrix")
        .expect("user found");

    assert_eq!(matrix.user.department.as_deref(), Some("Default"));
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_denies_an_entity_that_is_not_default_included() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("denydefault")).await;
    let skill = unique("skill");
    set_default_included(&db.pool, "skill", &skill, false).await;

    let row = grade(&db.pool, &user, &skill).await;

    assert_eq!(row.effective, "deny");
    assert_eq!(row.source.layer, "default");
    assert!(!row.default_included);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_allows_a_default_included_entity_with_no_rules() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("allowdefault")).await;
    let skill = unique("skill");
    set_default_included(&db.pool, "skill", &skill, true).await;

    let row = grade(&db.pool, &user, &skill).await;

    assert_eq!(row.effective, "allow");
    assert_eq!(row.source.layer, "default");
    assert!(row.default_included);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_treats_an_entity_with_no_catalog_row_as_not_included() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nocatalog")).await;

    let row = grade(&db.pool, &user, &unique("skill")).await;

    assert_eq!(row.effective, "deny");
    assert!(!row.default_included);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_honours_a_grant_naming_the_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("useralow")).await;
    let skill = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &skill, "user", user.as_str()),
    )
    .await;

    let row = grade(&db.pool, &user, &skill).await;

    assert_eq!(row.effective, "allow");
    assert_eq!(row.source.layer, "user");
    assert!(row.source.detail.contains(user.as_str()));
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_lets_a_user_deny_beat_a_default_include() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("userdeny")).await;
    let skill = unique("skill");
    set_default_included(&db.pool, "skill", &skill, true).await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::deny("skill", &skill, "user", user.as_str()),
    )
    .await;

    let row = grade(&db.pool, &user, &skill).await;

    assert_eq!(row.effective, "deny");
    assert_eq!(row.source.layer, "user");
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_honours_a_grant_naming_one_of_the_users_roles() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("rolealow"),
        Some("Role Holder"),
        &["user".to_owned(), "auditor".to_owned()],
    )
    .await;
    let skill = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &skill, "role", "auditor"),
    )
    .await;

    let row = grade(&db.pool, &user, &skill).await;

    assert_eq!(row.effective, "allow");
    assert_eq!(row.source.layer, "role");
    assert!(row.source.detail.contains("auditor"));
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_user_matrix_ignores_a_grant_for_a_role_the_user_does_not_hold() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("otherrole")).await;
    let skill = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &skill, "role", "auditor"),
    )
    .await;

    let row = grade(&db.pool, &user, &skill).await;

    assert_eq!(row.effective, "deny");
    db.cleanup().await;
}
