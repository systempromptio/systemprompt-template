-- Rows in every hot table of a restored release schema, so the upgrade runs
-- its backfills against data and fires whatever row triggers that release
-- left live. A schema-only rung never did: 0.58 -> 0.60 passed this ladder
-- and failed on the first client database (ai 032 timeout, web 093/102 on a
-- trigger writing a table core had dropped).
--
-- Rung-tolerant by construction: columns are read from the catalog, so the
-- same seed fits every release. Every text column of row g is 'seed-g', so a
-- request, its messages, its tool calls and the hook events that name them
-- share keys and the backfills that join them actually touch rows. Seeding
-- runs with session_replication_role = replica (no triggers, no FK checks):
-- the rows only have to exist; the upgrade is what must cope with them.
DO $$
DECLARE
    t text;
    cols text;
    vals text;
BEGIN
    PERFORM set_config('session_replication_role', 'replica', true);
    FOREACH t IN ARRAY ARRAY[
        'users', 'user_sessions', 'user_contexts', 'ai_requests', 'ai_request_messages',
        'ai_request_tool_calls', 'mcp_tool_executions', 'plugin_usage_events'
    ] LOOP
        CONTINUE WHEN to_regclass('public.' || t) IS NULL;
        SELECT string_agg(quote_ident(column_name), ', ' ORDER BY ordinal_position),
               string_agg(expr, ', ' ORDER BY ordinal_position)
          INTO cols, vals
          FROM (
            SELECT c.column_name, c.ordinal_position,
                   CASE
                       -- a text column a CHECK limits to a list takes its first value
                       WHEN c.data_type IN ('text', 'character varying', 'character') AND allowed.v IS NOT NULL
                           THEN quote_literal(allowed.v)
                       WHEN c.data_type IN ('text', 'character varying', 'character') THEN '''seed-'' || g'
                       WHEN c.data_type IN ('integer', 'bigint', 'smallint', 'numeric', 'double precision', 'real') THEN 'g'
                       WHEN c.data_type = 'boolean' THEN 'false'
                       WHEN c.data_type LIKE 'timestamp%' THEN 'now() - make_interval(secs => g)'
                       WHEN c.data_type = 'date' THEN 'current_date'
                       WHEN c.data_type IN ('json', 'jsonb') THEN '''{}'''
                       WHEN c.data_type = 'uuid' THEN 'gen_random_uuid()'
                       WHEN c.data_type = 'ARRAY' THEN '''{}'''
                       WHEN c.data_type = 'USER-DEFINED' THEN format('(enum_range(NULL::%I))[1]', c.udt_name)
                   END AS expr
              FROM information_schema.columns c
              LEFT JOIN LATERAL (
                SELECT substring(pg_get_constraintdef(k.oid) FROM '''([^'']*)''') AS v
                  FROM pg_constraint k
                  JOIN pg_attribute a ON a.attrelid = k.conrelid AND a.attnum = k.conkey[1]
                 WHERE k.conrelid = ('public.' || t)::regclass
                   AND k.contype = 'c'
                   AND cardinality(k.conkey) = 1
                   AND a.attname = c.column_name
                 LIMIT 1
              ) allowed ON true
             WHERE c.table_schema = 'public'
               AND c.table_name = t
               AND c.is_generated = 'NEVER'
               AND c.identity_generation IS NULL
               AND (
                   (c.is_nullable = 'NO' AND c.column_default IS NULL)
                   -- the join keys the upgrade's backfills follow
                   OR (t, c.column_name) IN (
                       ('ai_requests', 'session_id'), ('ai_requests', 'user_id'),
                       ('ai_requests', 'provider'), ('ai_requests', 'model'),
                       ('plugin_usage_events', 'tool_use_id'), ('plugin_usage_events', 'session_id'),
                       ('mcp_tool_executions', 'ai_tool_call_id')
                   )
               )
          ) x
         WHERE expr IS NOT NULL;
        EXECUTE format(
            'INSERT INTO public.%I (%s) SELECT %s FROM generate_series(1, 2000) AS g ON CONFLICT DO NOTHING',
            t, cols, vals
        );
    END LOOP;
END $$;
