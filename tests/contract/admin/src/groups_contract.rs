//! The group and project routes over the real router: which tier each verb
//! sits on, and the two refusals the system group carries.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const CREATE_BODY: &str = r#"{"id":"contract-made","name":"Made by the contract suite"}"#;

#[tokio::test]
async fn an_admin_creates_reads_and_deletes_a_group() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (created, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/groups",
            Principal::Admin,
            CREATE_BODY,
        ))
        .await;
    assert_eq!(created, StatusCode::CREATED, "body: {body}");

    let (read, body) = app
        .call(Call::get(
            "/api/public/admin/groups/contract-made",
            Principal::Admin,
        ))
        .await;
    assert_eq!(read, StatusCode::OK, "body: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(json["group"]["id"], "contract-made");
    assert_eq!(json["member_count"], 0);
    assert_eq!(json["marketplace_ids"], serde_json::json!([]));

    let (deleted, body) = app
        .call(Call::json(
            "delete",
            "/api/public/admin/groups/contract-made",
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(deleted, StatusCode::NO_CONTENT, "body: {body}");

    let (gone, _) = app
        .call(Call::get(
            "/api/public/admin/groups/contract-made",
            Principal::Admin,
        ))
        .await;
    assert_eq!(gone, StatusCode::NOT_FOUND);
    db.cleanup().await;
}

// Why: an id that fails the column's CHECK should name the field, not surface
// a constraint name through a 500.
#[tokio::test]
async fn a_malformed_group_id_is_a_bad_request() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/groups",
            Principal::Admin,
            r#"{"id":"Not A Slug","name":"Bad"}"#,
        ))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "body: {body}");
    db.cleanup().await;
}

// Why: `unassigned` is a derived set. Deleting or editing it would describe a
// membership the database computes rather than one an admin controls.
#[tokio::test]
async fn the_system_group_refuses_every_write() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (members, body) = app
        .call(Call::get(
            "/api/public/admin/groups/unassigned/members",
            Principal::Admin,
        ))
        .await;
    assert_eq!(
        members,
        StatusCode::OK,
        "reading the derived membership works: {body}"
    );

    let (renamed, body) = app
        .call(Call::json(
            "put",
            "/api/public/admin/groups/unassigned",
            Principal::Admin,
            r#"{"name":"Renamed"}"#,
        ))
        .await;
    assert_eq!(renamed, StatusCode::CONFLICT, "body: {body}");

    let (removed, body) = app
        .call(Call::json(
            "delete",
            "/api/public/admin/groups/unassigned",
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(removed, StatusCode::CONFLICT, "body: {body}");
    db.cleanup().await;
}

// Why: a mapping decides what the directory grants everyone holding that AD
// group, so it sits a tier above an ordinary admin write.
#[tokio::test]
async fn an_admin_retains_authority_to_manage_ad_mappings() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    app.call(Call::json(
        "post",
        "/api/public/admin/groups",
        Principal::Admin,
        CREATE_BODY,
    ))
    .await;

    let (read, _) = app
        .call(Call::get(
            "/api/public/admin/groups/contract-made/ad-mappings",
            Principal::Admin,
        ))
        .await;
    let (write, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/groups/contract-made/ad-mappings",
            Principal::Admin,
            r#"{"ad_group":"AD-Contract"}"#,
        ))
        .await;

    assert_eq!(read, StatusCode::OK, "reading is a console act");
    assert_eq!(
        write,
        StatusCode::CREATED,
        "an admin can configure a mapping: {body}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_project_manager_reads_groups_and_writes_none() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (list, body) = app
        .call(Call::get(
            "/api/public/admin/groups",
            Principal::ProjectManager,
        ))
        .await;
    assert_eq!(list, StatusCode::OK, "body: {body}");

    let (create, _) = app
        .call(Call::json(
            "post",
            "/api/public/admin/groups",
            Principal::ProjectManager,
            CREATE_BODY,
        ))
        .await;
    assert_eq!(create, StatusCode::FORBIDDEN);
    db.cleanup().await;
}

