use crate::activity::types::{ActivityCategorySummary, ActivityTimelineEvent};
use crate::types::{EventsQuery, CATEGORY_AI_SESSIONS, CATEGORY_EDITS};

use super::super::types::MyActivityPageData;

pub(super) struct ActivityStats {
    pub total_activities: i64,
    pub total_edits: i64,
    pub total_sessions: i64,
    pub xp_progress_pct: u32,
}

pub(super) fn compute_activity_stats(
    category_summary: &[ActivityCategorySummary],
    gamification: Option<&crate::types::UserGamificationProfile>,
    total: i64,
) -> ActivityStats {
    ActivityStats {
        total_activities: total,
        total_edits: category_summary
            .iter()
            .find(|c| c.category == CATEGORY_EDITS)
            .map_or(0, |c| c.count),
        total_sessions: category_summary
            .iter()
            .find(|c| c.category == CATEGORY_AI_SESSIONS)
            .map_or(0, |c| c.count),
        xp_progress_pct: gamification.map_or(0, |g| {
            if g.xp_to_next_rank > 0 {
                let total_for_rank = g.total_xp + g.xp_to_next_rank;
                if total_for_rank > 0 {
                    u32::try_from(g.total_xp.saturating_mul(100) / total_for_rank).unwrap_or(100)
                } else {
                    100
                }
            } else {
                100
            }
        }),
    }
}

pub(super) struct BuildActivityTemplateParams<'a> {
    pub activities: &'a [ActivityTimelineEvent],
    pub total: i64,
    pub query: &'a EventsQuery,
    pub category_summary: &'a [ActivityCategorySummary],
    pub gamification: Option<&'a crate::types::UserGamificationProfile>,
    pub stats: &'a ActivityStats,
}

pub(super) fn build_activity_template(
    params: &BuildActivityTemplateParams<'_>,
) -> MyActivityPageData {
    let query = params.query;
    let stats = params.stats;
    let has_prev = query.offset > 0;
    let has_next = query.offset + query.limit < params.total;
    let prev_offset = if query.offset >= query.limit {
        query.offset - query.limit
    } else {
        0
    };
    MyActivityPageData {
        page: "my-activity",
        title: "My Activity",
        events: params.activities.to_vec(),
        total: params.total,
        limit: query.limit,
        offset: query.offset,
        has_prev,
        has_next,
        prev_offset,
        next_offset: query.offset + query.limit,
        search: query.search.clone(),
        category_summary: params.category_summary.to_vec(),
        gamification: params.gamification.cloned(),
        total_activities: stats.total_activities,
        total_edits: stats.total_edits,
        total_sessions: stats.total_sessions,
        xp_progress_pct: stats.xp_progress_pct,
    }
}
