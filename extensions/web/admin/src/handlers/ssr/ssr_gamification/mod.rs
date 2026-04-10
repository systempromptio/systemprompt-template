mod data;

use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{AchievementCategoryView, AchievementView, AchievementsPageData, RankView};
use data::{build_leaderboard_data, category_icon, rarity_label, CATEGORY_ORDER};

struct AchievementMaps<'a> {
    unlocked_ids: std::collections::HashSet<&'a str>,
    unlocked_at: std::collections::HashMap<&'a str, &'a chrono::DateTime<chrono::Utc>>,
    stats: std::collections::HashMap<&'a str, &'a crate::types::AchievementInfo>,
}

fn build_achievement_maps<'a>(
    profile: Option<&'a crate::types::UserGamificationProfile>,
    stats: &'a [crate::types::AchievementInfo],
) -> AchievementMaps<'a> {
    let unlocked_ids = profile.map_or_else(std::collections::HashSet::new, |g| {
        g.achievements
            .iter()
            .map(|ua| ua.achievement_id.as_str())
            .collect()
    });

    let unlocked_at = profile.map_or_else(std::collections::HashMap::new, |g| {
        g.achievements
            .iter()
            .map(|ua| (ua.achievement_id.as_str(), &ua.unlocked_at))
            .collect()
    });

    let stats_map = stats.iter().map(|s| (s.id.as_str(), s)).collect();

    AchievementMaps {
        unlocked_ids,
        unlocked_at,
        stats: stats_map,
    }
}

