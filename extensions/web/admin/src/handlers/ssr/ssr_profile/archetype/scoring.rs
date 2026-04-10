use crate::repositories::profile_reports::UserAggregateMetrics;

pub(super) fn score_speed_builder(apm_r: f64, err_r: f64, user_thr: f64, g_thr: f64) -> f64 {
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

pub(super) fn score_quality_artisan(quality_r: f64, goal_r: f64, err_r: f64) -> f64 {
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

pub(super) fn score_power_user(multi_r: f64, conc: f64, spd: f64, g_sessions: f64) -> f64 {
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

pub(super) fn score_methodical_planner(goal_r: f64, velocity: f64, feat: f64, design: f64) -> f64 {
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

pub(super) fn score_debugger(bugfix: f64, user_err: f64, g_err: f64) -> f64 {
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

pub(super) fn score_tool_explorer(td_r: f64, user: &UserAggregateMetrics) -> f64 {
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

pub(super) fn score_efficiency_expert(
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

pub(super) fn score_steady_performer(apm_r: f64, q_r: f64, td_r: f64, m_r: f64, g_r: f64) -> f64 {
    let c = |r: f64| match () {
        () if (r - 1.0).abs() < 0.1 => 15.0,
        () if (r - 1.0).abs() < 0.2 => 10.0,
        () if (r - 1.0).abs() < 0.3 => 5.0,
        () => 0.0,
    };
    c(apm_r) + c(q_r) + c(td_r) + c(m_r) + c(g_r)
}
