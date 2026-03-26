ALTER TABLE user_skills ADD COLUMN IF NOT EXISTS base_skill_id TEXT;
CREATE INDEX IF NOT EXISTS idx_user_skills_base ON user_skills(base_skill_id);