fn build_category_achievements(
    maps: &AchievementMaps<'_>,
    cat_defs: &[&crate::gamification::AchievementDef],
) -> (Vec<AchievementView>, u32) {
    let mut achievements: Vec<AchievementView> = Vec::new();
    let mut cat_unlocked = 0u32;

    for def in cat_defs {
        let is_unlocked = maps.unlocked_ids.contains(def.id);
        if is_unlocked {
            cat_unlocked += 1;
        }
        let stat = maps.stats.get(def.id);
        let pct = stat.map_or(0.0, |s| s.unlock_percentage);

        achievements.push(AchievementView {
            achievement_id: def.id,
            name: def.name,
            description: def.description,
            category: def.category,
            is_unlocked,
            rarity_pct: pct,
            rarity_label: rarity_label(pct),
            total_unlocked: stat.map_or(0, |s| s.total_unlocked),
            unlocked_at: maps.unlocked_at.get(def.id).copied().copied(),
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

fn build_categories(
    maps: &AchievementMaps<'_>,
    defs: &[crate::gamification::AchievementDef],
) -> (Vec<AchievementCategoryView>, u32) {
    let mut categories: Vec<AchievementCategoryView> = Vec::new();
    let mut total_unlocked_count = 0u32;

    for &cat in CATEGORY_ORDER {
        let cat_defs: Vec<_> = defs.iter().filter(|d| d.category == cat).collect();
        if cat_defs.is_empty() {
            continue;
        }

        let cat_total = u32::try_from(cat_defs.len()).unwrap_or(u32::MAX);
        let (achievements, cat_unlocked) = build_category_achievements(maps, &cat_defs);
        total_unlocked_count += cat_unlocked;

        let completion_pct = completion_percentage(cat_unlocked, cat_total);

        categories.push(AchievementCategoryView {
            category: cat,
            icon: category_icon(cat),
            achievements,
            unlocked_count: cat_unlocked,
            total_count: cat_total,
            completion_pct,
        });
    }

    (categories, total_unlocked_count)
}

fn compute_xp_progress(profile: Option<&crate::types::UserGamificationProfile>) -> i64 {
    profile.map_or(0, |p| {
        if p.xp_to_next_rank > 0 {
            let current_rank_xp = crate::gamification::constants::RANKS
                .iter()
                .rfind(|&&(_, _, threshold)| threshold <= p.total_xp)
                .map_or(0, |&(_, _, threshold)| threshold);
            let numerator = p.total_xp - current_rank_xp;
            let range = p.xp_to_next_rank + numerator;
            if range > 0 {
                numerator.saturating_mul(100) / range
            } else {
                100
            }
        } else {
            100
        }
    })
}

const RANK_WINDOW_SIZE: i32 = 4;

fn build_ranks(user_rank_level: i32) -> (Vec<RankView>, bool, bool) {
    let all_ranks = crate::gamification::constants::RANKS;
    let total = i32::try_from(all_ranks.len()).unwrap_or(i32::MAX);
    let start = usize::try_from((user_rank_level - 1 - RANK_WINDOW_SIZE).max(0)).unwrap_or(0);
    let end = usize::try_from((user_rank_level + RANK_WINDOW_SIZE).max(0))
        .unwrap_or(0)
        .min(all_ranks.len());
    let window: Vec<RankView> = all_ranks[start..end]
        .iter()
        .map(|&(level, name, xp_threshold)| RankView {
            level,
            name,
            xp: xp_threshold,
            is_current: level == user_rank_level,
            is_completed: level < user_rank_level,
        })
        .collect();
    let can_scroll_left = start > 0;
    let can_scroll_right = i32::try_from(end).unwrap_or(i32::MAX) < total;
    (window, can_scroll_left, can_scroll_right)
}

async fn load_achievements_data(
    pool: &PgPool,
    user_id: &str,
) -> (
    Vec<crate::types::AchievementInfo>,
    Option<crate::types::UserGamificationProfile>,
) {
    let stats = crate::gamification::queries::get_achievement_stats(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get achievement stats");
            vec![]
        });

    let profile = crate::gamification::queries::find_user_gamification(pool, user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch gamification profile for achievements");
        })
        .ok()
        .flatten();

    (stats, profile)
}

pub async fn achievements_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let (stats, profile) = load_achievements_data(&pool, user_ctx.user_id.as_str()).await;

    let user_rank_level = profile.as_ref().map_or(1, |p| p.rank_level);
    let (ranks, can_scroll_left, can_scroll_right) = build_ranks(user_rank_level);
    let max_rank = i32::try_from(crate::gamification::constants::RANKS.len()).unwrap_or(i32::MAX);
    let xp_progress_pct = compute_xp_progress(profile.as_ref());
    let maps = build_achievement_maps(profile.as_ref(), &stats);
    let defs = crate::gamification::ACHIEVEMENTS;
    let total_count = u32::try_from(defs.len()).unwrap_or(u32::MAX);
    let (categories, total_unlocked_count) = build_categories(&maps, defs);

    let data = AchievementsPageData {
        page: "achievements",
        title: "Achievements",
        achievements_by_category: categories,
        profile,
        ranks,
        can_scroll_left,
        can_scroll_right,
        current_rank_level: user_rank_level,
        max_rank,
        xp_progress_pct,
        unlocked_count: total_unlocked_count,
        total_count,
    };

    let mut value = serde_json::to_value(&data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize achievements page data");
        serde_json::Value::Null
    });
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "page_stats".to_string(),
            serde_json::json!([
                {"value": format!("{}/{}", total_unlocked_count, total_count), "label": "Achievements"},
            ]),
        );
    }
    super::render_page(&engine, "achievements", &value, &user_ctx, &mkt_ctx)
}

pub async fn leaderboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let sort = params.get("sort").map_or("xp", |s| s.as_str());
    let sort = match sort {
        "sessions" | "prompts" | "tools" | "subagents" | "streak" | "achievements" => sort,
        _ => "xp",
    };

    let (entries, averages) = tokio::join!(
        crate::gamification::queries::get_leaderboard(&pool, 50, 0, None),
        crate::gamification::queries::get_leaderboard_averages(&pool),
    );
    let entries = entries.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch leaderboard entries");
        vec![]
    });
    let averages = averages
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to fetch leaderboard averages");
        })
        .ok();

    let current_user_id = user_ctx.user_id.to_string();
    let data = build_leaderboard_data(&entries, averages.as_ref(), &current_user_id, sort);
    let value = serde_json::to_value(&data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize leaderboard data");
        serde_json::Value::Null
    });
    super::render_page(&engine, "leaderboard", &value, &user_ctx, &mkt_ctx)
}
