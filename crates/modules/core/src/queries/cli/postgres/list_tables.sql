SELECT tablename as name
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY tablename
