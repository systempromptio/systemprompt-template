//! The boot-time platform-admin reconciler — the production path that mints
//! the first platform admin.
//!
//! "Platform admin" is the `admin` role plus membership in the `is_platform`
//! organization, and every ordinary provisioning door refuses to write that
//! membership, so this job is the only automatic path. These tests pin its
//! contract: idempotent, self-healing when the admin user does not exist yet,
//! and strictly non-destructive when the admin is settled in another
//! organization.

use systemprompt_web_extension::jobs::internals::{ReconcileOutcome, reconcile_platform_admin};

use crate::fixtures::{OrgSpec, insert_member, insert_org, insert_user_full, unique};
use crate::tempdb::TempDb;

const HOUSE: &str = "house";
const ADMIN_ROLES: &[&str] = &["admin", "user"];

async fn admin_user(db: &TempDb, name: &str) -> systemprompt::identifiers::UserId {
    let roles: Vec<String> = ADMIN_ROLES.iter().map(|r| (*r).to_owned()).collect();
    insert_user_full(&db.pool, &unique("u"), name, Some(name), &roles, "active").await
}

#[tokio::test]
async fn a_fresh_install_grants_platform_membership_to_the_system_admin() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = format!("{}@astound.example", unique("admin"));
    let user_id = admin_user(&db, &name).await;

    let outcome = reconcile_platform_admin(&db.pool, &name)
        .await
        .expect("reconcile");
    assert_eq!(outcome, ReconcileOutcome::Granted);

    let org = systemprompt_web_admin::repositories::organizations::crud::find_membership_org(
        &db.pool, &user_id,
    )
    .await
    .expect("membership query");
    assert_eq!(org.as_deref(), Some(HOUSE));
}

#[tokio::test]
async fn a_second_run_is_a_no_op() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = format!("{}@astound.example", unique("admin"));
    admin_user(&db, &name).await;

    let first = reconcile_platform_admin(&db.pool, &name)
        .await
        .expect("first run");
    let second = reconcile_platform_admin(&db.pool, &name)
        .await
        .expect("second run");
    assert_eq!(first, ReconcileOutcome::Granted);
    assert_eq!(second, ReconcileOutcome::AlreadyPlatformMember);
}

#[tokio::test]
async fn a_missing_admin_user_warns_and_succeeds() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let outcome = reconcile_platform_admin(&db.pool, "nobody@astound.example")
        .await
        .expect("reconcile must not fail the boot");
    assert_eq!(outcome, ReconcileOutcome::AdminUserMissing);
}

#[tokio::test]
async fn an_admin_settled_in_another_organization_is_not_moved() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = format!("{}@astound.example", unique("admin"));
    let user_id = admin_user(&db, &name).await;

    let customer_org = unique("org");
    insert_org(&db.pool, &OrgSpec::active(&customer_org, &customer_org)).await;
    insert_member(&db.pool, &user_id, &customer_org, "admin").await;

    let outcome = reconcile_platform_admin(&db.pool, &name)
        .await
        .expect("reconcile");
    assert_eq!(outcome, ReconcileOutcome::SettledInAnotherOrg);

    let org = systemprompt_web_admin::repositories::organizations::crud::find_membership_org(
        &db.pool, &user_id,
    )
    .await
    .expect("membership query");
    assert_eq!(org.as_deref(), Some(customer_org.as_str()));
}
