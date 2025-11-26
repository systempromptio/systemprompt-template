SELECT
    (SELECT COUNT(DISTINCT user_id)
     FROM user_sessions
     WHERE last_activity_at >= datetime('now', '-1 day')) as dau,
    (SELECT COUNT(DISTINCT user_id)
     FROM user_sessions
     WHERE last_activity_at >= datetime('now', '-7 days')) as wau,
    (SELECT COUNT(DISTINCT user_id)
     FROM user_sessions
     WHERE last_activity_at >= datetime('now', '-30 days')) as mau,
    (SELECT COUNT(*)
     FROM users
     WHERE created_at >= datetime('now', '-7 days')) as new_users_7d,
    (SELECT COUNT(*)
     FROM users
     WHERE created_at >= datetime('now', '-30 days')) as new_users_30d,
    0.0 as growth_rate;
