//! The header counts for the access-control page.
//!
//! Three totals an auditor reads before the ledger: how many groups, projects
//! and people the rules below can name. Each read degrades to zero rather
//! than taking the page down; the ledger is the reason the page was opened.

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::scope::SubjectScope;

use super::view::AcStatsView;

pub(super) async fn load_stats(pool: &PgPool) -> AcStatsView {
    let (users, groups, projects) = tokio::join!(
        repositories::users::queries::list_users(pool, &SubjectScope::All),
        repositories::groups::crud::list_groups(pool),
        repositories::projects::crud::list_project_summaries(pool),
    );
    AcStatsView {
        users: users
            .inspect_err(|e| tracing::warn!(error = %e, "access-control: user listing failed"))
            .map(|v| v.len())
            .unwrap_or_default(),
        groups: groups
            .inspect_err(|e| tracing::warn!(error = %e, "access-control: group listing failed"))
            .map(|v| v.len())
            .unwrap_or_default(),
        projects: projects
            .inspect_err(|e| tracing::warn!(error = %e, "access-control: project listing failed"))
            .map(|v| v.len())
            .unwrap_or_default(),
    }
}
