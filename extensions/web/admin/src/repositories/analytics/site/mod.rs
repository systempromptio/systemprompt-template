//! Read side of the site analytics dashboard (`/admin/analytics`).
//!
//! Every query here takes the same [`SiteScope`] and applies it with static
//! null-skip binds over LEFT JOINs to `organizations`/`organization_members`
//! (one org per user, so the join cannot fan out) and `user_profile_ext`.
//! Department matching uses the same `COALESCE(NULLIF(…,''),'Default')`
//! normalisation as `departments::summaries`, so users with an empty
//! department don't vanish when the filter is active. Synthetic gateway rows
//! (`ai_requests.synthetic`) are excluded everywhere: demo traffic is not a
//! business metric.

use systemprompt::identifiers::UserId;

use crate::util::org_scope::OrgScope;

pub mod code;
pub mod distribution;
pub mod kpis;
pub mod latency;
pub mod leaderboards;
pub mod model_series;
pub mod seats;
pub mod series;
pub mod session_costs;
pub mod user_rollups;

/// Drill-down filters, all conjunctive. `org_slug` is the URL-facing
/// organization key (slugs are immutable; ids are not typed by hand).
#[derive(Debug, Clone)]
pub struct SiteScope {
    pub org_slug: OrgScope,
    pub department: Option<String>,
    pub user_id: Option<UserId>,
}

// Why: Written out rather than derived because the derived default for the
// organization field would be the cross-customer view, making the widest scope
// the one a caller reaches by naming nothing.
impl Default for SiteScope {
    fn default() -> Self {
        Self {
            org_slug: OrgScope::AllOrganizations,
            department: None,
            user_id: None,
        }
    }
}

impl SiteScope {
    #[must_use]
    pub fn user_id_str(&self) -> Option<&str> {
        self.user_id.as_ref().map(UserId::as_str)
    }
}
