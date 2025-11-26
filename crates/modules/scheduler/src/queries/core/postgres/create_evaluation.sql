INSERT INTO conversation_evaluations (
    context_id,
    agent_goal, goal_achieved, goal_achievement_confidence, goal_achievement_notes,
    primary_category, topics_discussed, keywords,
    user_satisfied, conversation_quality, quality_notes, issues_encountered,
    agent_name, total_turns, conversation_duration_seconds, user_initiated, completion_status,
    overall_score, evaluation_summary
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
    $11, $12, $13, $14, $15, $16, $17, $18, $19
)
