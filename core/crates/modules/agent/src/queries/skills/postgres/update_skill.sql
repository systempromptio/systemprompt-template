UPDATE agent_skills
SET
    name = $2,
    description = $3,
    instructions = $4,
    enabled = $5,
    allowed_tools = $6,
    tags = $7,
    updated_at = CURRENT_TIMESTAMP
WHERE skill_id = $1;
