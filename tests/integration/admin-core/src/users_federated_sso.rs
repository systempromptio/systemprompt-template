//! `repositories::users::federated` — the SSO resolution order, the profile
//! connect/disconnect writes, and the role projection a returning sign-in
//! performs.
//!
//! The role rule is the load-bearing one: the directory owns its own half of
//! the set and a manual grant survives it, so a sign-in that demotes someone
//! in AD must not also erase what an admin gave them by hand.

use systemprompt_web_admin::repositories::users::federated::{
    FederatedClaims, LinkOutcome, delete_federated_identities_for_issuer, link_identity_to_user,
    resolve_federated_user,
};

use systemprompt_web_admin::repositories::users::roles::{list_manual_roles, set_manual_roles};

use crate::fixtures::{
    insert_federated_identity, insert_user, insert_user_full, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

pub const ISSUER: &str = "https://login.adfs.test/adfs";

const USER_ROLE: &[String] = &[];

fn claims<'a>(external_sub: &'a str, email: &'a str) -> FederatedClaims<'a> {
    FederatedClaims {
        issuer: ISSUER,
        external_sub,
        email,
        display_name: "Federated Person",
        roles: USER_ROLE,
    }
}

#[tokio::test]
async fn resolve_federated_user_returns_the_existing_mapping_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("mapped")).await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;

    let resolved = resolve_federated_user(&db.pool, &claims(&sub, "other@elsewhere.test"), false)
        .await
        .expect("resolution succeeds")
        .expect("an existing mapping resolves without provisioning");

    assert_eq!(resolved.user_id, user, "the mapping wins over the email");
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_links_a_verified_email_to_an_active_local_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("merge");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    let sub = unique("sub");

    let resolved = resolve_federated_user(&db.pool, &claims(&sub, &email), false)
        .await
        .expect("resolution succeeds")
        .expect("an active local account is linked rather than duplicated");

    assert_eq!(resolved.user_id, user);
    let owner: Option<String> = sqlx::query_scalar(
        "SELECT user_id FROM federated_identities WHERE issuer = $1 AND external_sub = $2",
    )
    .bind(ISSUER)
    .bind(&sub)
    .fetch_optional(&*db.pool)
    .await
    .expect("mapping lookup succeeds");
    assert_eq!(
        owner.as_deref(),
        Some(user.as_str()),
        "the merge writes the mapping it resolved through"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_matches_the_email_case_insensitively() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("case");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    let shouted = email.to_uppercase();

    let resolved = resolve_federated_user(&db.pool, &claims(&unique("sub"), &shouted), false)
        .await
        .expect("resolution succeeds")
        .expect("an upper-cased claim still finds the local account");

    assert_eq!(resolved.user_id, user);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_ignores_an_inactive_local_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("inactive");
    insert_user_full(
        &db.pool,
        &unique("user"),
        &email,
        None,
        &["user".to_owned()],
        "inactive",
    )
    .await;

    let resolved = resolve_federated_user(&db.pool, &claims(&unique("sub"), &email), false)
        .await
        .expect("resolution succeeds");

    assert!(
        resolved.is_none(),
        "only an active account may be merged into; provisioning is off"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_returns_none_when_provisioning_is_off() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let resolved = resolve_federated_user(
        &db.pool,
        &claims(&unique("sub"), &unclaimed_email("stranger")),
        false,
    )
    .await
    .expect("resolution succeeds");

    assert!(
        resolved.is_none(),
        "an unknown identity is not an error — the caller says 'ask an admin'"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_provisions_when_asked_to() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("jit");
    let sub = unique("sub");

    let resolved = resolve_federated_user(&db.pool, &claims(&sub, &email), true)
        .await
        .expect("resolution succeeds")
        .expect("auto_provision mints the account");

    assert_eq!(resolved.email, email);
    assert_eq!(resolved.display_name, "Federated Person");
    assert_eq!(resolved.roles, vec!["user".to_owned()]);
    let status: String = sqlx::query_scalar("SELECT status FROM users WHERE id = $1")
        .bind(resolved.user_id.as_str())
        .fetch_one(&*db.pool)
        .await
        .expect("the provisioned user exists");
    assert_eq!(status, "active");
    db.cleanup().await;
}

#[tokio::test]
async fn a_sign_in_keeps_a_manually_granted_role_the_directory_never_mentions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("keepsmanual")).await;
    let granter = insert_user(&db.pool, &unique("admin"), &unclaimed_email("ssogranter")).await;
    set_manual_roles(&db.pool, &user, &["developer".to_owned()], &granter)
        .await
        .expect("grant developer by hand");
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;
    let mapped = ["user".to_owned()];
    let claims = FederatedClaims {
        roles: &mapped,
        ..claims(&sub, "other@elsewhere.test")
    };

    let resolved = resolve_federated_user(&db.pool, &claims, false)
        .await
        .expect("resolution succeeds")
        .expect("an existing mapping resolves");

    assert_eq!(
        resolved.roles,
        vec!["developer".to_owned(), "user".to_owned()],
        "the effective set is the union, not the assertion alone"
    );
    assert_eq!(
        list_manual_roles(&db.pool, &user)
            .await
            .expect("list manual"),
        vec!["developer".to_owned()],
        "and the manual grant is still recorded as one"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn losing_platform_admin_in_the_directory_reports_a_demotion() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("wasplatform"),
        None,
        &["user".to_owned(), "platform_admin".to_owned()],
        "active",
    )
    .await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;
    let demoted = ["user".to_owned()];
    let claims = FederatedClaims {
        roles: &demoted,
        ..claims(&sub, "other@elsewhere.test")
    };

    let resolved = resolve_federated_user(&db.pool, &claims, false)
        .await
        .expect("resolution succeeds")
        .expect("an existing mapping resolves");

    assert!(
        resolved.lost_admin,
        "platform_admin mints the same token permission as admin, so losing it must revoke too"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn link_identity_to_user_refuses_to_steal_another_users_mapping() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = insert_user(&db.pool, &unique("user"), &unclaimed_email("owner")).await;
    let thief = insert_user(&db.pool, &unique("user"), &unclaimed_email("thief")).await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &owner).await;

    let outcome = link_identity_to_user(&db.pool, ISSUER, &sub, &thief)
        .await
        .expect("link attempt succeeds");

    assert_eq!(outcome, LinkOutcome::AlreadyLinkedElsewhere);
    db.cleanup().await;
}

