use super::setup_test_pool;
use systemprompt_core_oauth::repository::ClientRepository;

#[tokio::test]
#[ignore]
async fn test_client_lifecycle() {
    let pool = setup_test_pool().await;
    let repo = ClientRepository::new(pool);

    let client_id = "test_client_lifecycle";
    let redirect_uris = vec!["http://localhost:3000/callback".to_string()];
    let grant_types = vec!["authorization_code".to_string(), "refresh_token".to_string()];
    let response_types = vec!["code".to_string()];
    let scopes = vec!["openid".to_string(), "profile".to_string()];

    let created = repo
        .create(
            client_id,
            "hash_of_secret",
            "Test Client",
            &redirect_uris,
            Some(&grant_types),
            Some(&response_types),
            &scopes,
            Some("client_secret_post"),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create client");

    assert_eq!(created.client_id, client_id);
    assert!(created.is_active);
    assert_eq!(created.scopes.len(), 2);
    assert_eq!(created.redirect_uris.len(), 1);

    let found = repo
        .get_by_client_id(client_id)
        .await
        .expect("Failed to get client")
        .expect("Client not found");

    assert_eq!(found.client_id, created.client_id);
    assert_eq!(found.scopes, created.scopes);

    repo.deactivate(client_id)
        .await
        .expect("Failed to deactivate client");

    let inactive = repo
        .get_by_client_id(client_id)
        .await
        .expect("Failed to get client");

    assert!(inactive.is_none(), "Deactivated client should not be found");

    let found_any = repo
        .get_by_client_id_any(client_id)
        .await
        .expect("Failed to get client (any)")
        .expect("Client not found (any)");

    assert!(!found_any.is_active, "Client should be inactive");

    repo.activate(client_id)
        .await
        .expect("Failed to activate client");

    let reactivated = repo
        .get_by_client_id(client_id)
        .await
        .expect("Failed to get client")
        .expect("Client not found after reactivation");

    assert!(reactivated.is_active, "Client should be active again");

    repo.delete(client_id)
        .await
        .expect("Failed to delete client");

    let deleted = repo
        .get_by_client_id(client_id)
        .await
        .expect("Failed to query deleted client");

    assert!(deleted.is_none(), "Deleted client should not be found");
}

#[tokio::test]
#[ignore]
async fn test_client_update() {
    let pool = setup_test_pool().await;
    let repo = ClientRepository::new(pool);

    let client_id = "test_client_update";
    let original_scopes = vec!["openid".to_string()];
    let new_scopes = vec!["openid".to_string(), "email".to_string()];
    let new_uris = vec!["http://localhost:4000/callback".to_string()];

    repo.create(
        client_id,
        "hash_of_secret",
        "Original Name",
        &vec!["http://localhost:3000/callback".to_string()],
        None,
        None,
        &original_scopes,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("Failed to create client");

    let updated = repo
        .update(
            client_id,
            "Updated Name",
            &new_uris,
            None,
            None,
            &new_scopes,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to update client")
        .expect("Client not found after update");

    assert_eq!(updated.client_name, "Updated Name");
    assert_eq!(updated.scopes.len(), 2);
    assert!(updated.scopes.contains(&"email".to_string()));
    assert_eq!(updated.redirect_uris[0], "http://localhost:4000/callback");
}

#[tokio::test]
#[ignore]
async fn test_client_secret_update() {
    let pool = setup_test_pool().await;
    let repo = ClientRepository::new(pool);

    let client_id = "test_client_secret_update";

    repo.create(
        client_id,
        "original_hash",
        "Test Client",
        &vec!["http://localhost:3000/callback".to_string()],
        None,
        None,
        &vec!["openid".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .expect("Failed to create client");

    let updated = repo
        .update_secret(client_id, "new_hash")
        .await
        .expect("Failed to update secret")
        .expect("Client not found");

    assert_eq!(updated.client_secret_hash, Some("new_hash".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_client_counting() {
    let pool = setup_test_pool().await;
    let repo = ClientRepository::new(pool);

    let count_before = repo
        .count()
        .await
        .expect("Failed to count clients");

    repo.create(
        "test_count_client_1",
        "hash",
        "Count Test 1",
        &vec!["http://localhost:3000/callback".to_string()],
        None,
        None,
        &vec!["openid".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .expect("Failed to create client 1");

    repo.create(
        "test_count_client_2",
        "hash",
        "Count Test 2",
        &vec!["http://localhost:3000/callback".to_string()],
        None,
        None,
        &vec!["openid".to_string()],
        None,
        None,
        None,
        None,
    )
    .await
    .expect("Failed to create client 2");

    let count_after = repo
        .count()
        .await
        .expect("Failed to count clients");

    assert_eq!(count_after, count_before + 2);
}
