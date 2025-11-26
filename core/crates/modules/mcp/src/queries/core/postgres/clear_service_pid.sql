UPDATE services
SET pid = NULL
WHERE name = $1 AND protocol = 'mcp'