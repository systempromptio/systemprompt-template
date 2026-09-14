//! Shared experiment repositories constructed once for the admin router.

use sqlx::PgPool;
use systemprompt::evaluation::experiments::{ExperimentRepository, RevisionRepository};
use systemprompt::evaluation::repository::experiments::{
    BudgetRepository, EvaluationLifecycleRepository, EvidenceRepository, WorkerRepository,
};

#[derive(Debug)]
pub(crate) struct EvaluationState {
    pub(crate) optimization: systemprompt::system::optimization::SkillOptimizationOrchestrator,
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) campaigns: systemprompt::evaluation::campaigns::repository::CampaignRepository,
    pub(crate) pool: PgPool,
    pub(crate) experiments: ExperimentRepository,
    pub(crate) workers: WorkerRepository,
    pub(crate) evidence: EvidenceRepository,
    pub(crate) revisions: RevisionRepository,
    pub(crate) budgets: BudgetRepository,
    pub(crate) lifecycle: EvaluationLifecycleRepository,
}

impl EvaluationState {
    pub(crate) fn new(pool: PgPool, owner: systemprompt::identifiers::UserId) -> Self {
        let repositories =
            systemprompt::evaluation::repository::experiments::EvaluationRepositories::new(&pool);
        let managed = systemprompt::marketplace::managed::ManagedRepository::new(pool.clone());
        Self {
            optimization: systemprompt::system::optimization::SkillOptimizationOrchestrator::new(
                managed,
                repositories.clone(),
                repositories.revisions.clone(),
            ),
            owner,
            campaigns: repositories.campaigns,
            pool,
            experiments: repositories.experiments,
            workers: repositories.workers,
            evidence: repositories.evidence,
            revisions: repositories.revisions,
            budgets: repositories.budgets,
            lifecycle: repositories.lifecycle,
        }
    }
}
