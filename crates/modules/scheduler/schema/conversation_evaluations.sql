-- Conversation Evaluations Table (Simplified - Focus on Goals & Empirical Data)
-- PostgreSQL version
--
-- IMPORTANT: Evaluations are keyed by context_id (not task_id)
-- A context represents one complete conversation with potentially multiple tasks/turns
-- One evaluation per context, aggregating all messages and metrics from all tasks in that context

CREATE TABLE IF NOT EXISTS conversation_evaluations (
    id SERIAL PRIMARY KEY,
    context_id VARCHAR(255) NOT NULL UNIQUE,

    -- === GOAL ACHIEVEMENT ===
    agent_goal TEXT NOT NULL,
    goal_achieved VARCHAR(50) NOT NULL CHECK(goal_achieved IN ('yes', 'no', 'partial')),
    goal_achievement_confidence REAL NOT NULL CHECK(goal_achievement_confidence BETWEEN 0 AND 1),
    goal_achievement_notes TEXT,

    -- === CONVERSATION CATEGORIZATION ===
    primary_category VARCHAR(100) NOT NULL,
    topics_discussed TEXT NOT NULL,
    keywords TEXT NOT NULL,

    -- === CONVERSATION QUALITY ===
    user_satisfied INTEGER NOT NULL CHECK(user_satisfied BETWEEN 0 AND 100),
    conversation_quality INTEGER NOT NULL CHECK(conversation_quality BETWEEN 0 AND 100),
    quality_notes TEXT,
    issues_encountered TEXT,

    -- === EMPIRICAL DATA ===
    agent_name VARCHAR(255) NOT NULL,
    total_turns INTEGER NOT NULL,
    conversation_duration_seconds INTEGER NOT NULL,
    user_initiated BOOLEAN NOT NULL DEFAULT true,
    completion_status VARCHAR(50) NOT NULL CHECK(completion_status IN ('completed', 'abandoned', 'error')),

    -- === META ===
    overall_score REAL NOT NULL CHECK(overall_score BETWEEN 0 AND 1),
    evaluation_summary TEXT NOT NULL,
    analyzed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    analysis_version VARCHAR(10) DEFAULT 'v4'
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_evals_analyzed_at ON conversation_evaluations(analyzed_at);
CREATE INDEX IF NOT EXISTS idx_evals_goal_achieved ON conversation_evaluations(goal_achieved);
CREATE INDEX IF NOT EXISTS idx_evals_category ON conversation_evaluations(primary_category);
CREATE INDEX IF NOT EXISTS idx_evals_agent ON conversation_evaluations(agent_name);
CREATE INDEX IF NOT EXISTS idx_evals_quality ON conversation_evaluations(conversation_quality);

-- Simple metrics view
CREATE OR REPLACE VIEW evaluation_metrics_daily AS
SELECT
    DATE(analyzed_at) as date,
    COUNT(*) as total_conversations,
    SUM(CASE WHEN goal_achieved = 'yes' THEN 1 ELSE 0 END) * 100.0 / NULLIF(COUNT(*), 0) as goal_success_rate,
    AVG(user_satisfied) as avg_user_satisfaction,
    AVG(overall_score) as avg_overall_score,
    AVG(total_turns) as avg_turns,
    AVG(conversation_duration_seconds) as avg_duration_seconds,
    SUM(CASE WHEN completion_status = 'completed' THEN 1 ELSE 0 END) * 100.0 / NULLIF(COUNT(*), 0) as completion_rate
FROM conversation_evaluations
GROUP BY DATE(analyzed_at)
ORDER BY date DESC;
