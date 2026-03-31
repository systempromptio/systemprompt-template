use super::super::types::{LeaderboardAveragesView, LeaderboardEntryView, LeaderboardPageData};

pub(super) const CATEGORY_ORDER: &[&str] = &[
    "Sessions",
    "Prompts",
    "Tool Mastery",
    "Subagents",
    "Content Volume",
    "Performance",
    "Skills",
    "Agents",
    "Plugins",
    "MCP Servers",
    "Files",
    "Resilience",
    "Streaks",
    "Best Day",
    "Engagement",
    "Time Patterns",
    "Milestones",
];

pub(super) fn category_icon(category: &str) -> &'static str {
    match category {
        "Sessions" => "&#x1F5A5;",
        "Prompts" => "&#x1F4AC;",
        "Tool Mastery" => "&#x1F527;",
        "Subagents" => "&#x1F465;",
        "Content Volume" => "&#x1F4BE;",
        "Performance" => "&#x26A1;",
        "Agents" => "&#x1F916;",
        "Plugins" => "&#x1F9E9;",
        "MCP Servers" => "&#x1F517;",
        "Files" => "&#x1F4C1;",
        "Resilience" => "&#x1F6E1;",
        "Streaks" => "&#x1F525;",
        "Best Day" => "&#x1F3C6;",
        "Engagement" => "&#x1F44D;",
        "Time Patterns" => "&#x1F552;",
        "Milestones" => "&#x1F3C5;",
        _ => "&#x2B50;",
    }
}

pub(super) fn rarity_label(pct: f64) -> &'static str {
    if pct > 50.0 {
        "common"
    } else if pct > 25.0 {
        "uncommon"
    } else if pct > 10.0 {
        "rare"
    } else if pct > 3.0 {
        "epic"
    } else {
        "legendary"
    }
}

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        let whole = n / 1_000_000;
        let frac = (n % 1_000_000) / 100_000;
        format!("{whole}.{frac}M")
    } else if n >= 10_000 {
        let whole = n / 1_000;
        let frac = (n % 1_000) / 100;
        format!("{whole}.{frac}K")
    } else {
        n.to_string()
    }
}

fn build_entry_view(
    i: usize,
    e: &crate::admin::types::LeaderboardEntry,
    current_user_id: &str,
) -> LeaderboardEntryView {
    LeaderboardEntryView {
        position: i + 1,
        display_name: e.display_name.as_deref().unwrap_or("Anonymous").to_string(),
        rank_level: e.rank_level,
        rank_name: e.rank_name.clone(),
        total_xp: e.total_xp,
        events_count: e.events_count,
        current_streak: e.current_streak,
        longest_streak: e.longest_streak,
        achievement_count: e.achievement_count,
        total_sessions: e.total_sessions,
        total_prompts: format_number(e.total_prompts),
        total_tool_uses: format_number(e.total_tool_uses),
        total_subagents: e.total_subagents,
        unique_skills_count: e.unique_skills_count,
        total_days_active: e.total_days_active,
        is_self: e.user_id.as_str() == current_user_id,
        medal: None,
    }
}

fn build_podium(
    entries: &[crate::admin::types::LeaderboardEntry],
    current_user_id: &str,
) -> Vec<LeaderboardEntryView> {
    let medals = ["gold", "silver", "bronze"];
    entries
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, e)| {
            let mut view = build_entry_view(i, e, current_user_id);
            view.medal = Some(medals[i]);
            view
        })
        .collect()
}

fn build_averages_view(
    averages: Option<&crate::admin::gamification::queries::LeaderboardAverages>,
) -> Option<LeaderboardAveragesView> {
    averages.map(|a| LeaderboardAveragesView {
        avg_xp: format!("{:.0}", a.avg_xp),
        avg_sessions: format!("{:.0}", a.avg_sessions),
        avg_prompts: format!("{:.0}", a.avg_prompts),
        avg_tool_uses: format!("{:.0}", a.avg_tool_uses),
        avg_subagents: format!("{:.0}", a.avg_subagents),
        avg_streak: format!("{:.0}", a.avg_streak),
        avg_achievements: format!("{:.0}", a.avg_achievements),
        avg_days_active: format!("{:.0}", a.avg_days_active),
        total_users: a.total_users,
    })
}

pub(super) fn build_leaderboard_data(
    entries: &[crate::admin::types::LeaderboardEntry],
    averages: Option<&crate::admin::gamification::queries::LeaderboardAverages>,
    current_user_id: &str,
    sort: &str,
) -> LeaderboardPageData {
    let podium = build_podium(entries, current_user_id);
    let enriched: Vec<LeaderboardEntryView> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| build_entry_view(i, e, current_user_id))
        .collect();

    LeaderboardPageData {
        page: "leaderboard",
        title: "Leaderboard",
        entries: enriched,
        podium,
        current_sort: sort.to_string(),
        averages: build_averages_view(averages),
    }
}
