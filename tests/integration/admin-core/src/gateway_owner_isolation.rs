//! Identical client-supplied conversation and tool IDs never cross owners.
use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;
use std::sync::Arc;
use std::time::Duration;
use systemprompt::ai::repository::AiThoughtSignatureRepository;
use systemprompt::api::services::gateway::signature_cache::ThoughtSignatureCache;
use systemprompt::database::{Database, DbPool};
use systemprompt::identifiers::{ContextId, GatewayConversationId};

#[tokio::test]
async fn identical_client_keys_are_isolated_in_memory_and_across_replicas() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = insert_user(&db.pool, &unique("alice"), &unclaimed_email("alice")).await;
    let bob = insert_user(&db.pool, &unique("bob"), &unclaimed_email("bob")).await;
    let pool: DbPool = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let repo = Arc::new(AiThoughtSignatureRepository::new(&pool).expect("repository"));
    let first = ThoughtSignatureCache::new(Duration::from_secs(60), Arc::clone(&repo));
    let second = ThoughtSignatureCache::new(Duration::from_secs(60), repo);
    let conversation = GatewayConversationId::from_prefix_hash(42);
    let alice_context = ContextId::derived_from_gateway_conversation(&alice, &conversation);
    let bob_context = ContextId::derived_from_gateway_conversation(&bob, &conversation);
    assert_ne!(alice_context, bob_context);
    assert_eq!(
        alice_context,
        ContextId::derived_from_gateway_conversation(&alice, &conversation)
    );

    first
        .store(&alice, &conversation, "same-tool", "alice-signature")
        .await;
    assert_eq!(first.lookup(&bob, &conversation, "same-tool").await, None);
    assert_eq!(second.lookup(&bob, &conversation, "same-tool").await, None);
    second
        .store(&bob, &conversation, "same-tool", "bob-signature")
        .await;
    for cache in [&first, &second] {
        assert_eq!(
            cache
                .lookup(&alice, &conversation, "same-tool")
                .await
                .as_deref(),
            Some("alice-signature")
        );
        assert_eq!(
            cache
                .lookup(&bob, &conversation, "same-tool")
                .await
                .as_deref(),
            Some("bob-signature")
        );
    }
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(alice.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("delete owner");
    let fresh = ThoughtSignatureCache::new(
        Duration::from_secs(60),
        Arc::new(AiThoughtSignatureRepository::new(&pool).expect("repository")),
    );
    assert_eq!(fresh.lookup(&alice, &conversation, "same-tool").await, None);
    assert_eq!(
        fresh
            .lookup(&bob, &conversation, "same-tool")
            .await
            .as_deref(),
        Some("bob-signature")
    );
}