#[tokio::test]
async fn a_non_admin_reaches_no_group_route() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (list, _) = app
        .call(Call::get("/api/public/admin/groups", Principal::NonAdmin))
        .await;
    let (projects, _) = app
        .call(Call::get("/api/public/admin/projects", Principal::NonAdmin))
        .await;

    assert_eq!(list, StatusCode::FORBIDDEN);
    assert_eq!(projects, StatusCode::FORBIDDEN);
    db.cleanup().await;
}

#[tokio::test]
async fn group_usage_answers_with_an_empty_rollup_for_a_fresh_group() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    app.call(Call::json(
        "post",
        "/api/public/admin/groups",
        Principal::Admin,
        CREATE_BODY,
    ))
    .await;

    let (status, body) = app
        .call(Call::get(
            "/api/public/admin/groups/contract-made/usage?range=7d",
            Principal::Admin,
        ))
        .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    let json: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(json["range"], "7d");
    assert_eq!(json["summary"]["requests"], 0);
    assert_eq!(json["top_models"], serde_json::json!([]));
    db.cleanup().await;
}

// The recompute endpoint exists because attribution keys drift: the directory
// replaces a signer-in's whole membership set at each sign-in without passing
// through the API that rewrites them. What is pinned here is that it answers
// with the count it wrote, that it sits on the admin tier, and that it is
// idempotent — running it twice must not move anyone a second time, or an
// operator could not press it safely.
#[tokio::test]
async fn recomputing_attribution_keys_reports_its_writes_and_repeats_cleanly() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/scope-defaults/recompute",
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let first: serde_json::Value = serde_json::from_str(&body).expect("json");
    let written = first["recomputed"].as_i64().expect("a write count");

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/scope-defaults/recompute",
            Principal::Admin,
            "{}",
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "body: {body}");
    let second: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(
        second["recomputed"].as_i64(),
        Some(written),
        "the second run touched a different number of rows"
    );

    for principal in [Principal::NonAdmin, Principal::Developer] {
        let (status, body) = app
            .call(Call::json(
                "post",
                "/api/public/admin/scope-defaults/recompute",
                principal,
                "{}",
            ))
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{principal:?}: {body}");
    }

    db.cleanup().await;
}

// Why: upgrades must introduce groups alongside departments, without copying
// memberships or deleting the old authorization rules.
#[tokio::test]
async fn group_schema_reapplication_preserves_department_membership_and_denials() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user =
        crate::seed::insert_user(&db.pool, "legacy-dept-user", "legacy-dept@contract.test").await;
    sqlx::query("INSERT INTO departments (id, name) VALUES ('legacy-dept', 'Legacy department')")
        .execute(&*db.pool)
        .await
        .expect("legacy department");
    sqlx::query(
        "INSERT INTO user_profile_ext (user_id, department) VALUES ($1, 'Legacy department')",
    )
    .bind(user.as_str())
    .execute(&*db.pool)
    .await
    .expect("legacy assignment");
    crate::seed::insert_acl_rule(
        &db.pool,
        &crate::seed::AclRule {
            entity_type: "mcp_server",
            entity_id: "legacy-mcp",
            rule_type: "department",
            rule_value: "Legacy department",
            access: "deny",
        },
    )
    .await;
    sqlx::raw_sql(include_str!(
        "../../../../extensions/web/schema/migrations/052_dashboard_groups_projects.sql"
    ))
    .execute(&*db.pool)
    .await
    .expect("reapply additive migration");
    let department: String =
        sqlx::query_scalar("SELECT department FROM user_profile_ext WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("department retained");
    assert_eq!(department, "Legacy department");
    let denials: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM access_control_rules WHERE rule_type = 'department' AND rule_value = 'Legacy department' AND access = 'deny'")
        .fetch_one(&*db.pool).await.expect("legacy ACL retained");
    assert_eq!(denials, 1);
    let mapped: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM group_members WHERE user_id = $1")
        .bind(user.as_str())
        .fetch_one(&*db.pool)
        .await
        .expect("no inferred mapping");
    assert_eq!(mapped, 0);
    db.cleanup().await;
}
