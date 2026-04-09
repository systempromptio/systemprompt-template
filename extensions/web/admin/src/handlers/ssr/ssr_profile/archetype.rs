use crate::repositories::daily_summaries::GlobalAverages;
use crate::repositories::profile_reports::UserAggregateMetrics;

#[derive(Debug)]
pub struct ArchetypeResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub confidence: u8,
}

pub fn classify_archetype(user: &UserAggregateMetrics, global: &GlobalAverages) -> ArchetypeResult {
    if user.total_days == 0 {
        return ArchetypeResult {
            id: "newcomer".into(),
            name: "Newcomer".into(),
            description: "Just getting started with Claude Code. Keep building sessions to unlock your profile archetype.".into(),
            confidence: 100,
        };
    }

    let g_apm = f64::from(global.avg_apm.unwrap_or(0.0));
    let g_quality = f64::from(global.avg_quality.unwrap_or(0.0));
    let g_tool_div = f64::from(global.avg_tool_diversity.unwrap_or(0.0));
    let g_multitask = f64::from(global.avg_multitasking.unwrap_or(0.0));
    let g_goal_rate = global.avg_goal_rate.unwrap_or(0.0);
    let g_error_rate = global.avg_error_rate.unwrap_or(0.0);
    let g_throughput = global
        .avg_throughput
        .map_or(0.0, |v| f64::from(u32::try_from(v).unwrap_or(0)));
    let g_sessions = f64::from(global.avg_sessions.unwrap_or(0.0));

    let ratio = |user_val: f64, global_val: f64| -> f64 {
        if global_val > 0.0 {
            user_val / global_val
        } else if user_val > 0.0 {
            1.5
        } else {
            1.0
        }
    };

    let apm_ratio = ratio(user.avg_apm, g_apm);
    let quality_ratio = ratio(user.avg_quality, g_quality);
    let tool_div_ratio = ratio(user.avg_tool_diversity, g_tool_div);
    let multitask_ratio = ratio(user.avg_multitasking, g_multitask);
    let goal_ratio = ratio(user.avg_goal_rate, g_goal_rate);
    let error_ratio = ratio(user.avg_error_rate, g_error_rate);
    let bugfix_pct = category_pct(user, "bugfix");
    let feature_pct = category_pct(user, "feature");
    let design_pct = category_pct(user, "design");

    let mut candidates = vec![
        ("speed_builder", "Speed Builder",
         "High-velocity developer who moves fast with Claude Code. Above-average APM and throughput, getting things done quickly while keeping errors in check.",
         score_speed_builder(apm_ratio, error_ratio, user.avg_throughput, g_throughput)),
        ("quality_artisan", "Quality Artisan",
         "Precision-focused developer who prioritises quality over speed. Consistently high quality scores and goal achievement, with careful attention to outcomes.",
         score_quality_artisan(quality_ratio, goal_ratio, error_ratio)),
        ("power_user", "Power User",
         "Claude Code power user running parallel sessions and multitasking heavily. High concurrency and session counts show deep integration with AI-assisted workflows.",
         score_power_user(multitask_ratio, user.avg_concurrency, user.avg_sessions_per_day, g_sessions)),
        ("methodical_planner", "Methodical Planner",
         "Strategic thinker who plans before acting. High goal achievement with deliberate pacing, often working on features and design tasks with careful structure.",
         score_methodical_planner(goal_ratio, user.avg_session_velocity, feature_pct, design_pct)),
        ("debugger", "Debugger",
         "Problem solver who excels at tracking down and fixing issues. Spends significant time on bug fixes and has developed strong debugging patterns with Claude Code.",
         score_debugger(bugfix_pct, user.avg_error_rate, g_error_rate)),
        ("tool_explorer", "Tool Explorer",
         "Curious experimentalist who leverages a wide variety of tools and skills. Above-average tool diversity shows a willingness to find the right tool for each job.",
         score_tool_explorer(tool_div_ratio, user)),
        ("efficiency_expert", "Efficiency Expert",
         "Maximum output with minimum waste. High effective APM, low error rates, and strong goal completion per prompt show a refined and efficient workflow.",
         score_efficiency_expert(apm_ratio, error_ratio, goal_ratio, user)),
        ("steady_performer", "Steady Performer",
         "Reliable and consistent Claude Code user. Metrics track close to platform averages with a steady rhythm, showing dependable and sustainable usage patterns.",
         score_steady_performer(apm_ratio, quality_ratio, tool_div_ratio, multitask_ratio, goal_ratio)),
    ];

    candidates.sort_unstable_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    let (id, name, description, score) = candidates.remove(0);

    ArchetypeResult {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        confidence: f64_to_u8(score),
    }
}

fn category_pct(user: &UserAggregateMetrics, category: &str) -> f64 {
    let total: i64 = user.category_distribution.values().sum();
    if total == 0 {
        return 0.0;
    }
    let count = user
        .category_distribution
        .get(category)
        .copied()
        .unwrap_or(0);
    let count_f64 = f64::from(u32::try_from(count).unwrap_or(0));
    let total_f64 = f64::from(u32::try_from(total).unwrap_or(1));
    count_f64 / total_f64 * 100.0
}

