mod data;

use std::sync::Arc;

use crate::activity;
use crate::templates::AdminTemplateEngine;
use crate::types::{EventsQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{AchievementCategoryView, AchievementView};
use data::{
    build_activity_template, category_icon, compute_activity_stats, enrich_achievements,
    rarity_label, BuildActivityTemplateParams, CATEGORY_ORDER,
};

pub async fn my_activity_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> Response {
    let user_id = &user_ctx.user_id;

    let (activities, total) = activity::queries::search_user_entity_activity(
        &pool,
        user_id.as_str(),
        query.search.as_deref(),
        query.limit,
        query.offset,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to search user entity activity");
        (vec![], 0)
    });

    let category_summary = activity::queries::get_user_activity_summary(&pool, user_id.as_str())
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get user activity summary");
            vec![]
        });

    let gamification = crate::gamification::queries::find_user_gamification(
        &pool,
        user_ctx.user_id.as_str(),
    )
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "Failed to fetch user gamification");
    })
    .ok()
    .flatten();

    let achievement_stats = crate::gamification::queries::get_achievement_stats(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch achievement stats");
            vec![]
        });

    let achievements_by_category =
        build_achievements_by_category(gamification.as_ref(), &achievement_stats);
    let unlocked_count = gamification.as_ref().map_or(0, |g| g.achievements.len());
    let total_count = crate::gamification::ACHIEVEMENTS.len();

    let enriched_achievements = enrich_achievements(gamification.as_ref());
    let stats = compute_activity_stats(&category_summary, gamification.as_ref(), total);

    let data = build_activity_template(&BuildActivityTemplateParams {
        activities: &activities,
        total,
        query: &query,
        category_summary: &category_summary,
        gamification: gamification.as_ref(),
        enriched_achievements: &enriched_achievements,
        achievements_by_category: &achievements_by_category,
        unlocked_count,
        total_count,
        stats: &stats,
    });
    let mut value = serde_json::to_value(&data).unwrap_or_else(|_| serde_json::Value::Null);
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "page_stats".to_string(),
            serde_json::json!([
                {"value": stats.total_sessions, "label": "Sessions"},
                {"value": stats.total_edits, "label": "Edits"},
                {"value": total, "label": "Activities"},
            ]),
        );
    }
    super::render_page(&engine, "my-activity", &value, &user_ctx, &mkt_ctx)
}

struct AchievementLookups<'a> {
    unlocked_ids: std::collections::HashSet<&'a str>,
    unlocked_at: std::collections::HashMap<&'a str, &'a chrono::DateTime<chrono::Utc>>,
    stats: std::collections::HashMap<&'a str, &'a crate::types::AchievementInfo>,
}

fn build_lookups<'a>(
    gamification: Option<&'a crate::types::UserGamificationProfile>,
    stats: &'a [crate::types::AchievementInfo],
) -> AchievementLookups<'a> {
    let unlocked_ids = gamification.map_or_else(std::collections::HashSet::new, |g| {
        g.achievements
            .iter()
            .map(|ua| ua.achievement_id.as_str())
            .collect()
    });

    let unlocked_at = gamification.map_or_else(std::collections::HashMap::new, |g| {
        g.achievements
            .iter()
            .map(|ua| (ua.achievement_id.as_str(), &ua.unlocked_at))
            .collect()
    });

    let stats_map = stats.iter().map(|s| (s.id.as_str(), s)).collect();

    AchievementLookups {
        unlocked_ids,
        unlocked_at,
        stats: stats_map,
    }
}

fn build_category_entry(
    cat: &'static str,
    cat_defs: &[&crate::gamification::AchievementDef],
    lookups: &AchievementLookups<'_>,
) -> (Vec<AchievementView>, u32) {
    let mut achievements: Vec<AchievementView> = Vec::new();
    let mut cat_unlocked = 0u32;

    for def in cat_defs {
        let is_unlocked = lookups.unlocked_ids.contains(def.id);
        if is_unlocked {
            cat_unlocked += 1;
        }
        let stat = lookups.stats.get(def.id);
        let pct = stat.map_or(0.0, |s| s.unlock_percentage);

        achievements.push(AchievementView {
            achievement_id: def.id,
            name: def.name,
            description: def.description,
            category: cat,
            is_unlocked,
            rarity_pct: pct,
            rarity_label: rarity_label(pct),
            total_unlocked: stat.map_or(0, |s| s.total_unlocked),
            unlocked_at: lookups.unlocked_at.get(def.id).copied().copied(),
        });
    }

    sort_achievements(&mut achievements);
    (achievements, cat_unlocked)
}

fn sort_achievements(achievements: &mut [AchievementView]) {
    achievements.sort_unstable_by(|a, b| match (a.is_unlocked, b.is_unlocked) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a
            .rarity_pct
            .partial_cmp(&b.rarity_pct)
            .unwrap_or(std::cmp::Ordering::Equal),
    });
}

fn completion_percentage(unlocked: u32, total: u32) -> u32 {
    if total == 0 {
        return 0;
    }
    u32::min(unlocked.saturating_mul(100) / total, 100)
}

fn build_achievements_by_category(
    gamification: Option<&crate::types::UserGamificationProfile>,
    stats: &[crate::types::AchievementInfo],
) -> Vec<AchievementCategoryView> {
    let defs = crate::gamification::ACHIEVEMENTS;
    let lookups = build_lookups(gamification, stats);
    let mut categories: Vec<AchievementCategoryView> = Vec::new();

    for &cat in CATEGORY_ORDER {
        let cat_defs: Vec<_> = defs.iter().filter(|d| d.category == cat).collect();
        if cat_defs.is_empty() {
            continue;
        }

        let total_count = u32::try_from(cat_defs.len()).unwrap_or(u32::MAX);
        let (achievements, cat_unlocked) = build_category_entry(cat, &cat_defs, &lookups);

        categories.push(AchievementCategoryView {
            category: cat,
            icon: category_icon(cat),
            achievements,
            unlocked_count: cat_unlocked,
            total_count,
            completion_pct: completion_percentage(cat_unlocked, total_count),
        });
    }

    categories
}
