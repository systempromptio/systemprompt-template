-- Audit-event NOTIFY triggers
--
-- Fires a `pg_notify('audit_events', <json>)` whenever a row is inserted into
-- `governance_decisions` or `ai_requests`. The Rust admin extension subscribes
-- via PgListener and fans the payload out to SSE clients on
-- `/admin/api/sse/audit`. CREATE OR REPLACE TRIGGER (PG14+) keeps the file
-- declarative and idempotent across re-runs.

CREATE OR REPLACE FUNCTION audit_event_notify_governance()
RETURNS TRIGGER AS $$
DECLARE
    sev          TEXT;
    u_display    TEXT;
    u_department TEXT;
BEGIN
    IF NEW.decision = 'deny' AND NEW.policy IN ('secret_scan', 'secret_injection') THEN
        sev := 'breach';
    ELSIF NEW.decision = 'deny' THEN
        sev := 'deny';
    ELSE
        sev := 'info';
    END IF;

    SELECT u.display_name, upe.department INTO u_display, u_department
    FROM users u
    LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
    WHERE u.id = NEW.user_id;

    PERFORM pg_notify(
        'audit_events',
        json_build_object(
            'table',        'governance_decisions',
            'id',           NEW.id,
            'session_id',   NEW.session_id,
            'user_id',      NEW.user_id,
            'display_name', u_display,
            'department',   u_department,
            'tool_name',    NEW.tool_name,
            'policy',       NEW.policy,
            'decision',     NEW.decision,
            'reason',       NEW.reason,
            'severity',     sev,
            'created_at',   NEW.created_at
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER audit_event_notify_governance_trg
    AFTER INSERT ON governance_decisions
    FOR EACH ROW
    EXECUTE FUNCTION audit_event_notify_governance();


CREATE OR REPLACE FUNCTION audit_event_notify_ai_requests()
RETURNS TRIGGER AS $$
DECLARE
    sev          TEXT;
    u_display    TEXT;
    u_department TEXT;
BEGIN
    IF NEW.status NOT IN ('ok', 'success', 'completed', 'pending') THEN
        sev := 'error';
    ELSE
        sev := 'info';
    END IF;

    SELECT u.display_name, upe.department INTO u_display, u_department
    FROM users u
    LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
    WHERE u.id = NEW.user_id;

    PERFORM pg_notify(
        'audit_events',
        json_build_object(
            'table',        'ai_requests',
            'id',           NEW.id,
            'session_id',   NEW.session_id,
            'trace_id',     NEW.trace_id,
            'context_id',   NEW.context_id,
            'user_id',      NEW.user_id,
            'tenant_id',    NEW.tenant_id,
            'display_name', u_display,
            'department',   u_department,
            'model',          NEW.model,
            'status',         NEW.status,
            'severity',       sev,
            'error_message',  NEW.error_message,
            'cost_display',   '$' || to_char(NEW.cost_microdollars::numeric / 1000000, 'FM999990.0000'),
            'created_at',     NEW.created_at
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER audit_event_notify_ai_requests_trg
    AFTER INSERT ON ai_requests
    FOR EACH ROW
    EXECUTE FUNCTION audit_event_notify_ai_requests();


-- MCP tool execution events (plugin_usage_events). Fans tool-call activity onto
-- the same `audit_events` channel that AI requests and governance decisions use,
-- so the Live Overview > Services & tools pane can stream them without a
-- separate bus or polling fallback.
CREATE OR REPLACE FUNCTION audit_event_notify_plugin_usage()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'audit_events',
        json_build_object(
            'table',       'plugin_usage_events',
            'id',          NEW.id,
            'session_id',  NEW.session_id,
            'user_id',     NEW.user_id,
            'event_type',  NEW.event_type,
            'tool_name',   NEW.tool_name,
            'severity',    'info',
            'created_at',  NEW.created_at
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER audit_event_notify_plugin_usage_trg
    AFTER INSERT ON plugin_usage_events
    FOR EACH ROW
    EXECUTE FUNCTION audit_event_notify_plugin_usage();