fn score_speed_builder(apm_r: f64, err_r: f64, user_thr: f64, g_thr: f64) -> f64 {
    let mut s = 0.0;
    if apm_r > 1.3 {
        s += 40.0;
    } else if apm_r > 1.15 {
        s += 25.0;
    } else if apm_r > 1.0 {
        s += 10.0;
    }
    if err_r < 0.8 {
        s += 25.0;
    } else if err_r < 1.0 {
        s += 15.0;
    }
    if g_thr > 0.0 && user_thr / g_thr > 1.2 {
        s += 20.0;
    }
    s
}

fn score_quality_artisan(quality_r: f64, goal_r: f64, err_r: f64) -> f64 {
    let mut s = 0.0;
    if quality_r > 1.2 {
        s += 35.0;
    } else if quality_r > 1.1 {
        s += 20.0;
    }
    if goal_r > 1.2 {
        s += 30.0;
    } else if goal_r > 1.05 {
        s += 15.0;
    }
    if err_r < 0.7 {
        s += 20.0;
    } else if err_r < 0.9 {
        s += 10.0;
    }
    s
}

fn score_power_user(multi_r: f64, conc: f64, spd: f64, g_sessions: f64) -> f64 {
    let mut s = 0.0;
    if multi_r > 1.3 {
        s += 30.0;
    } else if multi_r > 1.1 {
        s += 15.0;
    }
    if conc > 1.5 {
        s += 25.0;
    } else if conc > 1.0 {
        s += 10.0;
    }
    let sr = if g_sessions > 0.0 {
        spd / g_sessions
    } else {
        1.0
    };
    if sr > 1.5 {
        s += 30.0;
    } else if sr > 1.2 {
        s += 15.0;
    }
    s
}

fn score_methodical_planner(goal_r: f64, velocity: f64, feat: f64, design: f64) -> f64 {
    let mut s = 0.0;
    if goal_r > 1.15 {
        s += 30.0;
    } else if goal_r > 1.0 {
        s += 15.0;
    }
    if velocity > 0.0 && velocity < 0.5 {
        s += 20.0;
    } else if velocity < 1.0 {
        s += 10.0;
    }
    if feat + design > 50.0 {
        s += 25.0;
    } else if feat + design > 30.0 {
        s += 15.0;
    }
    s
}

fn score_debugger(bugfix: f64, user_err: f64, g_err: f64) -> f64 {
    let mut s = 0.0;
    if bugfix > 40.0 {
        s += 45.0;
    } else if bugfix > 25.0 {
        s += 30.0;
    } else if bugfix > 15.0 {
        s += 15.0;
    }
    if g_err > 0.0 && user_err / g_err < 0.9 {
        s += 20.0;
    }
    s
}

fn score_tool_explorer(td_r: f64, user: &UserAggregateMetrics) -> f64 {
    let mut s = 0.0;
    if td_r > 1.4 {
        s += 40.0;
    } else if td_r > 1.2 {
        s += 25.0;
    } else if td_r > 1.05 {
        s += 10.0;
    }
    let cats = user.category_distribution.len();
    if cats >= 6 {
        s += 25.0;
    } else if cats >= 4 {
        s += 15.0;
    }
    s
}

fn score_efficiency_expert(
    apm_r: f64,
    err_r: f64,
    goal_r: f64,
    user: &UserAggregateMetrics,
) -> f64 {
    let mut s = 0.0;
    if apm_r > 1.1 {
        s += 20.0;
    }
    if err_r < 0.7 {
        s += 25.0;
    } else if err_r < 0.9 {
        s += 15.0;
    }
    if goal_r > 1.15 {
        s += 25.0;
    } else if goal_r > 1.0 {
        s += 15.0;
    }
    let total_goals = user.total_goals_achieved + user.total_goals_partial;
    if total_goals > 0 {
        let prompts_f64 = f64::from(u32::try_from(user.total_prompts).unwrap_or(0));
        let goals_f64 = f64::from(u32::try_from(total_goals).unwrap_or(1));
        let ppg = prompts_f64 / goals_f64;
        if ppg < 10.0 {
            s += 20.0;
        } else if ppg < 20.0 {
            s += 10.0;
        }
    }
    s
}

fn score_steady_performer(apm_r: f64, q_r: f64, td_r: f64, m_r: f64, g_r: f64) -> f64 {
    let closeness = |r: f64| -> f64 {
        let d = (r - 1.0).abs();
        if d < 0.1 {
            15.0
        } else if d < 0.2 {
            10.0
        } else if d < 0.3 {
            5.0
        } else {
            0.0
        }
    };
    closeness(apm_r) + closeness(q_r) + closeness(td_r) + closeness(m_r) + closeness(g_r)
}

fn f64_to_u8(v: f64) -> u8 {
    format!("{:.0}", v.clamp(0.0, 100.0))
        .parse::<u8>()
        .unwrap_or(0)
}
