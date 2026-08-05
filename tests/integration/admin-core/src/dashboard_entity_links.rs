//! `repositories::dashboard::conversation_analytics` — the session-to-entity
//! links the hook pipeline records as a session mentions files, tools, and
//! services.
//!
//! The upsert's conflict target is `(user_id, session_id, entity_type,
//! entity_name)`, so the same entity named by two different users, or in two
//! sessions, must stay on separate rows while a repeat within one session
//! increments.

use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::dashboard::conversation_analytics::{
    EntityLinkInput, list_session_entity_links, upsert_session_entity_link,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn link<'a>(
    session_id: &'a str,
    entity_type: &'a str,
    entity_name: &'a str,
) -> EntityLinkInput<'a> {
    EntityLinkInput {
        session_id,
        entity_type,
        entity_name,
        entity_id: None,
    }
}

#[tokio::test]
async fn list_session_entity_links_is_empty_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let links = list_session_entity_links(&db.pool, &SessionId::new(unique("session")))
        .await
        .expect("list links");

    assert!(links.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_session_entity_link_records_a_new_link_with_a_count_of_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("links")).await;
    let session = unique("session");

    upsert_session_entity_link(&db.pool, &user, link(&session, "file", "src/main.rs"))
        .await
        .expect("upsert link");

    let links = list_session_entity_links(&db.pool, &SessionId::new(session))
        .await
        .expect("list links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].entity_type, "file");
    assert_eq!(links[0].entity_name, "src/main.rs");
    assert_eq!(links[0].usage_count, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_session_entity_link_increments_a_repeat_mention() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("linkrepeat")).await;
    let session = unique("session");

    for _ in 0..3 {
        upsert_session_entity_link(&db.pool, &user, link(&session, "tool", "Bash"))
            .await
            .expect("upsert link");
    }

    let links = list_session_entity_links(&db.pool, &SessionId::new(session))
        .await
        .expect("list links");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].usage_count, 3);
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_session_entity_link_keeps_a_later_entity_id_and_never_clears_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("linkid")).await;
    let session = unique("session");
    upsert_session_entity_link(&db.pool, &user, link(&session, "mcp_server", "salesforce"))
        .await
        .expect("upsert link");

    let mut with_id = link(&session, "mcp_server", "salesforce");
    with_id.entity_id = Some("srv-42");
    upsert_session_entity_link(&db.pool, &user, with_id)
        .await
        .expect("upsert link with id");
    upsert_session_entity_link(&db.pool, &user, link(&session, "mcp_server", "salesforce"))
        .await
        .expect("upsert link without id");

    let stored: Option<String> =
        sqlx::query_scalar("SELECT entity_id FROM session_entity_links WHERE session_id = $1")
            .bind(&session)
            .fetch_one(&*db.pool)
            .await
            .expect("read entity id");
    assert_eq!(stored.as_deref(), Some("srv-42"));
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_entity_links_orders_the_most_used_entity_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("linkorder")).await;
    let session = unique("session");
    for _ in 0..2 {
        upsert_session_entity_link(&db.pool, &user, link(&session, "tool", "Read"))
            .await
            .expect("upsert link");
    }
    for _ in 0..5 {
        upsert_session_entity_link(&db.pool, &user, link(&session, "tool", "Bash"))
            .await
            .expect("upsert link");
    }

    let links = list_session_entity_links(&db.pool, &SessionId::new(session))
        .await
        .expect("list links");

    let names: Vec<&str> = links.iter().map(|l| l.entity_name.as_str()).collect();
    assert_eq!(names, ["Bash", "Read"]);
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_session_entity_link_keeps_two_users_on_separate_rows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let first = insert_user(&db.pool, &unique("user"), &unclaimed_email("linkuser1")).await;
    let second = insert_user(&db.pool, &unique("user"), &unclaimed_email("linkuser2")).await;
    let session = unique("session");

    upsert_session_entity_link(&db.pool, &first, link(&session, "tool", "Bash"))
        .await
        .expect("upsert first");
    upsert_session_entity_link(&db.pool, &second, link(&session, "tool", "Bash"))
        .await
        .expect("upsert second");

    let links = list_session_entity_links(&db.pool, &SessionId::new(session))
        .await
        .expect("list links");
    assert_eq!(links.len(), 2, "the conflict key includes user_id");
    assert!(links.iter().all(|l| l.usage_count == 1));
    db.cleanup().await;
}
