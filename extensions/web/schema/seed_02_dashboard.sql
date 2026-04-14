-- Seed realistic plugin_usage_events for dashboard demonstration
-- Uses existing user IDs from the users table
-- Only inserts if the table is empty (idempotent)

DO $$
DECLARE
    user_ids TEXT[];
    uid TEXT;
    sid TEXT;
    ts TIMESTAMPTZ;
    ev_id TEXT;
    tool TEXT;
    plugin TEXT;
    ev_type TEXT;
    hour_offset INT;
    day_offset INT;
    i INT;
    skills TEXT[] := ARRAY[
        'use_dangerous_secret', 'example_web_search',
        'enterprise_agent_governance', 'enterprise_security_compliance', 'enterprise_mcp_management',
        'systemprompt_admin_logs', 'systemprompt_admin_analytics', 'systemprompt_admin_agent_management',
        'systemprompt_admin_services', 'systemprompt_admin_jobs', 'systemprompt_admin_database',
        'dev_rust_standards', 'dev_architecture_standards', 'dev_frontend_standards',
        'dev_ext_infrastructure', 'dev_ext_providers', 'dev_ext_hooks',
        'code_review', 'compliance_check', 'general_assistance'
    ];
    plugins TEXT[] := ARRAY[
        'enterprise-demo', 'systemprompt-admin', 'systemprompt-dev', 'systemprompt'
    ];
    event_types TEXT[] := ARRAY[
        'claude_code_PostToolUse', 'claude_code_PostToolUse', 'claude_code_PostToolUse',
        'claude_code_PostToolUse', 'claude_code_PostToolUse', 'claude_code_PostToolUse',
        'claude_code_PostToolUse', 'claude_code_PostToolUse',
        'claude_code_SessionStart', 'claude_code_SessionStart',
        'claude_code_error'
    ];
    row_count INT;
BEGIN
    SELECT COUNT(*) INTO row_count FROM plugin_usage_events;
    IF row_count > 0 THEN
        RAISE NOTICE 'plugin_usage_events already has data, skipping seed';
        RETURN;
    END IF;

    SELECT ARRAY_AGG(id) INTO user_ids FROM users WHERE department != '' AND department IS NOT NULL;

    IF user_ids IS NULL OR array_length(user_ids, 1) IS NULL THEN
        RAISE NOTICE 'No users with departments found, skipping seed';
        RETURN;
    END IF;

    FOR i IN 1..160 LOOP
        uid := user_ids[1 + floor(random() * array_length(user_ids, 1))::INT];
        tool := skills[1 + floor(random() * array_length(skills, 1))::INT];
        plugin := plugins[1 + floor(random() * array_length(plugins, 1))::INT];
        ev_type := event_types[1 + floor(random() * array_length(event_types, 1))::INT];
        ev_id := gen_random_uuid()::TEXT;
        sid := gen_random_uuid()::TEXT;

        -- Spread across last 7 days, weighted toward business hours
        day_offset := floor(random() * 7)::INT;
        IF random() < 0.75 THEN
            hour_offset := 9 + floor(random() * 9)::INT;  -- 9-17 business hours
        ELSE
            hour_offset := floor(random() * 24)::INT;      -- any hour
        END IF;

        ts := NOW() - (day_offset || ' days')::INTERVAL
             - (24 - hour_offset || ' hours')::INTERVAL
             + (floor(random() * 59) || ' minutes')::INTERVAL;

        -- For SessionStart events, clear tool_name
        IF ev_type = 'claude_code_SessionStart' THEN
            INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, plugin_id, metadata, created_at)
            VALUES (ev_id, uid, sid, ev_type, NULL, plugin,
                    jsonb_build_object('source', plugin || '-plugin'),
                    ts);
        ELSIF ev_type = 'claude_code_error' THEN
            INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, plugin_id, metadata, created_at)
            VALUES (ev_id, uid, sid, ev_type, tool, plugin,
                    jsonb_build_object('tool', tool, 'source', plugin || '-plugin', 'error', 'timeout'),
                    ts);
        ELSE
            INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, plugin_id, metadata, created_at)
            VALUES (ev_id, uid, sid, ev_type, tool, plugin,
                    jsonb_build_object('tool', tool, 'source', plugin || '-plugin'),
                    ts);
        END IF;
    END LOOP;

    GET DIAGNOSTICS row_count = ROW_COUNT;
    RAISE NOTICE 'Seeded % plugin_usage_events rows', 160;
END $$;