#[tokio::test]
async fn delete_federated_identities_for_issuer_reports_how_many_went() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("disconnect")).await;
    insert_federated_identity(&db.pool, ISSUER, &unique("sub"), &user).await;
    insert_federated_identity(&db.pool, ISSUER, &unique("sub"), &user).await;
    insert_federated_identity(&db.pool, "https://other.test", &unique("sub"), &user).await;

    let removed = delete_federated_identities_for_issuer(&db.pool, &user, ISSUER)
        .await
        .expect("delete succeeds");

    assert_eq!(removed, 2, "only this issuer's mappings are removed");
    let left: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM federated_identities WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("count succeeds");
    assert_eq!(left, 1, "the other issuer's mapping survives");
    db.cleanup().await;
}

#[tokio::test]
async fn delete_federated_identities_for_issuer_is_zero_when_nothing_matches() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nothing")).await;

    let removed = delete_federated_identities_for_issuer(&db.pool, &user, ISSUER)
        .await
        .expect("delete succeeds");

    assert_eq!(removed, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_reprojects_mapped_roles_on_a_returning_login() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("promoted"),
        None,
        &["user".to_owned(), "admin".to_owned()],
        "active",
    )
    .await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;
    let demoted = ["user".to_owned()];
    let claims = FederatedClaims {
        roles: &demoted,
        ..claims(&sub, "other@elsewhere.test")
    };

    let resolved = resolve_federated_user(&db.pool, &claims, false)
        .await
        .expect("resolution succeeds")
        .expect("an existing mapping resolves");

    assert_eq!(
        resolved.roles, demoted,
        "the directory's roles replace the row's"
    );
    let stored: Vec<String> = sqlx::query_scalar("SELECT roles FROM users WHERE id = $1")
        .bind(user.as_str())
        .fetch_one(&*db.pool)
        .await
        .expect("read roles");
    assert_eq!(stored, demoted, "and the row itself was rewritten");
    assert!(
        resolved.lost_admin,
        "the caller is told, so it can revoke the tokens minted while the role held"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_does_not_report_a_demotion_on_a_lateral_change() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("lateral"),
        None,
        &["user".to_owned(), "admin".to_owned()],
        "active",
    )
    .await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;
    let reordered = ["admin".to_owned(), "user".to_owned()];
    let claims = FederatedClaims {
        roles: &reordered,
        ..claims(&sub, "other@elsewhere.test")
    };

    let resolved = resolve_federated_user(&db.pool, &claims, false)
        .await
        .expect("resolution succeeds")
        .expect("an existing mapping resolves");

    assert!(
        !resolved.lost_admin,
        "admin is still held — reordering must not tear down a live bridge"
    );
    db.cleanup().await;
}

// Why: these four columns are exactly what the signed bridge manifest's
// `UserInfo` and `GET /v1/bridge/whoami` read back, so this is the test that
// an ADFS-provisioned user arrives at the desktop app with a name and the
// roles their AD group mapped to — not a bare id.
#[tokio::test]
async fn just_in_time_provisioning_writes_the_identity_the_bridge_reads() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("newstarter");
    let sub = unique("sub");
    let mapped = ["admin".to_owned(), "user".to_owned()];
    let claims = FederatedClaims {
        roles: &mapped,
        ..claims(&sub, &email)
    };

    let resolved = resolve_federated_user(&db.pool, &claims, true)
        .await
        .expect("provisioning succeeds")
        .expect("auto_provision mints the account");

    let (name, stored_email, display_name, roles): (String, String, Option<String>, Vec<String>) =
        sqlx::query_as("SELECT name, email, display_name, roles FROM users WHERE id = $1")
            .bind(resolved.user_id.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the row exists");

    assert_eq!(stored_email, email);
    assert_eq!(name, email, "name falls back to the address AD asserted");
    assert_eq!(display_name.as_deref(), Some("Federated Person"));
    assert_eq!(roles, mapped, "the AD group map decides the roles");
    assert!(!resolved.lost_admin, "a fresh account has lost nothing");
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_federated_user_keeps_local_roles_when_the_idp_maps_none() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("operator"),
        None,
        &["admin".to_owned()],
        "active",
    )
    .await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;

    let resolved = resolve_federated_user(&db.pool, &claims(&sub, "x@elsewhere.test"), false)
        .await
        .expect("resolution succeeds")
        .expect("an existing mapping resolves");

    assert_eq!(
        resolved.roles,
        vec!["admin".to_owned()],
        "an empty claim is not a demotion"
    );
    db.cleanup().await;
}
