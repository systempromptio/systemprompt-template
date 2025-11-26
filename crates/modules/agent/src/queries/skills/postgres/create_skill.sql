INSERT INTO agent_skills (
    skill_id, file_path, name, description, instructions,
    enabled, allowed_tools, tags, category_id, source_id,
    created_at, updated_at
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
    CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT (skill_id) DO NOTHING;
