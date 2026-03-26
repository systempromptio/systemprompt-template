-- User activity table for rich dashboard timeline
-- Tracks business-level events: logins, marketplace connects, edits, skill usage

CREATE TABLE IF NOT EXISTS user_activity (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    entity_name TEXT,
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_activity_user ON user_activity(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_activity_category ON user_activity(category, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_activity_created ON user_activity(created_at DESC);

-- Backfill from marketplace_changelog
INSERT INTO user_activity (user_id, category, action, entity_type, entity_id, entity_name, description, metadata, created_at)
SELECT
    mc.user_id,
    'marketplace_edit',
    mc.action,
    'skill',
    mc.skill_id,
    mc.skill_name,
    CASE mc.action
        WHEN 'added' THEN 'Added skill ''' || mc.skill_name || ''''
        WHEN 'updated' THEN 'Updated skill ''' || mc.skill_name || ''''
        WHEN 'deleted' THEN 'Deleted skill ''' || mc.skill_name || ''''
        WHEN 'restored' THEN 'Restored skill ''' || mc.skill_name || ''''
        ELSE mc.action || ' skill ''' || mc.skill_name || ''''
    END,
    '{}',
    mc.created_at
FROM marketplace_changelog mc
WHERE EXISTS (SELECT 1 FROM users WHERE id = mc.user_id);

-- Backfill from marketplace_versions
INSERT INTO user_activity (user_id, category, action, entity_type, entity_name, description, metadata, created_at)
SELECT
    mv.user_id,
    'marketplace_connect',
    mv.version_type,
    'marketplace',
    'v' || mv.version_number,
    CASE mv.version_type
        WHEN 'upload' THEN 'Uploaded marketplace v' || mv.version_number
        WHEN 'restore' THEN 'Restored marketplace v' || mv.version_number
        ELSE mv.version_type || ' marketplace v' || mv.version_number
    END,
    jsonb_build_object('version_number', mv.version_number),
    mv.created_at
FROM marketplace_versions mv
WHERE EXISTS (SELECT 1 FROM users WHERE id = mv.user_id);

-- Clean up any backfilled internal tool usage (Bash, Read, Write, etc. are NOT skills)
DELETE FROM user_activity
WHERE category = 'skill_usage'
  AND entity_name IN ('Bash', 'Read', 'Write', 'Edit', 'Glob', 'Grep', 'NotebookEdit',
                       'Task', 'TodoRead', 'TodoWrite', 'WebFetch', 'WebSearch',
                       'AskFollowupQuestion', 'AttemptCompletion', 'MultiEdit');

-- Backfill session starts as login events (one per user per hour)
INSERT INTO user_activity (user_id, category, action, entity_type, description, metadata, created_at)
SELECT DISTINCT ON (p.user_id, date_trunc('hour', p.created_at))
    p.user_id,
    'login',
    'logged_in',
    NULL,
    'Logged in',
    '{}',
    p.created_at
FROM plugin_usage_events p
WHERE p.event_type = 'claude_code_SessionStart'
  AND EXISTS (SELECT 1 FROM users WHERE id = p.user_id)
ORDER BY p.user_id, date_trunc('hour', p.created_at), p.created_at;
