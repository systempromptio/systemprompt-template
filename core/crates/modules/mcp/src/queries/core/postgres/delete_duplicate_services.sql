DELETE FROM services
WHERE module_name = 'mcp' AND id NOT IN (
    SELECT MAX(id) FROM services
    WHERE module_name = 'mcp'
    GROUP BY name
)