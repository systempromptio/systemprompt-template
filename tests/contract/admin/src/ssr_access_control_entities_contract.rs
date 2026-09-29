//! A populated access-control ledger row must explain both the rule bands and
//! the default fallback an operator is auditing.

use axum::http::StatusCode;
use chrono::{Duration, Utc};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

#[tokio::test(flavor = "multi_thread")]
async fn access_control_ledger_filters_and_explains_expiring_mixed_rules() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let entity = seed::unique("access-ledger-skill");
    let non_expiring_entity = seed::unique("access-ledger-other-skill");
    let group = seed::unique("delivery-group");
    let allow_rule = seed::unique("access-ledger-allow");
    let deny_rule = seed::unique("access-ledger-deny");
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('skill', $1, false, 'dashboard')",
    )
    .bind(&entity)
    .execute(&*db.pool)
    .await
    .expect("register closed skill");
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('skill', $1, false, 'dashboard')",
    )
    .bind(&non_expiring_entity)
    .execute(&*db.pool)
    .await
    .expect("register non-expiring comparison skill");
    for (id, rule_type, subject, access, reason) in [
        (
            &allow_rule,
            "group",
            group.as_str(),
            "allow",
            "delivery workspace exception",
        ),
        (
            &deny_rule,
            "role",
            "intern",
            "deny",
            "interns cannot invoke this skill",
        ),
    ] {
        sqlx::query(
            "INSERT INTO access_control_rules
                 (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
             VALUES ($1, 'skill', $2, $3, $4, $5, $6, 'dashboard')",
        )
        .bind(id)
        .bind(&entity)
        .bind(rule_type)
        .bind(subject)
        .bind(access)
        .bind(reason)
        .execute(&*db.pool)
        .await
        .expect("write access rule");
    }
    sqlx::query("INSERT INTO access_control_rule_validity (rule_id, valid_until) VALUES ($1, $2)")
        .bind(&allow_rule)
        .bind(Utc::now() + Duration::days(3))
        .execute(&*db.pool)
        .await
        .expect("make group grant expiring");
    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
         VALUES ($1, 'skill', $2, 'group', $3, 'allow', 'permanent comparison rule', 'dashboard')",
    )
    .bind(seed::unique("access-ledger-permanent"))
    .bind(&non_expiring_entity)
    .bind(&group)
    .execute(&*db.pool)
    .await
    .expect("write non-expiring comparison rule");

    let path = "/admin/access-control?entity_kind=skill&band=group&expiring=7d&q=access-ledger";
    let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "access ledger: {body}");
    let marker = format!("data-entity-id=\"{entity}\"");
    assert!(
        !body.contains(&format!("data-entity-id=\"{non_expiring_entity}\"")),
        "the matching permanent rule is excluded by the expiry filter: {body}"
    );
    let row_start = body.find(&marker).expect("filtered entity row");
    let row_tail = &body[row_start..];
    let why_cell = row_tail
        .find("<td class=\"sp-ac-entity__why\">")
        .expect("outer entity provenance cell");
    let outer_row_end = why_cell
        + row_tail[why_cell..]
            .find("</tr>")
            .expect("outer entity row closes");
    let row = &row_tail[..outer_row_end];
    assert!(
        row.contains("closed"),
        "the entity default is visible: {row}"
    );
    assert!(
        row.contains(&group),
        "the group band reaches the entity: {row}"
    );
    assert!(
        row.contains("intern"),
        "the denial band remains visible: {row}"
    );
    assert!(
        row.contains("delivery workspace exception")
            && row.contains("interns cannot invoke this skill"),
        "the page preserves each rule's provenance: {row}"
    );
    assert!(
        row.contains("expiring"),
        "the soon-to-expire grant is flagged: {row}"
    );
    assert!(
        row.contains("Refused to anyone who holds role intern.")
            && row.contains(&format!(
                "A person reaches this if they is in group {group}."
            ))
            && row.contains("Everyone else is refused — the entity is closed."),
        "the resolution explains precedence and the closed fallback: {row}"
    );
    db.cleanup().await;
}
