//! The developer login link's pure halves: the gate that decides whether the
//! redeem route is mounted and the CLI will issue, the code hash stored at
//! rest, and the URL the CLI prints.
//!
//! The gate is pinned here in full because the contract suite cannot prove
//! the production branch: `ProfileBootstrap` is a per-process `OnceLock`
//! installed once from the development fixture, so the route's absence on a
//! production or cloud profile is only ever a matter of this predicate.

use systemprompt::models::profile::{Environment, ProfileType};
use systemprompt_web_admin::repositories::dev_login::hash_dev_login_code;
use systemprompt_web_admin::{DEV_LOGIN_PATH, dev_login_allowed, dev_login_url};

const ENVIRONMENTS: [Environment; 4] = [
    Environment::Development,
    Environment::Test,
    Environment::Staging,
    Environment::Production,
];

#[test]
fn only_a_development_local_profile_may_offer_dev_login() {
    for environment in ENVIRONMENTS {
        for target in [ProfileType::Local, ProfileType::Cloud] {
            let expected = matches!(environment, Environment::Development)
                && matches!(target, ProfileType::Local);
            assert_eq!(
                dev_login_allowed(environment, target),
                expected,
                "{environment:?} / {target:?}"
            );
        }
    }
}

#[test]
fn a_cloud_profile_is_refused_even_when_labelled_development() {
    assert!(!dev_login_allowed(
        Environment::Development,
        ProfileType::Cloud
    ));
}

#[test]
fn the_stored_hash_is_sha256_hex_and_input_sensitive() {
    let a = hash_dev_login_code("abc");
    assert_eq!(a.len(), 64);
    assert!(
        a.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    );
    assert_eq!(a, hash_dev_login_code("abc"), "deterministic");
    assert_ne!(a, hash_dev_login_code("abd"));
    assert_eq!(
        a, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        "sha256(\"abc\"), so a code cannot be recovered from the row"
    );
}

#[test]
fn the_printed_link_targets_the_redeem_route_under_the_external_url() {
    assert_eq!(
        dev_login_url("http://localhost:8080", "c0de"),
        format!("http://localhost:8080{DEV_LOGIN_PATH}?code=c0de")
    );
    assert_eq!(
        dev_login_url("http://localhost:8080/", "c0de"),
        "http://localhost:8080/admin/auth/dev/login?code=c0de",
        "a trailing slash on the base must not double up"
    );
}

#[test]
fn developer_sessions_preserve_admin_and_console_reader_permissions() {
    use systemprompt::models::auth::Permission;
    use systemprompt_web_admin::test_support::dev_login_permissions_for_roles;
    for role in ["admin", "platform_admin"] {
        assert_eq!(
            dev_login_permissions_for_roles(&[role.to_owned()]),
            vec![Permission::Admin, Permission::User]
        );
    }
    for role in ["user", "project_manager", "sales", "developer"] {
        assert_eq!(
            dev_login_permissions_for_roles(&[role.to_owned()]),
            vec![Permission::User]
        );
    }
}
