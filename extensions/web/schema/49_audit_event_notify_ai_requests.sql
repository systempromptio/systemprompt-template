-- Audit-event NOTIFY trigger on `ai_requests`, the gateway's request log.
-- Declared only when the build links the AI member, which owns that table;
-- the governance and plugin-usage triggers and their shared rules are in
-- 14_audit_event_notify.sql.

CREATE OR REPLACE FUNCTION audit_event_notify_ai_requests()
RETURNS TRIGGER AS $$
DECLARE
    sev     TEXT;
    payload TEXT;
BEGIN
    BEGIN
        IF NEW.status NOT IN ('ok', 'success', 'completed', 'pending') THEN
            sev := 'error';
        ELSE
            sev := 'info';
        END IF;

        payload := json_build_object(
            'table',      'ai_requests',
            'id',         NEW.id,
            'session_id', NEW.session_id,
            'trace_id',   NEW.trace_id,
            'context_id', NEW.context_id,
            'user_id',    NEW.user_id,
            'model',      NEW.model,
            'status',     NEW.status,
            'severity',   sev,
            'created_at', NEW.created_at
        )::text;

        IF length(payload) > 7800 THEN
            RAISE WARNING 'audit_event_notify_ai_requests: payload truncated (% bytes)', length(payload);
            payload := json_build_object(
                'table',     'ai_requests',
                'id',        NEW.id,
                'truncated', true
            )::text;
        END IF;

        PERFORM pg_notify('audit_events', payload);
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'audit_event_notify_ai_requests failed: % (id=%, session=%)',
            SQLERRM, NEW.id, NEW.session_id;
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER audit_event_notify_ai_requests_trg
    AFTER INSERT ON ai_requests
    FOR EACH ROW
    EXECUTE FUNCTION audit_event_notify_ai_requests();
