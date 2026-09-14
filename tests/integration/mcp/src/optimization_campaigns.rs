//! Campaign replay, ownership, shared-budget and iteration-boundary coverage.

use systemprompt::evaluation::campaigns::repository::{CampaignAction, CampaignRepository};
use systemprompt::evaluation::campaigns::{CampaignPolicy, OptimizationObjective};
use systemprompt::evaluation::experiments::BudgetRepository;
use systemprompt::identifiers::{EvalExperimentId, ManagedResourceId, ResourceRevisionId, UserId};

#[tokio::test]
async fn campaign_creation_replay_and_state_transitions_preserve_owner_and_actor() {
    let db = template_test_common::db_or_skip!();
    let owner = UserId::new("campaign-owner");
    let actor = UserId::new("campaign-reviewer");
    let stranger = UserId::new("campaign-stranger");
    for user in [&owner, &actor, &stranger] {
        sqlx::query("INSERT INTO users(id,name,email) VALUES($1,$1,$2)")
            .bind(user.as_str())
            .bind(format!("{user}@example.invalid"))
            .execute(&*db.pool)
            .await
            .unwrap();
    }
    let budgets = BudgetRepository::new((*db.pool).clone());
    let budget_id = budgets
        .create_shared(&owner, "campaign-budget", 1_000_000)
        .await
        .unwrap();
    let policy = CampaignPolicy {
        name: "Token efficiency".to_owned(),
        resource_id: ManagedResourceId::generate(),
        baseline_revision_id: ResourceRevisionId::generate(),
        budget_id,
        objective: OptimizationObjective::Tokens,
        minimum_quality_milli: 4000,
        minimum_pairs: 10,
        maximum_iterations: 1,
        automatic: true,
    };
    let repository = CampaignRepository::new((*db.pool).clone());
    let id = repository
        .create(&owner, &actor, "create", &policy)
        .await
        .unwrap();
    assert_eq!(
        id,
        repository
            .create(&owner, &actor, "create", &policy)
            .await
            .unwrap()
    );
    assert!(repository.get(&stranger, &id).await.is_err());
    let mut different = policy.clone();
    different.name = "Changed".to_owned();
    assert!(
        repository
            .create(&owner, &actor, "create", &different)
            .await
            .is_err()
    );
    let record = repository.get(&owner, &id).await.unwrap();
    assert_eq!(record.created_by, actor);
    let experiment = EvalExperimentId::generate();
    sqlx::query("INSERT INTO eval_experiments(id,owner_id,spec,spec_digest,budget_id,idempotency_key) VALUES($1,$2,'{}','test',$3,'experiment')").bind(experiment.as_str()).bind(owner.as_str()).bind(policy.budget_id.as_str()).execute(&*db.pool).await.unwrap();
    repository
        .attach_experiment(&owner, &actor, &id, &experiment)
        .await
        .unwrap();
    repository
        .attach_experiment(&owner, &actor, &id, &experiment)
        .await
        .unwrap();
    assert_eq!(
        repository.list_experiments(&owner, &id).await.unwrap(),
        vec![experiment]
    );
    assert!(
        repository
            .transition(&owner, &actor, &id, (0, CampaignAction::Pause))
            .await
            .is_err()
    );
    repository
        .transition(&owner, &actor, &id, (1, CampaignAction::Cancel))
        .await
        .unwrap();
    assert!(
        repository
            .transition(&owner, &actor, &id, (2, CampaignAction::Resume))
            .await
            .is_err()
    );
    let actors: Vec<String> = sqlx::query_scalar(
        "SELECT actor_id FROM eval_campaign_events WHERE campaign_id=$1 ORDER BY generation",
    )
    .bind(id.as_str())
    .fetch_all(&*db.pool)
    .await
    .unwrap();
    assert_eq!(actors, vec![actor.to_string(); 3]);
    db.cleanup().await;
}
