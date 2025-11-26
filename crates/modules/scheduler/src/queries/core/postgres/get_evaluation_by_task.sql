SELECT
    id, context_id,
    agent_goal, goal_achieved, goal_achievement_confidence, goal_achievement_notes,
    primary_category, topics_discussed, keywords,
    user_satisfied, conversation_quality, quality_notes, issues_encountered,
    agent_name, total_turns, conversation_duration_seconds, user_initiated, completion_status,
    overall_score, evaluation_summary, analyzed_at, analysis_version
FROM conversation_evaluations
WHERE context_id = $1
