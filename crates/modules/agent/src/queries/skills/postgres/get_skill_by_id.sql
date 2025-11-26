SELECT skill_id, file_path, name, description, instructions,
       enabled, allowed_tools, tags, category_id, source_id,
       created_at, updated_at
FROM agent_skills
WHERE skill_id = $1
LIMIT 1;
