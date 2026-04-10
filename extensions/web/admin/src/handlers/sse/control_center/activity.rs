use std::collections::{HashMap, HashSet};

use serde::Serialize;
use sqlx::PgPool;

use crate::handlers::ssr::{build_session_groups_with_status, ssr_control_center::enrichment};
use crate::repositories::{control_center, session_analyses};
use crate::types::control_center::RecentSession;
use crate::types::STATUS_ACTIVE;

use super::SessionGroup;

type SessionEntityLink = crate::types::conversation_analytics::SessionEntityLink;
type SessionRating = crate::types::conversation_analytics::SessionRating;

#[derive(Serialize)]
pub(super) struct ActivityWrapper {
    pub session_groups: Vec<SessionGroup>,
}

pub(super) struct ActivityEvent {
    pub session_groups: Vec<SessionGroup>,
    pub entity_links: Vec<(String, SessionEntityLink)>,
    pub session_ratings: Vec<SessionRating>,
}

pub(super) async fn build_activity_event(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    recent_sessions: &[RecentSession],
) -> ActivityEvent {
    let mut session_ids = Vec::with_capacity(recent_sessions.len());
    let mut active_session_ids = HashSet::new();
    let mut status_map = HashMap::with_capacity(recent_sessions.len());
    for s in recent_sessions {
        session_ids.push(s.session_id.clone());
        if s.ended_at.is_none() && s.status == STATUS_ACTIVE {
            active_session_ids.insert(s.session_id.clone());
        }
        status_map.insert(s.session_id.clone(), s.status.clone());
    }

    let activity_feed = control_center::fetch_session_events(pool, user_id, &session_ids)
        .await
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to fetch session events");
            Vec::new()
        });

    let mut session_groups =
        build_session_groups_with_status(&activity_feed, &active_session_ids, &status_map);

    let (ai_summaries_res, entity_links_res, session_ratings_res, analysed_ids) = tokio::join!(
        crate::repositories::usage_aggregations::fetch_session_ai_summaries(pool, &session_ids),
        crate::repositories::conversation_analytics::fetch_all_session_entity_links(pool, user_id),
        crate::repositories::conversation_analytics::fetch_all_session_ratings(pool, user_id),
        session_analyses::fetch_analysed_session_ids(pool, &session_ids),
    );

    enrich_with_ai_summaries(&mut session_groups, &ai_summaries_res, recent_sessions);

    let entity_links = entity_links_res.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch session entity links");
        Vec::new()
    });
    let session_ratings = session_ratings_res.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch session ratings");
        Vec::new()
    });
    enrichment::enrich_session_groups(&mut session_groups, &session_ratings, &entity_links);

    let analysed_list: Vec<_> = analysed_ids.into_iter().collect();
    for group in &mut session_groups {
        group.flags.is_analysed = analysed_list.iter().any(|id| id == &group.session_id);
    }

    ActivityEvent {
        session_groups,
        entity_links,
        session_ratings,
    }
}

fn enrich_with_ai_summaries(
    session_groups: &mut [SessionGroup],
    ai_summaries_res: &Result<Vec<(String, String, String, String)>, sqlx::Error>,
    recent_sessions: &[RecentSession],
) {
    let Ok(ai_summaries) = ai_summaries_res else {
        return;
    };
    for group in session_groups.iter_mut() {
        if let Some((summary, tags, title)) = ai_summaries
            .iter()
            .find(|(k, _, _, _)| k == &group.session_id)
            .map(|(_, s, t, ti)| (s, t, ti))
        {
            group.ai_summary = Some(summary.clone());
            group.ai_tags = Some(tags.clone());
            if !title.is_empty() {
                group.ai_title = Some(title.clone());
            }
        }
        if let Some(session) = recent_sessions
            .iter()
            .find(|s| s.session_id == group.session_id)
        {
            group.content_bytes = session.content_input_bytes + session.content_output_bytes;
        }
    }
}
