-- systemprompt-systemprompt release-baseline: 0.60.0 (core v0.60.0)
-- Recorded by 'just schema-baseline' from a fresh install; the upgrade test
-- restores it and migrates forward. Re-record after every version bump.
--
-- PostgreSQL database dump
--


-- Dumped from database version 18.6 (Debian 18.6-1.pgdg12+2)
-- Dumped by pg_dump version 18.6 (Debian 18.6-1.pgdg12+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: marketplace; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA marketplace;


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: accept_ingestion_delivery(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.accept_ingestion_delivery() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE old_digest text; digest text;
BEGIN
    digest := NEW.metadata->>'_ingestion_digest';
    IF digest IS NOT NULL THEN
        INSERT INTO ingestion_event_receipts(dedup_key, payload_digest) VALUES(NEW.dedup_key, digest)
        ON CONFLICT(dedup_key) DO NOTHING;
        SELECT payload_digest INTO old_digest FROM ingestion_event_receipts WHERE dedup_key = NEW.dedup_key;
        IF old_digest <> digest THEN
            RAISE EXCEPTION 'Conflicting event delivery' USING ERRCODE = '23505';
        END IF;
    END IF;
    RETURN NEW;
END $$;


--
-- Name: analysis_native_session(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.analysis_native_session(metadata_user_id text) RETURNS text
    LANGUAGE plpgsql IMMUTABLE
    AS $_$
DECLARE session_key text;
BEGIN
    IF left(btrim(metadata_user_id), 1) = '{' THEN
        session_key := (metadata_user_id::jsonb)->>'session_id';
    ELSE
        session_key := substring(metadata_user_id from '_session_([^[:space:]]+)$');
    END IF;
    RETURN (session_key::uuid)::text;
EXCEPTION WHEN invalid_text_representation THEN RETURN NULL;
END $_$;


--
-- Name: assert_ingestion_owner(text, text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.assert_ingestion_owner(session_key text, owner_key text) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE bound_owner text;
BEGIN
    IF session_key IS NULL OR btrim(session_key) = '' OR length(session_key) > 255 THEN
        RAISE EXCEPTION 'Invalid ingestion session' USING ERRCODE = '23514';
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended('ingestion:' || session_key, 0));
    IF NOT EXISTS (SELECT 1 FROM users WHERE id = owner_key) THEN
        RAISE EXCEPTION 'Unknown ingestion owner' USING ERRCODE = '23503';
    END IF;
    IF EXISTS (SELECT 1 FROM plugin_usage_events WHERE session_id = session_key AND user_id <> owner_key)
       OR EXISTS (SELECT 1 FROM plugin_session_summaries WHERE session_id = session_key AND user_id <> owner_key)
       OR EXISTS (SELECT 1 FROM session_cost_snapshots WHERE session_id = session_key AND user_id <> owner_key)
       OR EXISTS (SELECT 1 FROM session_transcripts WHERE session_id = session_key AND user_id <> owner_key) THEN
        RAISE EXCEPTION 'Ingestion session ownership conflict' USING ERRCODE = '23514';
    END IF;
    INSERT INTO ingestion_session_owners(session_id, user_id) VALUES(session_key, owner_key)
    ON CONFLICT (session_id) DO NOTHING;
    SELECT user_id INTO bound_owner FROM ingestion_session_owners WHERE session_id = session_key;
    IF bound_owner <> owner_key THEN
        RAISE EXCEPTION 'Ingestion session ownership conflict' USING ERRCODE = '23514';
    END IF;
END $$;


--
-- Name: attribute_ingested_skill_invocation(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.attribute_ingested_skill_invocation() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    raw_skill text;
    skill_key text;
    receipt record;
    traffic text;
BEGIN
    IF NEW.event_type = 'UserPromptSubmit' AND NEW.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+' THEN
        raw_skill := substring(NEW.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)');
    ELSIF NEW.event_type IN ('PostToolUse','PostToolUseFailure') AND NEW.tool_name = 'Skill' THEN
        raw_skill := NEW.metadata->'tool_input'->>'skill';
    ELSE
        RETURN NEW;
    END IF;
    skill_key := replace(split_part(raw_skill, ':', 2), '-', '_');
    SELECT COALESCE(b.traffic_class,'production') INTO traffic
      FROM (SELECT 1) seed LEFT JOIN eval_session_bindings b
        ON b.owner_id=NEW.user_id AND b.session_id=NEW.session_id;
    SELECT i.id AS receipt_id,i.resource_id,i.generation,p.revision_id
      INTO receipt
      FROM managed_installation_receipts i
      JOIN managed_resources m ON m.id=i.resource_id AND m.owner_id=i.owner_id AND m.kind='skill'
      JOIN managed_publications p ON p.id=i.publication_id AND p.owner_id=i.owner_id
     WHERE i.owner_id=NEW.user_id
       AND i.installation_id=NEW.metadata->>'installation_id'
       AND i.generation::text=NEW.metadata->>'publication_generation'
       AND p.revision_id=NEW.metadata->>'resource_revision_id'
       AND m.resource_key=skill_key
       AND i.client_evidence->>'session_id'=NEW.session_id
     ORDER BY i.verified_at DESC LIMIT 1;
    INSERT INTO managed_invocation_attributions(
        id,owner_id,invocation_id,installation_id,resource_id,revision_id,
        publication_generation,traffic_class,status,receipt_id,authenticated_evidence
    ) VALUES (
        'mia_' || md5(NEW.id || NEW.user_id),NEW.user_id,NEW.id,
        CASE WHEN receipt.receipt_id IS NULL THEN NULL ELSE NEW.metadata->>'installation_id' END,
        receipt.resource_id,receipt.revision_id,receipt.generation,traffic,
        CASE WHEN receipt.receipt_id IS NULL THEN 'revision_unknown' ELSE 'verified' END,
        receipt.receipt_id,
        jsonb_build_object('session_id',NEW.session_id,'dedup_key',NEW.dedup_key,'ingestion_owner_verified',true)
    ) ON CONFLICT(owner_id,invocation_id) DO NOTHING;
    RETURN NEW;
END $$;


--
-- Name: audit_event_notify_ai_requests(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.audit_event_notify_ai_requests() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;


--
-- Name: audit_event_notify_governance(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.audit_event_notify_governance() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    sev     TEXT;
    payload TEXT;
BEGIN
    BEGIN
        IF NEW.decision = 'deny' AND NEW.policy = 'secret_scan' THEN
            sev := 'breach';
        ELSIF NEW.decision = 'deny' THEN
            sev := 'deny';
        ELSE
            sev := 'info';
        END IF;

        payload := json_build_object(
            'table',      'governance_decisions',
            'id',         NEW.id,
            'session_id', NEW.session_id,
            'user_id',    NEW.user_id,
            'tool_name',  NEW.tool_name,
            'policy',     NEW.policy,
            'decision',   NEW.decision,
            'severity',   sev,
            'created_at', NEW.created_at
        )::text;

        IF length(payload) > 7800 THEN
            RAISE WARNING 'audit_event_notify_governance: payload truncated (% bytes)', length(payload);
            payload := json_build_object(
                'table',     'governance_decisions',
                'id',        NEW.id,
                'truncated', true
            )::text;
        END IF;

        PERFORM pg_notify('audit_events', payload);
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'audit_event_notify_governance failed: % (id=%, session=%)',
            SQLERRM, NEW.id, NEW.session_id;
    END;
    RETURN NEW;
END;
$$;


--
-- Name: audit_event_notify_plugin_usage(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.audit_event_notify_plugin_usage() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    payload TEXT;
BEGIN
    BEGIN
        payload := json_build_object(
            'table',       'plugin_usage_events',
            'id',          NEW.id,
            'session_id',  NEW.session_id,
            'user_id',     NEW.user_id,
            'event_type',  NEW.event_type,
            'tool_name',   NEW.tool_name,
            'severity',    'info',
            'created_at',  NEW.created_at
        )::text;

        IF length(payload) > 7800 THEN
            RAISE WARNING 'audit_event_notify_plugin_usage: payload truncated (% bytes)', length(payload);
            payload := json_build_object(
                'table',     'plugin_usage_events',
                'id',        NEW.id,
                'truncated', true
            )::text;
        END IF;

        PERFORM pg_notify('audit_events', payload);
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'audit_event_notify_plugin_usage failed: % (id=%, session=%)',
            SQLERRM, NEW.id, NEW.session_id;
    END;
    RETURN NEW;
END;
$$;


--
-- Name: conversation_metrics_for(text[]); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.conversation_metrics_for(context_ids text[]) RETURNS TABLE(context_id text, user_id text, display_name text, session_id text, client_session_id text, group_name text, project_name text, model text, turn_count bigint, side_call_count bigint, side_call_cost_microdollars bigint, tool_call_count bigint, error_count bigint, total_input_tokens bigint, total_output_tokens bigint, total_cost_microdollars bigint, first_at timestamp with time zone, last_at timestamp with time zone, status text)
    LANGUAGE sql STABLE
    AS $$
WITH scoped AS (
    SELECT ar.id, ar.context_id, ar.gateway_conversation_id, ar.user_id,
           ar.session_id, ar.client_session_id, ar.model, ar.request_kind,
           ar.status, ar.input_tokens, ar.output_tokens, ar.cost_microdollars, ar.created_at,
           COUNT(*) OVER (PARTITION BY ar.context_id, ar.gateway_conversation_id) AS thread_size
    FROM ai_requests ar
    WHERE ar.context_id <> '00000000-0000-0000-0000-4c4547414359'
      AND (context_ids IS NULL OR ar.context_id IN (SELECT unnest(context_ids)))
), requests AS MATERIALIZED (
    SELECT r.*, conversation_request_kind(r.request_kind, p.offered_tools IS NOT NULL, r.thread_size) AS effective_kind
    FROM scoped r LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
), agg AS (
    SELECT r.context_id,
        COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint AS turn_count,
        COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS side_call_count,
        COALESCE(SUM(cost_microdollars) FILTER (WHERE effective_kind <> 'turn'), 0)::bigint AS side_call_cost_microdollars,
        COUNT(*) FILTER (WHERE status = 'failed')::bigint AS error_count,
        COALESCE(SUM(input_tokens), 0)::bigint AS total_input_tokens,
        COALESCE(SUM(output_tokens), 0)::bigint AS total_output_tokens,
        COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
        MIN(created_at) AS first_at, MAX(created_at) AS last_request_at,
        (ARRAY_AGG(user_id ORDER BY created_at DESC, id DESC))[1] AS user_id,
        (ARRAY_AGG(session_id ORDER BY created_at DESC, id DESC)
            FILTER (WHERE session_id IS NOT NULL))[1] AS session_id,
        (ARRAY_AGG(client_session_id ORDER BY created_at DESC, id DESC)
            FILTER (WHERE client_session_id IS NOT NULL))[1] AS client_session_id,
        (ARRAY_AGG(model ORDER BY created_at DESC, id DESC)
            FILTER (WHERE effective_kind = 'turn' AND model IS NOT NULL))[1] AS model
    FROM requests r GROUP BY r.context_id
), tools AS (
    SELECT r.context_id, COUNT(*)::bigint AS tool_call_count
    FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
    WHERE r.effective_kind = 'turn' GROUP BY r.context_id
)
SELECT a.context_id, a.user_id, u.display_name, a.session_id, a.client_session_id,
       g.name, p.name, a.model, a.turn_count, a.side_call_count,
       a.side_call_cost_microdollars, COALESCE(t.tool_call_count, 0), a.error_count,
       a.total_input_tokens, a.total_output_tokens, a.total_cost_microdollars,
       a.first_at, GREATEST(a.last_request_at, c.updated_at), s.status
FROM agg a
LEFT JOIN tools t ON t.context_id = a.context_id
LEFT JOIN user_contexts c ON c.context_id = a.context_id
LEFT JOIN plugin_session_summaries s ON s.session_id = a.client_session_id
LEFT JOIN users u ON u.id = a.user_id
LEFT JOIN user_scope_defaults d ON d.user_id = a.user_id
LEFT JOIN groups g ON g.id = d.primary_group_id
LEFT JOIN projects p ON p.id = d.primary_project_id
$$;


--
-- Name: conversation_opening_prompt(text, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.conversation_opening_prompt(context_key text, max_characters integer) RETURNS text
    LANGUAGE sql STABLE
    AS $$
SELECT LEFT(NULLIF(BTRIM(regexp_replace(regexp_replace(regexp_replace(
    split_part(m.content, '=== ASSISTANT ANSWER ===', 1),
    '=== USER PROMPT ===', '', 'g'), '<system-reminder>.*?</system-reminder>', '', 'g'),
    '\s+', ' ', 'g')), ''), max_characters)
FROM (
    SELECT m.content FROM conversation_requests cr
    JOIN ai_request_messages m ON m.request_id = cr.id
    WHERE cr.context_id = context_key AND cr.effective_kind = 'turn' AND m.role = 'user'
    ORDER BY cr.created_at, m.sequence_number, cr.id LIMIT 1
) m
$$;


--
-- Name: conversation_request_kind(text, boolean, bigint); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.conversation_request_kind(kind text, has_tools boolean, thread_size bigint) RETURNS text
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    AS $$
SELECT CASE WHEN kind = 'probe' THEN 'probe' WHEN kind = 'utility' THEN 'utility'
            WHEN NOT has_tools AND thread_size = 1 THEN 'utility' ELSE 'turn' END
$$;


--
-- Name: conversation_title(text, text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.conversation_title(context_key text, client_session_key text) RETURNS text
    LANGUAGE sql STABLE
    AS $$
SELECT COALESCE(
    (SELECT s.ai_title FROM plugin_session_summaries s WHERE s.session_id = client_session_key),
    (SELECT NULLIF(c.name, 'Gateway conversation') FROM user_contexts c WHERE c.context_id = context_key),
    conversation_opening_prompt(context_key, 160),
    'Conversation ' || LEFT(context_key, 12) || '…')
$$;


--
-- Name: drain_ingestion_outbox(integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.drain_ingestion_outbox(batch_size integer) RETURNS bigint
    LANGUAGE plpgsql
    AS $$
DECLARE event_ids text[]; affected bigint;
BEGIN
    SELECT array_agg(event_id) INTO event_ids FROM (
        SELECT event_id FROM ingestion_outbox WHERE processed_at IS NULL ORDER BY created_at
        LIMIT least(greatest(batch_size, 1), 10000) FOR UPDATE SKIP LOCKED
    ) pending;
    IF event_ids IS NULL THEN RETURN 0; END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended('ingestion:' || session_id, 0))
    FROM (SELECT DISTINCT session_id FROM plugin_usage_events WHERE id = ANY(event_ids) ORDER BY session_id) owners;
    INSERT INTO plugin_session_summaries(id,session_id,user_id,total_events,tool_uses,prompts,errors,
        content_input_bytes,content_output_bytes,loc_added,loc_removed,started_at,subagent_spawns,user_prompts,automated_actions)
    SELECT 'sess_' || e.session_id,e.session_id,e.user_id,count(*),
        count(*) FILTER(WHERE event_type IN ('PostToolUse','PostToolUseFailure')),
        count(*) FILTER(WHERE event_type = 'UserPromptSubmit'),
        count(*) FILTER(WHERE event_type = 'PostToolUseFailure'),
        coalesce(sum(content_input_bytes),0),coalesce(sum(content_output_bytes),0),
        sum(loc_added),sum(loc_removed),min(e.created_at),
        count(*) FILTER(WHERE event_type='SubagentStop'),
        count(*) FILTER(WHERE event_type='UserPromptSubmit' AND coalesce(metadata->>'agent_id','')=''),
        count(*) FILTER(WHERE event_type IN ('PostToolUse','PostToolUseFailure') AND coalesce(metadata->>'agent_id','')<>'')
    FROM plugin_usage_events e WHERE (e.user_id,e.session_id) IN (
        SELECT user_id,session_id FROM plugin_usage_events WHERE id = ANY(event_ids))
    GROUP BY e.user_id,e.session_id
    ON CONFLICT(session_id) DO UPDATE SET total_events=EXCLUDED.total_events,
        tool_uses=EXCLUDED.tool_uses,prompts=EXCLUDED.prompts,errors=EXCLUDED.errors,
        content_input_bytes=EXCLUDED.content_input_bytes,content_output_bytes=EXCLUDED.content_output_bytes,
        loc_added=EXCLUDED.loc_added,loc_removed=EXCLUDED.loc_removed,subagent_spawns=EXCLUDED.subagent_spawns,
        user_prompts=EXCLUDED.user_prompts,automated_actions=EXCLUDED.automated_actions,updated_at=now()
    WHERE plugin_session_summaries.user_id=EXCLUDED.user_id;
    UPDATE ingestion_outbox SET processed_at=now() WHERE event_id=ANY(event_ids);
    GET DIAGNOSTICS affected = ROW_COUNT;
    RETURN affected;
END $$;


--
-- Name: enforce_eval_lifecycle_owner(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.enforce_eval_lifecycle_owner() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE experiment_owner text; related_owner text;
BEGIN
    IF TG_TABLE_NAME = 'eval_execution_approvals' THEN
        SELECT e.owner_id INTO experiment_owner FROM eval_executions x JOIN eval_experiments e ON e.id=x.experiment_id WHERE x.id=NEW.execution_id;
        related_owner := NEW.owner_id;
    ELSIF TG_TABLE_NAME = 'eval_suggestions' THEN
        SELECT owner_id INTO experiment_owner FROM eval_experiments WHERE id=NEW.experiment_id;
        SELECT a.owner_id INTO related_owner FROM eval_budget_reservations r JOIN eval_budget_accounts a ON a.id=r.account_id WHERE r.id=NEW.reservation_id AND a.owner_id=NEW.owner_id;
    ELSE
        SELECT e.owner_id INTO experiment_owner FROM eval_experiments e JOIN eval_resource_revisions c ON c.id=NEW.case_revision_id AND c.owner_id=e.owner_id WHERE e.id=NEW.experiment_id;
        related_owner := NEW.owner_id;
    END IF;
    IF experiment_owner IS NULL OR related_owner IS NULL OR experiment_owner <> related_owner THEN RAISE EXCEPTION 'evaluation lifecycle ownership conflict' USING ERRCODE='23514'; END IF;
    RETURN NEW;
END $$;


--
-- Name: enforce_eval_owner_scope(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.enforce_eval_owner_scope() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE expected_owner text; related_owner text;
BEGIN
    IF TG_TABLE_NAME = 'eval_executions' THEN
        SELECT owner_id INTO expected_owner FROM eval_experiments WHERE id=NEW.experiment_id;
        SELECT owner_id INTO related_owner FROM eval_resource_revisions WHERE id=NEW.case_revision_id;
    ELSIF TG_TABLE_NAME = 'eval_session_bindings' THEN
        SELECT e.owner_id INTO expected_owner FROM eval_executions x JOIN eval_experiments e ON e.id=x.experiment_id WHERE x.id=NEW.execution_id;
        related_owner := NEW.owner_id;
    ELSIF TG_TABLE_NAME = 'eval_execution_capabilities' THEN
        SELECT e.owner_id INTO expected_owner FROM eval_executions x JOIN eval_experiments e ON e.id=x.experiment_id WHERE x.id=NEW.execution_id;
        SELECT w.owner_id INTO related_owner FROM eval_workers w JOIN user_sessions s ON s.user_id=w.owner_id WHERE w.id=NEW.worker_id AND s.session_id=NEW.session_id;
    ELSIF TG_TABLE_NAME = 'eval_request_reservations' THEN
        SELECT e.owner_id INTO expected_owner FROM eval_executions x JOIN eval_experiments e ON e.id=x.experiment_id WHERE x.id=NEW.execution_id;
        SELECT a.owner_id INTO related_owner FROM eval_budget_reservations r JOIN eval_budget_accounts a ON a.id=r.account_id JOIN ai_requests q ON q.user_id=a.owner_id WHERE r.id=NEW.reservation_id AND q.id=NEW.request_id;
    END IF;
    IF expected_owner IS NULL OR related_owner IS NULL OR expected_owner <> related_owner THEN
        RAISE EXCEPTION 'evaluation ownership conflict' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END $$;


--
-- Name: enforce_ingestion_owner(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.enforce_ingestion_owner() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'UPDATE' AND (NEW.user_id <> OLD.user_id OR NEW.session_id <> OLD.session_id) THEN
        RAISE EXCEPTION 'Ingestion ownership is immutable' USING ERRCODE = '23514';
    END IF;
    PERFORM assert_ingestion_owner(NEW.session_id, NEW.user_id);
    RETURN NEW;
END $$;


--
-- Name: enqueue_ingestion_event(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.enqueue_ingestion_event() RETURNS trigger
    LANGUAGE plpgsql
    AS $$ BEGIN
    INSERT INTO ingestion_outbox(event_id) VALUES(NEW.id) ON CONFLICT DO NOTHING;
    RETURN NEW;
END $$;


--
-- Name: governance_decisions_deny_update(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.governance_decisions_deny_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'governance_decisions is append-only: UPDATE is refused'
        USING ERRCODE = 'restrict_violation';
END;
$$;


--
-- Name: groups_protect_system(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.groups_protect_system() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.is_system THEN
        RAISE EXCEPTION 'group % is a system group and cannot be deleted', OLD.id;
    END IF;
    RETURN OLD;
END
$$;


--
-- Name: reject_eval_managed_workspace_change(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.reject_eval_managed_workspace_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN RAISE EXCEPTION 'managed evaluator workspace projections are immutable' USING ERRCODE='23514'; END $$;


--
-- Name: reject_eval_operation_receipt_change(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.reject_eval_operation_receipt_change() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN RAISE EXCEPTION 'approved operation receipts are immutable' USING ERRCODE='23514'; END $$;


--
-- Name: reject_managed_content_update(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.reject_managed_content_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'managed content is immutable' USING ERRCODE = '23514';
END;
$$;


--
-- Name: update_timestamp_trigger(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_timestamp_trigger() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: paddle_customers; Type: TABLE; Schema: marketplace; Owner: -
--

CREATE TABLE marketplace.paddle_customers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id text NOT NULL,
    paddle_customer_id text,
    email text NOT NULL,
    name text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: paddle_webhook_events; Type: TABLE; Schema: marketplace; Owner: -
--

CREATE TABLE marketplace.paddle_webhook_events (
    id bigint NOT NULL,
    event_id text NOT NULL,
    event_type text NOT NULL,
    payload jsonb DEFAULT '{}'::jsonb NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    error_message text,
    processed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: paddle_webhook_events_id_seq; Type: SEQUENCE; Schema: marketplace; Owner: -
--

CREATE SEQUENCE marketplace.paddle_webhook_events_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: paddle_webhook_events_id_seq; Type: SEQUENCE OWNED BY; Schema: marketplace; Owner: -
--

ALTER SEQUENCE marketplace.paddle_webhook_events_id_seq OWNED BY marketplace.paddle_webhook_events.id;


--
-- Name: plans; Type: TABLE; Schema: marketplace; Owner: -
--

CREATE TABLE marketplace.plans (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    display_name text NOT NULL,
    description text,
    paddle_product_id text NOT NULL,
    paddle_price_id text NOT NULL,
    amount_cents integer DEFAULT 0 NOT NULL,
    currency text DEFAULT 'USD'::text NOT NULL,
    billing_interval text DEFAULT 'month'::text NOT NULL,
    features jsonb DEFAULT '{}'::jsonb NOT NULL,
    limits jsonb DEFAULT '{}'::jsonb NOT NULL,
    role_name text,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: subscriptions; Type: TABLE; Schema: marketplace; Owner: -
--

CREATE TABLE marketplace.subscriptions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id text NOT NULL,
    paddle_subscription_id text,
    paddle_customer_id text,
    plan_id uuid,
    status text DEFAULT 'active'::text NOT NULL,
    current_period_start timestamp with time zone,
    current_period_end timestamp with time zone,
    cancel_at timestamp with time zone,
    paddle_data jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: access_control_entities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.access_control_entities (
    entity_type text NOT NULL,
    entity_id text NOT NULL,
    default_included boolean DEFAULT false NOT NULL,
    source text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT access_control_entities_entity_type_check CHECK ((entity_type = ANY (ARRAY['plugin'::text, 'agent'::text, 'mcp_server'::text, 'marketplace'::text, 'gateway_route'::text, 'skill'::text, 'hook'::text, 'slack_workspace'::text, 'teams_tenant'::text])))
);


--
-- Name: access_control_rules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.access_control_rules (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    entity_type text NOT NULL,
    entity_id text NOT NULL,
    rule_type text NOT NULL,
    rule_value text NOT NULL,
    access text DEFAULT 'allow'::text NOT NULL,
    justification text,
    source text DEFAULT 'yaml'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT access_control_rules_access_check CHECK ((access = ANY (ARRAY['allow'::text, 'deny'::text]))),
    CONSTRAINT access_control_rules_entity_type_check CHECK ((entity_type = ANY (ARRAY['plugin'::text, 'agent'::text, 'mcp_server'::text, 'marketplace'::text, 'gateway_route'::text, 'skill'::text, 'hook'::text, 'slack_workspace'::text, 'teams_tenant'::text])))
);


--
-- Name: admin_traffic_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.admin_traffic_reports (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    report_date date NOT NULL,
    report_period text DEFAULT 'am'::text NOT NULL,
    report_data jsonb DEFAULT '{}'::jsonb NOT NULL,
    generated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: admin_usage_daily_rollups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.admin_usage_daily_rollups (
    user_id text NOT NULL,
    date date NOT NULL,
    sessions_count integer DEFAULT 0 NOT NULL,
    prompts bigint DEFAULT 0 NOT NULL,
    tool_uses bigint DEFAULT 0 NOT NULL,
    errors bigint DEFAULT 0 NOT NULL,
    loc_added_ai bigint DEFAULT 0 NOT NULL,
    loc_removed_ai bigint DEFAULT 0 NOT NULL,
    commits_count integer DEFAULT 0 NOT NULL,
    commit_insertions bigint DEFAULT 0 NOT NULL,
    commit_deletions bigint DEFAULT 0 NOT NULL,
    ai_requests_count bigint DEFAULT 0 NOT NULL,
    input_tokens bigint DEFAULT 0 NOT NULL,
    output_tokens bigint DEFAULT 0 NOT NULL,
    cost_microdollars bigint DEFAULT 0 NOT NULL,
    group_id text,
    project_id text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: agent_tasks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.agent_tasks (
    task_id text NOT NULL,
    context_id text NOT NULL,
    status text DEFAULT 'TASK_STATE_SUBMITTED'::text NOT NULL,
    status_timestamp timestamp with time zone,
    user_id text,
    session_id text,
    trace_id text,
    agent_name text,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    execution_time_ms integer,
    error_message text,
    metadata jsonb DEFAULT '{}'::jsonb,
    version bigint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT agent_tasks_status_check CHECK ((status = ANY (ARRAY['TASK_STATE_PENDING'::text, 'TASK_STATE_SUBMITTED'::text, 'TASK_STATE_WORKING'::text, 'TASK_STATE_INPUT_REQUIRED'::text, 'TASK_STATE_COMPLETED'::text, 'TASK_STATE_CANCELED'::text, 'TASK_STATE_FAILED'::text, 'TASK_STATE_REJECTED'::text, 'TASK_STATE_AUTH_REQUIRED'::text, 'TASK_STATE_UNKNOWN'::text])))
);


--
-- Name: ai_gateway_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_gateway_policies (
    id text NOT NULL,
    name character varying(255) NOT NULL,
    spec jsonb NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    priority integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: ai_gateway_thought_signatures; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_gateway_thought_signatures (
    user_id text NOT NULL,
    conversation_id text NOT NULL,
    tool_use_id text NOT NULL,
    signature text NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


--
-- Name: ai_quota_buckets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_quota_buckets (
    id text NOT NULL,
    subject_id character varying(255) NOT NULL,
    subject_kind text DEFAULT 'user'::text NOT NULL,
    window_seconds integer NOT NULL,
    window_start timestamp with time zone NOT NULL,
    requests bigint DEFAULT 0 NOT NULL,
    input_tokens bigint DEFAULT 0 NOT NULL,
    output_tokens bigint DEFAULT 0 NOT NULL,
    cost_microdollars bigint DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: ai_request_messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_request_messages (
    id text DEFAULT gen_random_uuid() NOT NULL,
    request_id character varying(255) NOT NULL,
    role text NOT NULL,
    content text NOT NULL,
    sequence_number integer NOT NULL,
    name character varying(255),
    tool_call_id character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT ai_request_messages_role_check CHECK ((role = ANY (ARRAY['user'::text, 'assistant'::text, 'system'::text, 'tool'::text])))
);


--
-- Name: ai_request_payloads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_request_payloads (
    ai_request_id text NOT NULL,
    request_body jsonb,
    offered_tools jsonb,
    response_body jsonb,
    request_excerpt text,
    response_excerpt text,
    request_truncated boolean DEFAULT false NOT NULL,
    response_truncated boolean DEFAULT false NOT NULL,
    request_bytes integer,
    response_bytes integer,
    request_body_sha256 text,
    prepared_body_sha256 text,
    response_body_sha256 text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: ai_request_tool_calls; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_request_tool_calls (
    id text DEFAULT gen_random_uuid() NOT NULL,
    request_id character varying(255) NOT NULL,
    tool_name character varying(255) NOT NULL,
    tool_input text NOT NULL,
    mcp_execution_id character varying(255),
    ai_tool_call_id character varying(255),
    sequence_number integer NOT NULL,
    tool_result_payload jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: ai_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_requests (
    id text NOT NULL,
    request_id character varying(255) NOT NULL,
    user_id character varying(255) NOT NULL,
    session_id character varying(255),
    task_id text,
    context_id character varying(255) NOT NULL,
    gateway_conversation_id character varying(255),
    client_session_id text,
    provider_request_id character varying(255),
    trace_id character varying(255),
    mcp_execution_id character varying(255),
    provider text,
    model text,
    requested_model text,
    system_prompt_override text,
    route_match text,
    temperature double precision,
    top_p double precision,
    max_tokens integer,
    stop_sequences text,
    tokens_used integer,
    input_tokens integer,
    output_tokens integer,
    cost_microdollars bigint DEFAULT 0 NOT NULL,
    latency_ms integer,
    upstream_latency_ms integer,
    cache_hit boolean DEFAULT false NOT NULL,
    cache_read_tokens integer,
    cache_creation_tokens integer,
    reasoning_tokens integer,
    is_streaming boolean DEFAULT false NOT NULL,
    status character varying(255) DEFAULT 'pending'::character varying NOT NULL,
    error_message text,
    actor_kind text NOT NULL,
    actor_id text NOT NULL,
    synthetic boolean DEFAULT false NOT NULL,
    request_kind text DEFAULT 'turn'::text NOT NULL,
    instance_id character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT ai_requests_actor_id_check CHECK ((length(actor_id) > 0)),
    CONSTRAINT ai_requests_actor_kind_check CHECK ((actor_kind = ANY (ARRAY['user'::text, 'job'::text, 'mcp'::text]))),
    CONSTRAINT ai_requests_request_kind_check CHECK ((request_kind = ANY (ARRAY['turn'::text, 'probe'::text, 'utility'::text]))),
    CONSTRAINT ai_requests_routed_has_provider CHECK ((((status)::text = 'rejected'::text) OR ((provider IS NOT NULL) AND (model IS NOT NULL))))
);


--
-- Name: ai_safety_findings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_safety_findings (
    id text NOT NULL,
    ai_request_id text NOT NULL,
    phase character varying(32) NOT NULL,
    severity character varying(16) NOT NULL,
    category character varying(64) NOT NULL,
    scanner character varying(64) NOT NULL,
    excerpt text,
    blocked boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: plugin_usage_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plugin_usage_events (
    id text NOT NULL,
    user_id text NOT NULL,
    session_id text NOT NULL,
    event_type text NOT NULL,
    tool_name text,
    plugin_id text,
    metadata jsonb DEFAULT '{}'::jsonb,
    dedup_key text,
    prompt_preview text,
    description text,
    cwd text,
    content_input_bytes bigint DEFAULT 0,
    content_output_bytes bigint DEFAULT 0,
    loc_added bigint DEFAULT 0 NOT NULL,
    loc_removed bigint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: analysis_skill_events; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.analysis_skill_events AS
 SELECT e.id,
    e.user_id,
    e.session_id,
    e.plugin_id,
    replace("substring"(e.prompt_preview, '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)'::text), '_'::text, '-'::text) AS skill,
    NULL::text AS tool_use_id,
    'slash'::text AS source,
    e.created_at AS invoked_at
   FROM public.plugin_usage_events e
  WHERE ((e.event_type = 'UserPromptSubmit'::text) AND (e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'::text))
UNION ALL
 SELECT e.id,
    e.user_id,
    e.session_id,
    e.plugin_id,
    replace(((e.metadata -> 'tool_input'::text) ->> 'skill'::text), '_'::text, '-'::text) AS skill,
    (e.metadata ->> 'tool_use_id'::text) AS tool_use_id,
    'tool'::text AS source,
    e.created_at AS invoked_at
   FROM public.plugin_usage_events e
  WHERE ((e.event_type = ANY (ARRAY['PostToolUse'::text, 'PostToolUseFailure'::text])) AND (e.tool_name = 'Skill'::text) AND (((e.metadata -> 'tool_input'::text) ->> 'skill'::text) IS NOT NULL));


--
-- Name: managed_invocation_attributions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_invocation_attributions (
    id text NOT NULL,
    owner_id text NOT NULL,
    invocation_id text NOT NULL,
    installation_id text,
    resource_id text,
    revision_id text,
    publication_generation bigint,
    traffic_class text NOT NULL,
    status text NOT NULL,
    receipt_id text,
    authenticated_evidence jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_invocation_attributions_status_check CHECK ((status = ANY (ARRAY['verified'::text, 'revision_unknown'::text, 'unsupported'::text, 'historical'::text]))),
    CONSTRAINT managed_invocation_attributions_traffic_class_check CHECK ((traffic_class = ANY (ARRAY['production'::text, 'fixture'::text, 'live_evaluation'::text, 'suggestion'::text, 'judge'::text])))
);


--
-- Name: analysis_skill_version_events; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.analysis_skill_version_events AS
 SELECT e.id AS invocation_id,
    e.user_id,
    e.session_id,
    e.plugin_id,
    e.skill,
    e.tool_use_id,
    e.source,
    e.invoked_at,
    a.installation_id,
    a.resource_id,
    a.revision_id,
    a.publication_generation,
    COALESCE(a.traffic_class, 'production'::text) AS traffic_class,
    COALESCE(a.status, 'revision_unknown'::text) AS attribution_status
   FROM (public.analysis_skill_events e
     LEFT JOIN public.managed_invocation_attributions a ON (((a.owner_id = e.user_id) AND (a.invocation_id = e.id))));


--
-- Name: analytics_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.analytics_events (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    user_id character varying(255) NOT NULL,
    session_id text,
    context_id character varying(255),
    gateway_conversation_id character varying(255),
    provider_request_id character varying(255),
    event_type character varying(255) NOT NULL,
    event_category text NOT NULL,
    severity text NOT NULL,
    endpoint text,
    error_code integer,
    response_time_ms integer,
    agent_id character varying(255),
    task_id character varying(255),
    message text,
    metadata text,
    event_data jsonb DEFAULT '{}'::jsonb,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: anomaly_thresholds; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.anomaly_thresholds (
    metric_name character varying(100) NOT NULL,
    warning_threshold real NOT NULL,
    critical_threshold real NOT NULL,
    description text,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: approval_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.approval_requests (
    call_id text NOT NULL,
    tool_name text NOT NULL,
    server_name text NOT NULL,
    arguments jsonb DEFAULT '{}'::jsonb NOT NULL,
    args_digest text NOT NULL,
    requested_by text NOT NULL,
    session_id text,
    trace_id text,
    rule text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    approver_id text,
    approver_username text,
    decided_at timestamp with time zone,
    decision_note text,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT approval_decided_fields CHECK ((((status = 'pending'::text) AND (approver_id IS NULL) AND (decided_at IS NULL)) OR (status <> 'pending'::text))),
    CONSTRAINT approval_requests_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'denied'::text, 'expired'::text])))
);


--
-- Name: artifact_parts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.artifact_parts (
    id integer NOT NULL,
    artifact_id text NOT NULL,
    context_id text NOT NULL,
    part_kind text NOT NULL,
    sequence_number integer NOT NULL,
    text_content text,
    file_name text,
    file_mime_type text,
    file_uri text,
    file_bytes text,
    data_content jsonb,
    metadata jsonb DEFAULT '{}'::jsonb,
    CONSTRAINT artifact_parts_part_kind_check CHECK ((part_kind = ANY (ARRAY['text'::text, 'file'::text, 'data'::text]))),
    CONSTRAINT check_data_part CHECK (((part_kind <> 'data'::text) OR (data_content IS NOT NULL))),
    CONSTRAINT check_file_part CHECK (((part_kind <> 'file'::text) OR ((file_uri IS NOT NULL) OR (file_bytes IS NOT NULL)))),
    CONSTRAINT check_text_part CHECK (((part_kind <> 'text'::text) OR (text_content IS NOT NULL)))
);


--
-- Name: artifact_parts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.artifact_parts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: artifact_parts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.artifact_parts_id_seq OWNED BY public.artifact_parts.id;


--
-- Name: banned_ips; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.banned_ips (
    ip_address character varying(45) NOT NULL,
    reason character varying(255) NOT NULL,
    banned_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone,
    ban_count integer DEFAULT 1 NOT NULL,
    last_offense_path character varying(512),
    last_user_agent text,
    is_permanent boolean DEFAULT false NOT NULL,
    source_fingerprint text,
    ban_source character varying(50) DEFAULT 'manual'::character varying,
    associated_session_ids text[] DEFAULT '{}'::text[]
);


--
-- Name: bridge_exchange_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.bridge_exchange_codes (
    code_hash text NOT NULL,
    user_id text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    consumed_at timestamp with time zone
);


--
-- Name: bridge_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.bridge_sessions (
    session_id text NOT NULL,
    user_id text NOT NULL,
    bridge_version text NOT NULL,
    os text NOT NULL,
    hostname text NOT NULL,
    started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_heartbeat_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_activity_at timestamp with time zone,
    forwarded_total bigint DEFAULT 0 NOT NULL,
    tokens_in_total bigint DEFAULT 0 NOT NULL,
    tokens_out_total bigint DEFAULT 0 NOT NULL
);


--
-- Name: bridge_user_host_model_prefs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.bridge_user_host_model_prefs (
    user_id text NOT NULL,
    host_id text NOT NULL,
    model_protocols text[] NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: bridge_user_host_prefs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.bridge_user_host_prefs (
    user_id text NOT NULL,
    host_id text NOT NULL,
    enabled boolean NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: campaign_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.campaign_links (
    id text NOT NULL,
    short_code text NOT NULL,
    target_url text NOT NULL,
    link_type text NOT NULL,
    campaign_id text,
    campaign_name text,
    source_content_id text,
    source_page text,
    utm_params text,
    link_text text,
    link_position text,
    destination_type text,
    click_count integer DEFAULT 0 NOT NULL,
    unique_click_count integer DEFAULT 0 NOT NULL,
    conversion_count integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    expires_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: content_files; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.content_files (
    id integer NOT NULL,
    content_id text NOT NULL,
    file_id uuid NOT NULL,
    role character varying(50) DEFAULT 'attachment'::character varying NOT NULL,
    display_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: content_files_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.content_files_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: content_files_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.content_files_id_seq OWNED BY public.content_files.id;


--
-- Name: content_performance_metrics; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.content_performance_metrics (
    id text NOT NULL,
    content_id text NOT NULL,
    total_views integer DEFAULT 0 NOT NULL,
    unique_visitors integer DEFAULT 0 NOT NULL,
    avg_time_on_page_seconds double precision DEFAULT 0 NOT NULL,
    shares_total integer DEFAULT 0 NOT NULL,
    shares_linkedin integer DEFAULT 0 NOT NULL,
    shares_twitter integer DEFAULT 0 NOT NULL,
    comments_count integer DEFAULT 0 NOT NULL,
    search_impressions integer DEFAULT 0 NOT NULL,
    search_clicks integer DEFAULT 0 NOT NULL,
    avg_search_position real,
    views_last_7_days integer DEFAULT 0 NOT NULL,
    views_last_30_days integer DEFAULT 0 NOT NULL,
    trend_direction text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: context_agents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.context_agents (
    id integer NOT NULL,
    context_id text NOT NULL,
    agent_name text NOT NULL,
    added_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_active_at timestamp with time zone
);


--
-- Name: context_agents_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.context_agents_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: context_agents_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.context_agents_id_seq OWNED BY public.context_agents.id;


--
-- Name: context_notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.context_notifications (
    id integer NOT NULL,
    context_id text NOT NULL,
    agent_id text NOT NULL,
    notification_type text NOT NULL,
    notification_data jsonb NOT NULL,
    received_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    broadcasted boolean DEFAULT false NOT NULL,
    CONSTRAINT context_notifications_notification_type_check CHECK ((notification_type = ANY (ARRAY['notifications/taskStatusUpdate'::text, 'notifications/artifactCreated'::text, 'notifications/messageAdded'::text, 'notifications/contextUpdated'::text])))
);


--
-- Name: context_notifications_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.context_notifications_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: context_notifications_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.context_notifications_id_seq OWNED BY public.context_notifications.id;


--
-- Name: conversation_requests; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.conversation_requests AS
 WITH threads AS (
         SELECT ai_requests.context_id,
            ai_requests.gateway_conversation_id,
            count(*) AS thread_requests
           FROM public.ai_requests
          GROUP BY ai_requests.context_id, ai_requests.gateway_conversation_id
        )
 SELECT r.id,
    r.user_id,
    r.session_id,
    r.client_session_id,
    r.context_id,
    r.gateway_conversation_id,
    r.trace_id,
    r.provider,
    r.model,
    r.status,
    r.max_tokens,
    r.input_tokens,
    r.output_tokens,
    r.cost_microdollars,
    r.latency_ms,
    r.created_at,
    r.completed_at,
    public.conversation_request_kind(r.request_kind, (p.offered_tools IS NOT NULL), t.thread_requests) AS effective_kind
   FROM ((public.ai_requests r
     LEFT JOIN public.ai_request_payloads p ON ((p.ai_request_id = r.id)))
     JOIN threads t ON ((((t.context_id)::text = (r.context_id)::text) AND (NOT ((t.gateway_conversation_id)::text IS DISTINCT FROM (r.gateway_conversation_id)::text)))))
  WHERE ((r.context_id)::text <> '00000000-0000-0000-0000-4c4547414359'::text);


--
-- Name: conversation_rollups; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.conversation_rollups AS
 SELECT (context_id)::character varying(255) AS context_id,
    public.conversation_title(context_id, client_session_id) AS title,
    (user_id)::character varying AS user_id,
    (display_name)::character varying(255) AS display_name,
    (session_id)::character varying AS session_id,
    client_session_id,
    group_name,
    project_name,
    model,
    turn_count,
    side_call_count,
    side_call_cost_microdollars,
    tool_call_count,
    error_count,
    total_input_tokens,
    total_output_tokens,
    total_cost_microdollars,
    first_at,
    last_at,
    status
   FROM public.conversation_metrics_for(NULL::text[]) r(context_id, user_id, display_name, session_id, client_session_id, group_name, project_name, model, turn_count, side_call_count, side_call_cost_microdollars, tool_call_count, error_count, total_input_tokens, total_output_tokens, total_cost_microdollars, first_at, last_at, status);


--
-- Name: daily_summaries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.daily_summaries (
    user_id text NOT NULL,
    summary_date date NOT NULL,
    session_count integer DEFAULT 0 NOT NULL,
    avg_quality_score real,
    goals_achieved integer DEFAULT 0 NOT NULL,
    goals_partial integer DEFAULT 0 NOT NULL,
    goals_failed integer DEFAULT 0 NOT NULL,
    total_prompts bigint DEFAULT 0 NOT NULL,
    total_tool_uses bigint DEFAULT 0 NOT NULL,
    total_errors bigint DEFAULT 0 NOT NULL,
    summary text DEFAULT ''::text NOT NULL,
    patterns text,
    skill_gaps text,
    top_recommendation text,
    daily_xp integer DEFAULT 0 NOT NULL,
    tags text DEFAULT ''::text NOT NULL,
    avg_apm real,
    peak_apm real,
    avg_eapm real,
    peak_concurrency integer DEFAULT 0 NOT NULL,
    avg_concurrency real,
    total_input_bytes bigint DEFAULT 0 NOT NULL,
    total_output_bytes bigint DEFAULT 0 NOT NULL,
    peak_throughput_bps bigint DEFAULT 0 NOT NULL,
    tool_diversity integer DEFAULT 0 NOT NULL,
    multitasking_score real,
    session_velocity real,
    achievements_unlocked text DEFAULT ''::text NOT NULL,
    highlights text,
    trends text,
    category_distribution jsonb,
    plugins_count integer DEFAULT 0 NOT NULL,
    skills_count integer DEFAULT 0 NOT NULL,
    agents_count integer DEFAULT 0 NOT NULL,
    mcp_servers_count integer DEFAULT 0 NOT NULL,
    hooks_count integer DEFAULT 0 NOT NULL,
    health_score real,
    skill_effectiveness jsonb,
    avg_session_duration_minutes real,
    avg_turns_per_session real,
    total_corrections integer DEFAULT 0 NOT NULL,
    avg_automation_ratio real,
    plan_mode_sessions integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: dev_login_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.dev_login_codes (
    code_hash text NOT NULL,
    user_id text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    consumed_at timestamp with time zone
);


--
-- Name: device_app_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.device_app_links (
    device_id text NOT NULL,
    user_id text NOT NULL,
    app_platform text NOT NULL,
    app_version text DEFAULT ''::text NOT NULL,
    hostname text DEFAULT ''::text NOT NULL,
    last_seen_at timestamp with time zone,
    enrolled_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT device_app_links_app_platform_check CHECK ((app_platform = ANY (ARRAY['macos'::text, 'windows'::text, 'linux'::text])))
);


--
-- Name: engagement_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.engagement_events (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    session_id text NOT NULL,
    user_id character varying(255) NOT NULL,
    page_url text NOT NULL,
    content_id text,
    event_type character varying(50) DEFAULT 'page_exit'::character varying NOT NULL,
    time_on_page_ms integer DEFAULT 0 NOT NULL,
    time_to_first_interaction_ms integer,
    time_to_first_scroll_ms integer,
    max_scroll_depth integer DEFAULT 0 NOT NULL,
    scroll_velocity_avg real,
    scroll_direction_changes integer DEFAULT 0,
    click_count integer DEFAULT 0 NOT NULL,
    mouse_move_distance_px integer DEFAULT 0,
    keyboard_events integer DEFAULT 0,
    copy_events integer DEFAULT 0,
    focus_time_ms integer DEFAULT 0 NOT NULL,
    blur_count integer DEFAULT 0 NOT NULL,
    tab_switches integer DEFAULT 0 NOT NULL,
    visible_time_ms integer DEFAULT 0 NOT NULL,
    hidden_time_ms integer DEFAULT 0 NOT NULL,
    is_rage_click boolean DEFAULT false,
    is_dead_click boolean DEFAULT false,
    reading_pattern character varying(50),
    event_data jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: eval_approved_operation_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_approved_operation_receipts (
    approval_id text NOT NULL,
    execution_id text NOT NULL,
    operation_digest text NOT NULL,
    output jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_approved_operation_receipts_operation_digest_check CHECK ((operation_digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: eval_budget_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_budget_accounts (
    id text NOT NULL,
    owner_id text NOT NULL,
    cap bigint NOT NULL,
    reserved bigint DEFAULT 0 NOT NULL,
    settled bigint DEFAULT 0 NOT NULL,
    frozen boolean DEFAULT false NOT NULL,
    operation_key text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_budget_accounts_cap_check CHECK ((cap > 0)),
    CONSTRAINT eval_budget_accounts_reserved_check CHECK ((reserved >= 0)),
    CONSTRAINT eval_budget_accounts_settled_check CHECK ((settled >= 0))
);


--
-- Name: eval_budget_reservations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_budget_reservations (
    id text NOT NULL,
    account_id text NOT NULL,
    operation_key text NOT NULL,
    reserved bigint NOT NULL,
    actual bigint,
    request_id text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    settled_at timestamp with time zone,
    CONSTRAINT eval_budget_reservations_actual_check CHECK ((actual >= 0)),
    CONSTRAINT eval_budget_reservations_reserved_check CHECK ((reserved > 0))
);


--
-- Name: eval_cases; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_cases (
    id text NOT NULL,
    name text NOT NULL,
    prompt_body jsonb NOT NULL,
    source_ai_request_id text,
    expectation text,
    baseline_response jsonb,
    baseline_model text,
    tags text[] DEFAULT ARRAY[]::text[] NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    created_by text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    repair_hint text,
    canonical_messages jsonb,
    system_prompt text,
    offered_tools jsonb,
    provider text,
    model text,
    prepared_body_sha256 text
);


--
-- Name: eval_execution_approvals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_approvals (
    id text NOT NULL,
    execution_id text NOT NULL,
    owner_id text NOT NULL,
    fencing_token bigint NOT NULL,
    operation jsonb NOT NULL,
    operation_digest text NOT NULL,
    precondition_digest text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    requested_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '24:00:00'::interval) NOT NULL,
    decided_by text,
    decided_at timestamp with time zone,
    CONSTRAINT eval_execution_approvals_operation_digest_check CHECK ((operation_digest ~ '^[0-9a-f]{64}$'::text)),
    CONSTRAINT eval_execution_approvals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'denied'::text, 'expired'::text, 'consumed'::text])))
);


--
-- Name: eval_execution_artifacts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_artifacts (
    execution_id text NOT NULL,
    content jsonb NOT NULL
);


--
-- Name: eval_execution_capabilities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_capabilities (
    token_hash text NOT NULL,
    execution_id text NOT NULL,
    worker_id text NOT NULL,
    session_id text NOT NULL,
    fencing_token bigint NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    revoked_at timestamp with time zone
);


--
-- Name: eval_execution_cleanup; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_cleanup (
    execution_id text NOT NULL,
    container_id text,
    network_id text,
    status text NOT NULL,
    attempts integer DEFAULT 0 NOT NULL,
    last_error text,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_execution_cleanup_attempts_check CHECK ((attempts >= 0)),
    CONSTRAINT eval_execution_cleanup_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'verified'::text, 'failed'::text, 'retrying'::text])))
);


--
-- Name: eval_execution_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_events (
    execution_id text NOT NULL,
    sequence bigint NOT NULL,
    payload jsonb NOT NULL,
    digest text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_execution_events_sequence_check CHECK ((sequence >= 0))
);


--
-- Name: eval_execution_evidence; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_evidence (
    execution_id text NOT NULL,
    fencing_token bigint NOT NULL,
    digest text NOT NULL,
    manifest jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: eval_execution_measurements; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_execution_measurements (
    execution_id text NOT NULL,
    hard_failures text[] DEFAULT ARRAY[]::text[] NOT NULL,
    deterministic_checks jsonb DEFAULT '{}'::jsonb NOT NULL,
    judgment jsonb,
    quality_milli integer,
    latency_ms bigint,
    input_tokens bigint,
    output_tokens bigint,
    tool_calls bigint,
    attempted_cost_microdollars bigint DEFAULT 0 CONSTRAINT eval_execution_measurements_attempted_cost_microdollar_not_null NOT NULL,
    accounting_status text NOT NULL,
    verified_success boolean DEFAULT false NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_execution_measurements_accounting_status_check CHECK ((accounting_status = ANY (ARRAY['complete'::text, 'partial'::text, 'unknown'::text]))),
    CONSTRAINT eval_execution_measurements_attempted_cost_microdollars_check CHECK ((attempted_cost_microdollars >= 0)),
    CONSTRAINT eval_execution_measurements_input_tokens_check CHECK ((input_tokens >= 0)),
    CONSTRAINT eval_execution_measurements_latency_ms_check CHECK ((latency_ms >= 0)),
    CONSTRAINT eval_execution_measurements_output_tokens_check CHECK ((output_tokens >= 0)),
    CONSTRAINT eval_execution_measurements_quality_milli_check CHECK (((quality_milli >= 0) AND (quality_milli <= 5000))),
    CONSTRAINT eval_execution_measurements_tool_calls_check CHECK ((tool_calls >= 0))
);


--
-- Name: eval_executions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_executions (
    id text NOT NULL,
    experiment_id text NOT NULL,
    variant_index integer NOT NULL,
    case_revision_id text NOT NULL,
    repetition integer NOT NULL,
    status text DEFAULT 'queued'::text NOT NULL,
    lease_owner text,
    lease_expires_at timestamp with time zone,
    deadline_at timestamp with time zone,
    active_runtime_ms bigint DEFAULT 0 NOT NULL,
    last_heartbeat_at timestamp with time zone,
    fencing_token bigint DEFAULT 0 NOT NULL,
    result jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    finished_at timestamp with time zone,
    CONSTRAINT eval_executions_active_runtime_ms_check CHECK (((active_runtime_ms >= 0) AND (active_runtime_ms <= 1800000))),
    CONSTRAINT eval_executions_repetition_check CHECK ((repetition >= 0)),
    CONSTRAINT eval_executions_status_check CHECK ((status = ANY (ARRAY['queued'::text, 'running'::text, 'awaiting_approval'::text, 'completed'::text, 'error'::text, 'cancelled'::text, 'blocked'::text, 'budget_exhausted'::text]))),
    CONSTRAINT eval_executions_variant_index_check CHECK ((variant_index >= 0))
);


--
-- Name: eval_experiments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_experiments (
    id text NOT NULL,
    owner_id text NOT NULL,
    spec jsonb NOT NULL,
    spec_digest text NOT NULL,
    budget_id text NOT NULL,
    idempotency_key text NOT NULL,
    status text DEFAULT 'queued'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_experiments_status_check CHECK ((status = ANY (ARRAY['queued'::text, 'running'::text, 'completed'::text, 'cancelled'::text, 'blocked'::text])))
);


--
-- Name: eval_fixture_payloads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_fixture_payloads (
    owner_id text NOT NULL,
    fixture_key text NOT NULL,
    digest text NOT NULL,
    payload jsonb NOT NULL,
    evidence_label text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: eval_fixture_test_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_fixture_test_records (
    owner_id text NOT NULL,
    record_key text NOT NULL,
    value jsonb NOT NULL,
    original_value jsonb NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: eval_holdout_consumption; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_holdout_consumption (
    owner_id text NOT NULL,
    case_revision_id text NOT NULL,
    experiment_id text NOT NULL,
    consumed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: eval_managed_workspace_assets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_managed_workspace_assets (
    owner_id text NOT NULL,
    workspace_digest text NOT NULL,
    path text NOT NULL,
    asset_digest text NOT NULL,
    content bytea NOT NULL,
    executable boolean DEFAULT false NOT NULL,
    CONSTRAINT eval_managed_workspace_assets_asset_digest_check CHECK ((asset_digest ~ '^[0-9a-f]{64}$'::text)),
    CONSTRAINT eval_managed_workspace_assets_workspace_digest_check CHECK ((workspace_digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: eval_managed_workspace_projections; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_managed_workspace_projections (
    owner_id text NOT NULL,
    digest text NOT NULL,
    managed_revision_id text NOT NULL,
    publication_generation bigint,
    manifest jsonb NOT NULL,
    verified_file_count integer NOT NULL,
    verified_byte_count bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_managed_workspace_projections_digest_check CHECK ((digest ~ '^[0-9a-f]{64}$'::text)),
    CONSTRAINT eval_managed_workspace_projections_verified_byte_count_check CHECK (((verified_byte_count >= 0) AND (verified_byte_count <= 8388608))),
    CONSTRAINT eval_managed_workspace_projections_verified_file_count_check CHECK (((verified_file_count >= 0) AND (verified_file_count <= 256)))
);


--
-- Name: eval_request_reservations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_request_reservations (
    request_id text NOT NULL,
    execution_id text NOT NULL,
    reservation_id text NOT NULL,
    traffic_class text DEFAULT 'live_evaluation'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_request_reservations_traffic_class_check CHECK ((traffic_class = ANY (ARRAY['fixture'::text, 'live_evaluation'::text, 'suggestion'::text, 'judge'::text])))
);


--
-- Name: eval_resource_revisions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_resource_revisions (
    id text NOT NULL,
    owner_id text NOT NULL,
    resource_kind text NOT NULL,
    resource_key text NOT NULL,
    digest text NOT NULL,
    content jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_resource_revisions_resource_kind_check CHECK ((resource_kind = ANY (ARRAY['case'::text, 'dataset'::text, 'rubric'::text, 'policy'::text])))
);


--
-- Name: eval_session_bindings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_session_bindings (
    session_id text NOT NULL,
    execution_id text NOT NULL,
    owner_id text NOT NULL,
    fencing_token bigint NOT NULL,
    traffic_class text DEFAULT 'live_evaluation'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_session_bindings_traffic_class_check CHECK ((traffic_class = ANY (ARRAY['fixture'::text, 'live_evaluation'::text, 'suggestion'::text, 'judge'::text])))
);


--
-- Name: eval_suggestions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_suggestions (
    id text NOT NULL,
    owner_id text NOT NULL,
    experiment_id text NOT NULL,
    candidate_revision_id text,
    supporting_execution_ids text[] NOT NULL,
    proposed_changes jsonb NOT NULL,
    hypothesis text NOT NULL,
    reservation_id text NOT NULL,
    originating_evidence jsonb NOT NULL,
    status text DEFAULT 'draft'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT eval_suggestions_hypothesis_check CHECK (((length(hypothesis) >= 1) AND (length(hypothesis) <= 4000))),
    CONSTRAINT eval_suggestions_status_check CHECK ((status = ANY (ARRAY['draft'::text, 'accepted'::text, 'rejected'::text])))
);


--
-- Name: eval_workers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.eval_workers (
    id text NOT NULL,
    owner_id text NOT NULL,
    environment text NOT NULL,
    name text NOT NULL,
    token_hash text NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '7 days'::interval) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: event_outbox; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.event_outbox (
    id text NOT NULL,
    channel text NOT NULL,
    user_id text NOT NULL,
    payload jsonb NOT NULL,
    actor_kind text NOT NULL,
    actor_id text NOT NULL,
    origin_instance_id text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT event_outbox_actor_id_check CHECK ((length(actor_id) > 0)),
    CONSTRAINT event_outbox_actor_kind_check CHECK ((actor_kind = ANY (ARRAY['user'::text, 'job'::text, 'mcp'::text])))
);


--
-- Name: extension_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.extension_migrations (
    id text NOT NULL,
    extension_id text NOT NULL,
    version integer NOT NULL,
    name text NOT NULL,
    checksum text NOT NULL,
    applied_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: federated_identities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.federated_identities (
    issuer text NOT NULL,
    external_sub text NOT NULL,
    user_id text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: files; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.files (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    path text NOT NULL,
    public_url text NOT NULL,
    mime_type character varying(255) NOT NULL,
    size_bytes bigint,
    ai_content boolean DEFAULT false NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    user_id character varying(255),
    session_id character varying(255),
    trace_id character varying(255),
    context_id character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: fingerprint_reputation; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.fingerprint_reputation (
    fingerprint_hash text NOT NULL,
    first_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    total_session_count integer DEFAULT 0 NOT NULL,
    active_session_count integer DEFAULT 0 NOT NULL,
    total_request_count bigint DEFAULT 0 NOT NULL,
    requests_last_hour integer DEFAULT 0 NOT NULL,
    peak_requests_per_minute real DEFAULT 0 NOT NULL,
    sustained_high_velocity_minutes integer DEFAULT 0 NOT NULL,
    is_flagged boolean DEFAULT false NOT NULL,
    flag_reason text,
    flagged_at timestamp with time zone,
    reputation_score integer DEFAULT 50 NOT NULL,
    abuse_incidents integer DEFAULT 0 NOT NULL,
    last_abuse_at timestamp with time zone,
    last_ip_address text,
    last_user_agent text,
    associated_user_ids text[] DEFAULT '{}'::text[] NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: funnel_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.funnel_progress (
    id text NOT NULL,
    funnel_id text NOT NULL,
    session_id text NOT NULL,
    current_step integer DEFAULT 0 NOT NULL,
    completed_at timestamp with time zone,
    dropped_at_step integer,
    step_timestamps jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: funnel_steps; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.funnel_steps (
    funnel_id text NOT NULL,
    step_order integer NOT NULL,
    name text NOT NULL,
    match_pattern text NOT NULL,
    match_type text DEFAULT 'url_prefix'::text NOT NULL
);


--
-- Name: funnels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.funnels (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: governance_decisions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.governance_decisions (
    id text NOT NULL,
    user_id text NOT NULL,
    session_id text NOT NULL,
    tool_name text NOT NULL,
    agent_id text,
    agent_scope text,
    decision text NOT NULL,
    policy text NOT NULL,
    reason text NOT NULL,
    evaluated_rules jsonb DEFAULT '[]'::jsonb,
    plugin_id text,
    actor_kind text NOT NULL,
    actor_id text NOT NULL,
    act_chain jsonb DEFAULT '[]'::jsonb NOT NULL,
    context_id text NOT NULL,
    task_id text,
    trace_id text,
    client_id text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT governance_decisions_actor_id_check CHECK ((length(actor_id) > 0)),
    CONSTRAINT governance_decisions_actor_kind_check CHECK ((actor_kind = ANY (ARRAY['user'::text, 'anonymous'::text, 'system'::text, 'job'::text, 'mcp'::text, 'agent'::text]))),
    CONSTRAINT governance_decisions_decision_check CHECK ((decision = ANY (ARRAY['allow'::text, 'warn'::text, 'deny'::text, 'pending'::text])))
);


--
-- Name: group_ad_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.group_ad_mappings (
    ad_group text NOT NULL,
    group_id text NOT NULL,
    source text DEFAULT 'dashboard'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT group_ad_mappings_source_check CHECK ((source = ANY (ARRAY['yaml'::text, 'dashboard'::text])))
);


--
-- Name: group_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.group_members (
    group_id text NOT NULL,
    user_id text NOT NULL,
    source text NOT NULL,
    source_ad_group text,
    granted_by text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT group_members_source_check CHECK ((source = ANY (ARRAY['adfs'::text, 'manual'::text])))
);


--
-- Name: groups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.groups (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    is_system boolean DEFAULT false NOT NULL,
    source text DEFAULT 'dashboard'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT groups_id_check CHECK ((id ~ '^[a-z0-9][a-z0-9_-]{0,63}$'::text)),
    CONSTRAINT groups_source_check CHECK ((source = ANY (ARRAY['yaml'::text, 'dashboard'::text, 'system'::text])))
);


--
-- Name: id_jag_replay; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.id_jag_replay (
    jti text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    seen_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: ingestion_event_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ingestion_event_receipts (
    dedup_key text NOT NULL,
    payload_digest text NOT NULL,
    accepted_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: ingestion_outbox; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ingestion_outbox (
    event_id text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    processed_at timestamp with time zone
);


--
-- Name: ingestion_repairs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ingestion_repairs (
    request_id text NOT NULL,
    repair_kind text NOT NULL,
    source_digest text,
    recovered_client_session_id text NOT NULL,
    repaired_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: ingestion_session_owners; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ingestion_session_owners (
    session_id text NOT NULL,
    user_id text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT ingestion_session_owners_session_id_check CHECK (((length(session_id) >= 1) AND (length(session_id) <= 255)))
);


--
-- Name: link_clicks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.link_clicks (
    id text NOT NULL,
    link_id text NOT NULL,
    session_id text NOT NULL,
    user_id text,
    context_id text,
    task_id text,
    referrer_page text,
    referrer_url text,
    clicked_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    user_agent text,
    ip_address text,
    device_type text,
    country text,
    is_first_click boolean DEFAULT false NOT NULL,
    is_conversion boolean DEFAULT false NOT NULL,
    conversion_at timestamp with time zone,
    time_on_page_seconds integer,
    scroll_depth_percent integer
);


--
-- Name: logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.logs (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    "timestamp" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    level character varying(50) NOT NULL,
    module character varying(255) NOT NULL,
    message text NOT NULL,
    metadata text,
    user_id character varying(255),
    session_id character varying(255),
    task_id character varying(255),
    trace_id character varying(255),
    context_id character varying(255),
    gateway_conversation_id character varying(255),
    provider_request_id character varying(255),
    client_id character varying(255),
    instance_id character varying(255),
    CONSTRAINT log_level_check CHECK (((level)::text = ANY ((ARRAY['ERROR'::character varying, 'WARN'::character varying, 'INFO'::character varying, 'DEBUG'::character varying, 'TRACE'::character varying])::text[]))),
    CONSTRAINT logs_level_check CHECK (((level)::text = ANY ((ARRAY['ERROR'::character varying, 'WARN'::character varying, 'INFO'::character varying, 'DEBUG'::character varying, 'TRACE'::character varying])::text[])))
);


--
-- Name: managed_assets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_assets (
    owner_id text NOT NULL,
    digest text NOT NULL,
    content bytea NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_assets_content_check CHECK ((octet_length(content) <= 16777216)),
    CONSTRAINT managed_assets_digest_check CHECK ((digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: managed_distribution_deliveries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_distribution_deliveries (
    id text NOT NULL,
    owner_id text NOT NULL,
    outbox_id text NOT NULL,
    publication_id text NOT NULL,
    generation bigint NOT NULL,
    bundle_digest text,
    status text NOT NULL,
    claim_token text NOT NULL,
    claimed_at timestamp with time zone DEFAULT now() NOT NULL,
    delivered_at timestamp with time zone,
    error text,
    CONSTRAINT managed_distribution_deliveries_generation_check CHECK ((generation > 0)),
    CONSTRAINT managed_distribution_deliveries_status_check CHECK ((status = ANY (ARRAY['claimed'::text, 'distributed'::text, 'failed'::text])))
);


--
-- Name: managed_distribution_outbox; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_distribution_outbox (
    id text NOT NULL,
    owner_id text NOT NULL,
    publication_id text NOT NULL,
    generation bigint NOT NULL,
    payload jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    delivered_at timestamp with time zone
);


--
-- Name: managed_installation_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_installation_receipts (
    id text NOT NULL,
    owner_id text NOT NULL,
    installation_id text NOT NULL,
    publication_id text NOT NULL,
    resource_id text NOT NULL,
    generation bigint NOT NULL,
    bundle_digest text NOT NULL,
    installed_manifest jsonb NOT NULL,
    client_evidence jsonb NOT NULL,
    verified_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_installation_receipts_bundle_digest_check CHECK ((bundle_digest ~ '^[0-9a-f]{64}$'::text)),
    CONSTRAINT managed_installation_receipts_generation_check CHECK ((generation > 0))
);


--
-- Name: managed_publication_reviews; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_publication_reviews (
    id text NOT NULL,
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    revision_id text,
    action text NOT NULL,
    bundle_digest text,
    comparison_evidence jsonb NOT NULL,
    limitations text NOT NULL,
    reviewer_id text NOT NULL,
    expected_generation bigint NOT NULL,
    request_digest text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_publication_reviews_action_check CHECK ((action = ANY (ARRAY['initial_adoption'::text, 'publish_improvement'::text, 'withdraw'::text, 'rollback'::text]))),
    CONSTRAINT managed_publication_reviews_bundle_digest_check CHECK (((bundle_digest IS NULL) OR (bundle_digest ~ '^[0-9a-f]{64}$'::text))),
    CONSTRAINT managed_publication_reviews_expected_generation_check CHECK ((expected_generation >= 0)),
    CONSTRAINT managed_publication_reviews_limitations_check CHECK ((length(limitations) <= 4000)),
    CONSTRAINT managed_publication_reviews_request_digest_check CHECK ((request_digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: managed_publication_selections; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_publication_selections (
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    generation bigint NOT NULL,
    state text NOT NULL,
    publication_id text NOT NULL,
    revision_id text,
    bundle_digest text,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_publication_selections_bundle_digest_check CHECK (((bundle_digest IS NULL) OR (bundle_digest ~ '^[0-9a-f]{64}$'::text))),
    CONSTRAINT managed_publication_selections_check CHECK ((((state = 'published'::text) AND (revision_id IS NOT NULL) AND (bundle_digest IS NOT NULL)) OR ((state = 'withdrawn'::text) AND (revision_id IS NULL) AND (bundle_digest IS NULL)))),
    CONSTRAINT managed_publication_selections_generation_check CHECK ((generation > 0)),
    CONSTRAINT managed_publication_selections_state_check CHECK ((state = ANY (ARRAY['published'::text, 'withdrawn'::text])))
);


--
-- Name: managed_publications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_publications (
    id text NOT NULL,
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    review_id text NOT NULL,
    generation bigint NOT NULL,
    action text NOT NULL,
    revision_id text,
    bundle_digest text,
    operation_key text NOT NULL,
    request_digest text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_publications_action_check CHECK ((action = ANY (ARRAY['initial_adoption'::text, 'publish_improvement'::text, 'withdraw'::text, 'rollback'::text]))),
    CONSTRAINT managed_publications_bundle_digest_check CHECK (((bundle_digest IS NULL) OR (bundle_digest ~ '^[0-9a-f]{64}$'::text))),
    CONSTRAINT managed_publications_generation_check CHECK ((generation > 0)),
    CONSTRAINT managed_publications_operation_key_check CHECK (((length(operation_key) >= 1) AND (length(operation_key) <= 200))),
    CONSTRAINT managed_publications_request_digest_check CHECK ((request_digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: managed_reconciliation_conflicts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_reconciliation_conflicts (
    reconciliation_id text NOT NULL,
    path text NOT NULL,
    base_digest text,
    candidate_digest text,
    incoming_digest text,
    resolution text,
    resolved_digest text,
    CONSTRAINT managed_reconciliation_conflicts_resolution_check CHECK ((resolution = ANY (ARRAY['candidate'::text, 'incoming'::text, 'manual'::text, 'delete'::text])))
);


--
-- Name: managed_reconciliations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_reconciliations (
    id text NOT NULL,
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    upstream_base_revision_id text NOT NULL,
    managed_candidate_revision_id text NOT NULL,
    incoming_revision_id text NOT NULL,
    status text DEFAULT 'open'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    resolved_revision_id text,
    resolved_by text,
    resolved_at timestamp with time zone,
    CONSTRAINT managed_reconciliations_status_check CHECK ((status = ANY (ARRAY['open'::text, 'resolved'::text, 'withdrawal_proposed'::text])))
);


--
-- Name: managed_resources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_resources (
    id text NOT NULL,
    owner_id text NOT NULL,
    source_id text NOT NULL,
    upstream_key text NOT NULL,
    kind text NOT NULL,
    resource_key text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_resources_kind_check CHECK ((kind = ANY (ARRAY['skill'::text, 'plugin'::text, 'marketplace'::text, 'supporting'::text]))),
    CONSTRAINT managed_resources_resource_key_check CHECK (((length(resource_key) >= 1) AND (length(resource_key) <= 200))),
    CONSTRAINT managed_resources_upstream_key_check CHECK (((length(upstream_key) >= 1) AND (length(upstream_key) <= 200)))
);


--
-- Name: managed_revision_assets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_revision_assets (
    owner_id text NOT NULL,
    revision_id text NOT NULL,
    path text NOT NULL,
    digest text NOT NULL
);


--
-- Name: managed_revision_dependencies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_revision_dependencies (
    owner_id text NOT NULL,
    revision_id text NOT NULL,
    dependency_id text NOT NULL,
    CONSTRAINT managed_revision_dependencies_check CHECK ((revision_id <> dependency_id))
);


--
-- Name: managed_revisions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_revisions (
    id text NOT NULL,
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    source_id text NOT NULL,
    snapshot_id text NOT NULL,
    parent_id text,
    digest text NOT NULL,
    manifest jsonb NOT NULL,
    rationale text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_revisions_digest_check CHECK ((digest ~ '^[0-9a-f]{64}$'::text)),
    CONSTRAINT managed_revisions_rationale_check CHECK (((length(rationale) >= 1) AND (length(rationale) <= 4000)))
);


--
-- Name: managed_source_snapshots; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_source_snapshots (
    id text NOT NULL,
    owner_id text NOT NULL,
    source_id text NOT NULL,
    digest text NOT NULL,
    provenance jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_source_snapshots_digest_check CHECK ((digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: managed_sources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_sources (
    id text NOT NULL,
    owner_id text NOT NULL,
    name text NOT NULL,
    kind text NOT NULL,
    specification jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_sources_kind_check CHECK ((kind = ANY (ARRAY['git'::text, 'local_tree'::text, 'managed'::text]))),
    CONSTRAINT managed_sources_name_check CHECK (((length(name) >= 1) AND (length(name) <= 200)))
);


--
-- Name: managed_withdrawal_proposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_withdrawal_proposals (
    id text NOT NULL,
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    snapshot_id text NOT NULL,
    reason text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    decided_by text,
    decided_at timestamp with time zone,
    CONSTRAINT managed_withdrawal_proposals_reason_check CHECK (((length(reason) >= 1) AND (length(reason) <= 4000))),
    CONSTRAINT managed_withdrawal_proposals_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'approved'::text, 'rejected'::text])))
);


--
-- Name: markdown_categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.markdown_categories (
    id text NOT NULL,
    name text NOT NULL,
    slug text NOT NULL,
    description text,
    parent_id text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: markdown_content; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.markdown_content (
    id text NOT NULL,
    slug text NOT NULL,
    locale text DEFAULT 'en'::text NOT NULL,
    title text NOT NULL,
    description text NOT NULL,
    body text NOT NULL,
    author text NOT NULL,
    published_at timestamp with time zone NOT NULL,
    keywords text NOT NULL,
    kind text DEFAULT 'article'::text NOT NULL,
    image text,
    category_id text,
    source_id text NOT NULL,
    version_hash text NOT NULL,
    public boolean DEFAULT true NOT NULL,
    links jsonb DEFAULT '[]'::jsonb NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: markdown_content_enrichment; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.markdown_content_enrichment (
    content_id text NOT NULL,
    category text,
    after_reading_this jsonb DEFAULT '[]'::jsonb NOT NULL,
    related_playbooks jsonb DEFAULT '[]'::jsonb NOT NULL,
    related_code jsonb DEFAULT '[]'::jsonb NOT NULL,
    related_docs jsonb DEFAULT '[]'::jsonb NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: markdown_fts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.markdown_fts (
    content_id text NOT NULL,
    search_vector tsvector
);


--
-- Name: mcp_artifacts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_artifacts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    artifact_id character varying(255) NOT NULL,
    mcp_execution_id character varying(255) NOT NULL,
    context_id character varying(255),
    user_id character varying(255),
    server_name character varying(255) NOT NULL,
    artifact_type character varying(100) NOT NULL,
    title character varying(500),
    data jsonb NOT NULL,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone
);


--
-- Name: mcp_connector_revision; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.mcp_connector_revision
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: mcp_connector_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_connector_accounts (
    user_id text NOT NULL,
    provider text NOT NULL,
    status text DEFAULT 'not_connected'::text NOT NULL,
    auth_method text,
    account_id text,
    account_name text,
    resource_id text,
    resource_name text,
    error_code text,
    verified_at timestamp with time zone,
    generation bigint DEFAULT 0 NOT NULL,
    revision bigint DEFAULT nextval('public.mcp_connector_revision'::regclass) NOT NULL,
    CONSTRAINT mcp_connector_accounts_provider_check CHECK ((provider ~ '^[A-Za-z0-9_-]{1,128}$'::text))
);


--
-- Name: mcp_connector_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_connector_credentials (
    user_id text NOT NULL,
    provider text NOT NULL,
    ciphertext bytea NOT NULL,
    nonce bytea NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT mcp_connector_credentials_nonce_check CHECK ((octet_length(nonce) = 12)),
    CONSTRAINT mcp_connector_credentials_provider_check CHECK ((provider ~ '^[A-Za-z0-9_-]{1,128}$'::text))
);


--
-- Name: mcp_connector_oauth_states; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_connector_oauth_states (
    state text NOT NULL,
    user_id text NOT NULL,
    provider text NOT NULL,
    ciphertext bytea NOT NULL,
    nonce bytea NOT NULL,
    expires_at timestamp with time zone DEFAULT (now() + '00:10:00'::interval) NOT NULL,
    CONSTRAINT mcp_connector_oauth_states_nonce_check CHECK ((octet_length(nonce) = 12)),
    CONSTRAINT mcp_connector_oauth_states_provider_check CHECK ((provider ~ '^[A-Za-z0-9_-]{1,128}$'::text))
);


--
-- Name: mcp_external_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_external_sessions (
    server_name text NOT NULL,
    session_id text NOT NULL,
    user_id text NOT NULL,
    credential_hash bytea NOT NULL,
    expires_at timestamp with time zone DEFAULT (CURRENT_TIMESTAMP + '01:00:00'::interval) NOT NULL
);


--
-- Name: mcp_proxy_identities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_proxy_identities (
    session_id text NOT NULL,
    user_id character varying(255) NOT NULL,
    user_type text NOT NULL,
    permissions jsonb NOT NULL,
    auth_token text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone DEFAULT (CURRENT_TIMESTAMP + '24:00:00'::interval) NOT NULL
);


--
-- Name: mcp_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_sessions (
    session_id text NOT NULL,
    user_id character varying(255),
    mcp_server_id text,
    last_event_id text,
    initialize_params jsonb,
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_activity_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone DEFAULT (CURRENT_TIMESTAMP + '24:00:00'::interval) NOT NULL,
    CONSTRAINT mcp_sessions_status_check CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'closed'::character varying, 'expired'::character varying])::text[])))
);


--
-- Name: mcp_tool_executions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_tool_executions (
    mcp_execution_id text DEFAULT gen_random_uuid() NOT NULL,
    tool_name character varying(255) NOT NULL,
    server_name character varying(255) NOT NULL,
    started_at timestamp with time zone NOT NULL,
    completed_at timestamp with time zone,
    execution_time_ms integer,
    input text NOT NULL,
    output text,
    output_schema text,
    status character varying(255) DEFAULT 'pending'::character varying NOT NULL,
    error_message text,
    user_id character varying(255) NOT NULL,
    session_id character varying(255),
    context_id character varying(255),
    task_id character varying(255),
    trace_id character varying(255),
    request_method text,
    request_source text,
    actor_kind text,
    actor_id text,
    ai_tool_call_id character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT mcp_tool_executions_status_check CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'success'::character varying, 'failed'::character varying, 'timeout'::character varying])::text[])))
);


--
-- Name: message_parts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_parts (
    id integer NOT NULL,
    message_id text NOT NULL,
    task_id text NOT NULL,
    part_kind text NOT NULL,
    sequence_number integer NOT NULL,
    text_content text,
    file_name text,
    file_mime_type text,
    file_uri text,
    file_bytes text,
    file_id uuid,
    data_content jsonb,
    metadata jsonb DEFAULT '{}'::jsonb,
    CONSTRAINT check_data_part CHECK (((part_kind <> 'data'::text) OR (data_content IS NOT NULL))),
    CONSTRAINT check_file_part CHECK (((part_kind <> 'file'::text) OR ((file_uri IS NOT NULL) OR (file_bytes IS NOT NULL)))),
    CONSTRAINT check_text_part CHECK (((part_kind <> 'text'::text) OR (text_content IS NOT NULL))),
    CONSTRAINT message_parts_part_kind_check CHECK ((part_kind = ANY (ARRAY['text'::text, 'file'::text, 'data'::text])))
);


--
-- Name: message_parts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.message_parts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: message_parts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.message_parts_id_seq OWNED BY public.message_parts.id;


--
-- Name: oauth_auth_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_auth_codes (
    code character varying(255) NOT NULL,
    client_id character varying(255) NOT NULL,
    user_id character varying(255) NOT NULL,
    redirect_uri text NOT NULL,
    scope text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    code_challenge text,
    code_challenge_method text,
    nonce text,
    resource text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    used_at timestamp with time zone,
    refresh_token_id text,
    CONSTRAINT oauth_auth_codes_check CHECK ((expires_at > created_at)),
    CONSTRAINT oauth_auth_codes_code_challenge_method_check CHECK (((code_challenge_method = ANY (ARRAY['S256'::text, 'plain'::text])) OR (code_challenge_method IS NULL)))
);


--
-- Name: oauth_client_contacts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_client_contacts (
    client_id character varying(255) NOT NULL,
    contact_email character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: oauth_client_grant_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_client_grant_types (
    client_id character varying(255) NOT NULL,
    grant_type character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT oauth_client_grant_types_grant_type_check CHECK (((grant_type)::text = ANY ((ARRAY['authorization_code'::character varying, 'refresh_token'::character varying, 'client_credentials'::character varying, 'password'::character varying])::text[])))
);


--
-- Name: oauth_client_redirect_uris; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_client_redirect_uris (
    client_id character varying(255) NOT NULL,
    redirect_uri text NOT NULL,
    is_primary boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: oauth_client_response_types; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_client_response_types (
    client_id character varying(255) NOT NULL,
    response_type character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT oauth_client_response_types_response_type_check CHECK (((response_type)::text = ANY ((ARRAY['code'::character varying, 'token'::character varying])::text[])))
);


--
-- Name: oauth_client_scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_client_scopes (
    client_id character varying(255) NOT NULL,
    scope text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: oauth_clients; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_clients (
    client_id text NOT NULL,
    client_secret_hash text,
    client_name character varying(255) NOT NULL,
    name character varying(255) DEFAULT NULL::character varying,
    token_endpoint_auth_method text DEFAULT 'client_secret_post'::text,
    application_type text DEFAULT 'web'::text NOT NULL,
    client_uri text,
    logo_uri text,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_used_at timestamp with time zone,
    owner_user_id text NOT NULL
);


--
-- Name: oauth_jti_revocations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_jti_revocations (
    jti text NOT NULL,
    user_id uuid NOT NULL,
    revoked_at timestamp with time zone DEFAULT now() NOT NULL,
    exp timestamp with time zone NOT NULL
);


--
-- Name: oauth_refresh_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_refresh_tokens (
    token_id text NOT NULL,
    client_id character varying(255) NOT NULL,
    user_id character varying(255) NOT NULL,
    scope text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    family_id text NOT NULL,
    consumed_at timestamp with time zone
);


--
-- Name: oauth_state_bindings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.oauth_state_bindings (
    state_token_hash text NOT NULL,
    return_to text NOT NULL,
    client_id text NOT NULL,
    redirect_uri text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    consumed_at timestamp with time zone
);


--
-- Name: plugin_env_vars; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plugin_env_vars (
    id text NOT NULL,
    user_id text NOT NULL,
    plugin_id text NOT NULL,
    var_name text NOT NULL,
    var_value text DEFAULT ''::text NOT NULL,
    is_secret boolean DEFAULT false NOT NULL,
    encrypted_value bytea,
    value_nonce bytea,
    key_version integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: plugin_session_summaries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plugin_session_summaries (
    id text NOT NULL,
    session_id text NOT NULL,
    user_id text NOT NULL,
    plugin_id text,
    started_at timestamp with time zone,
    ended_at timestamp with time zone,
    total_events bigint DEFAULT 0 NOT NULL,
    tool_uses bigint DEFAULT 0 NOT NULL,
    prompts bigint DEFAULT 0 NOT NULL,
    errors bigint DEFAULT 0 NOT NULL,
    total_input_tokens bigint DEFAULT 0,
    total_output_tokens bigint DEFAULT 0,
    model text,
    status text,
    unique_files_touched integer,
    content_input_bytes bigint DEFAULT 0 NOT NULL,
    content_output_bytes bigint DEFAULT 0 NOT NULL,
    ai_title text,
    ai_summary text,
    ai_tags text,
    ai_description text,
    apm real,
    eapm real,
    peak_concurrent integer,
    permission_mode text,
    client_source text,
    subagent_spawns bigint DEFAULT 0 NOT NULL,
    user_prompts integer,
    automated_actions integer,
    loc_added bigint DEFAULT 0 NOT NULL,
    loc_removed bigint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: plugin_usage_daily; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plugin_usage_daily (
    id text NOT NULL,
    date date NOT NULL,
    plugin_id text,
    event_type text NOT NULL,
    tool_name text,
    user_id text NOT NULL,
    event_count bigint DEFAULT 0 NOT NULL,
    total_duration_ms bigint DEFAULT 0,
    total_input_tokens bigint DEFAULT 0,
    total_output_tokens bigint DEFAULT 0,
    error_count bigint DEFAULT 0 NOT NULL,
    content_input_bytes bigint DEFAULT 0,
    content_output_bytes bigint DEFAULT 0,
    loc_added bigint DEFAULT 0 NOT NULL,
    loc_removed bigint DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: project_ad_mappings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.project_ad_mappings (
    ad_group text NOT NULL,
    project_id text NOT NULL,
    source text DEFAULT 'dashboard'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT project_ad_mappings_source_check CHECK ((source = ANY (ARRAY['yaml'::text, 'dashboard'::text])))
);


--
-- Name: project_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.project_members (
    project_id text NOT NULL,
    user_id text NOT NULL,
    source text NOT NULL,
    source_ad_group text,
    granted_by text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT project_members_source_check CHECK ((source = ANY (ARRAY['adfs'::text, 'manual'::text])))
);


--
-- Name: projects; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.projects (
    id text NOT NULL,
    name text NOT NULL,
    description text,
    source text DEFAULT 'dashboard'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT projects_id_check CHECK ((id ~ '^[a-z0-9][a-z0-9_-]{0,63}$'::text)),
    CONSTRAINT projects_source_check CHECK ((source = ANY (ARRAY['yaml'::text, 'dashboard'::text])))
);


--
-- Name: reviewed_production_failures; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reviewed_production_failures (
    id text NOT NULL,
    owner_id text NOT NULL,
    invocation_id text NOT NULL,
    reviewer_id text NOT NULL,
    sanitized_evidence jsonb NOT NULL,
    development_case_revision_id text CONSTRAINT reviewed_production_failure_development_case_revision__not_null NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: salesforce_user_identities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.salesforce_user_identities (
    user_id text NOT NULL,
    provider text DEFAULT 'salesforce'::text NOT NULL,
    sf_username text NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT salesforce_user_identities_provider_check CHECK ((provider ~ '^salesforce(-[A-Za-z0-9_-]{1,117})?$'::text))
);


--
-- Name: scheduled_jobs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.scheduled_jobs (
    id text NOT NULL,
    job_name text NOT NULL,
    schedule text NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    last_run timestamp with time zone,
    next_run timestamp with time zone,
    last_status text,
    last_error text,
    last_instance_id text,
    run_count integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: secret_audit_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.secret_audit_log (
    id text NOT NULL,
    user_id text NOT NULL,
    plugin_id text NOT NULL,
    var_name text NOT NULL,
    action text NOT NULL,
    actor_id text NOT NULL,
    ip_address text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT secret_audit_log_action_check CHECK ((action = ANY (ARRAY['created'::text, 'updated'::text, 'accessed'::text, 'rotated'::text, 'deleted'::text])))
);


--
-- Name: secret_resolution_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.secret_resolution_tokens (
    id text NOT NULL,
    user_id text NOT NULL,
    plugin_id text NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: services; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.services (
    instance_id text NOT NULL,
    name text NOT NULL,
    module_name text NOT NULL,
    server_type text DEFAULT 'internal'::text NOT NULL,
    pid integer,
    port integer NOT NULL,
    status text DEFAULT 'stopped'::text NOT NULL,
    binary_mtime bigint,
    heartbeat_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: session_analyses; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_analyses (
    session_id text NOT NULL,
    user_id text NOT NULL,
    title text DEFAULT ''::text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    summary text DEFAULT ''::text NOT NULL,
    tags text DEFAULT ''::text NOT NULL,
    goal_achieved text DEFAULT ''::text NOT NULL,
    quality_score smallint DEFAULT 0 NOT NULL,
    outcome text DEFAULT ''::text NOT NULL,
    error_analysis text,
    skill_assessment text,
    recommendations text,
    skill_scores jsonb,
    category text DEFAULT 'other'::text NOT NULL,
    goal_outcome_map jsonb,
    efficiency_metrics jsonb,
    best_practices_checklist jsonb,
    improvement_hints text,
    corrections_count integer DEFAULT 0 NOT NULL,
    session_duration_minutes integer,
    total_turns integer,
    automation_ratio real,
    plan_mode_used boolean DEFAULT false NOT NULL,
    client_surface text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: session_cost_snapshots; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_cost_snapshots (
    session_id text NOT NULL,
    user_id text NOT NULL,
    model text,
    total_cost_microdollars bigint,
    context_window_size bigint,
    input_tokens bigint,
    output_tokens bigint,
    cache_creation_input_tokens bigint,
    cache_read_input_tokens bigint,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: session_entity_links; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_entity_links (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    user_id text NOT NULL,
    session_id text NOT NULL,
    entity_type text NOT NULL,
    entity_name text NOT NULL,
    entity_id text,
    usage_count integer DEFAULT 1 NOT NULL,
    first_seen_at timestamp with time zone DEFAULT now() NOT NULL,
    last_seen_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: session_ratings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_ratings (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    user_id text NOT NULL,
    session_id text NOT NULL,
    rating smallint NOT NULL,
    outcome text DEFAULT ''::text NOT NULL,
    notes text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: session_transcripts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.session_transcripts (
    id text NOT NULL,
    user_id text NOT NULL,
    session_id text NOT NULL,
    plugin_id text,
    transcript jsonb DEFAULT '[]'::jsonb NOT NULL,
    total_input_tokens bigint DEFAULT 0,
    total_output_tokens bigint DEFAULT 0,
    model text,
    entries_counted integer DEFAULT 0,
    captured_at timestamp with time zone DEFAULT now() NOT NULL,
    search_tsv tsvector GENERATED ALWAYS AS (to_tsvector('english'::regconfig, "left"((transcript)::text, 262144))) STORED
);


--
-- Name: skill_invocation_events; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.skill_invocation_events AS
 WITH raw AS (
         SELECT e.user_id,
            e.session_id,
            e.plugin_id,
            "substring"(e.prompt_preview, '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)'::text) AS skill,
            NULL::text AS tool_use_id,
            'slash'::text AS source,
            e.created_at AS invoked_at
           FROM public.plugin_usage_events e
          WHERE ((e.event_type = 'UserPromptSubmit'::text) AND (e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'::text))
        UNION ALL
         SELECT e.user_id,
            e.session_id,
            e.plugin_id,
            ((e.metadata -> 'tool_input'::text) ->> 'skill'::text) AS skill,
            (e.metadata ->> 'tool_use_id'::text) AS tool_use_id,
            'tool'::text AS source,
            e.created_at AS invoked_at
           FROM public.plugin_usage_events e
          WHERE ((e.event_type = ANY (ARRAY['PostToolUse'::text, 'PostToolUseFailure'::text])) AND (e.tool_name = 'Skill'::text) AND (((e.metadata -> 'tool_input'::text) ->> 'skill'::text) IS NOT NULL) AND (EXISTS ( SELECT 1
                   FROM public.governance_decisions g
                  WHERE ((g.session_id = e.session_id) AND (g.tool_name = 'Skill'::text) AND ((g.created_at >= (e.created_at - '00:00:05'::interval)) AND (g.created_at <= (e.created_at + '00:00:05'::interval)))))))
        )
 SELECT user_id,
    session_id,
    plugin_id,
    skill,
    tool_use_id,
    source,
    invoked_at
   FROM ( SELECT raw.user_id,
            raw.session_id,
            raw.plugin_id,
            raw.skill,
            raw.tool_use_id,
            raw.source,
            raw.invoked_at,
            lag(raw.invoked_at) OVER (PARTITION BY raw.session_id, raw.skill ORDER BY raw.invoked_at) AS prev_at
           FROM raw) d
  WHERE ((prev_at IS NULL) OR ((invoked_at - prev_at) > '00:00:05'::interval));


--
-- Name: skill_ratings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.skill_ratings (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    user_id text NOT NULL,
    skill_name text NOT NULL,
    rating smallint NOT NULL,
    notes text DEFAULT ''::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: task_artifacts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.task_artifacts (
    id integer NOT NULL,
    task_id text NOT NULL,
    context_id text NOT NULL,
    artifact_id text NOT NULL,
    name text,
    description text,
    artifact_type text NOT NULL,
    source text,
    tool_name text,
    mcp_execution_id text,
    fingerprint text,
    skill_id text,
    skill_name text,
    metadata jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: task_artifacts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.task_artifacts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: task_artifacts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.task_artifacts_id_seq OWNED BY public.task_artifacts.id;


--
-- Name: task_execution_steps; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.task_execution_steps (
    step_id text NOT NULL,
    task_id text NOT NULL,
    step_type text NOT NULL,
    title text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    content jsonb DEFAULT '{}'::jsonb NOT NULL,
    started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    duration_ms integer,
    error_message text
);


--
-- Name: task_messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.task_messages (
    id integer NOT NULL,
    task_id text NOT NULL,
    message_id text NOT NULL,
    client_message_id text,
    role text NOT NULL,
    context_id text NOT NULL,
    user_id text,
    session_id text,
    trace_id text,
    sequence_number integer NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb,
    reference_task_ids text[],
    CONSTRAINT task_messages_role_check CHECK ((role = ANY (ARRAY['user'::text, 'agent'::text])))
);


--
-- Name: task_messages_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.task_messages_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: task_messages_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.task_messages_id_seq OWNED BY public.task_messages.id;


--
-- Name: tenant_activity; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tenant_activity (
    id integer NOT NULL,
    external_id text NOT NULL,
    tenant_id text NOT NULL,
    event_type text NOT NULL,
    user_id text,
    user_email text,
    event_source text,
    event_data jsonb,
    remote_created_at timestamp with time zone NOT NULL,
    synced_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: tenant_activity_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tenant_activity_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tenant_activity_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tenant_activity_id_seq OWNED BY public.tenant_activity.id;


--
-- Name: usage_anomalies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.usage_anomalies (
    metric text NOT NULL,
    window_start timestamp with time zone NOT NULL,
    observed bigint NOT NULL,
    baseline bigint NOT NULL,
    detected_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT usage_anomalies_metric_check CHECK ((metric = ANY (ARRAY['requests'::text, 'cost'::text, 'errors'::text])))
);


--
-- Name: user_activity; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    user_id text NOT NULL,
    category text NOT NULL,
    action text NOT NULL,
    entity_type text,
    entity_id text,
    entity_name text,
    description text NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_api_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_api_keys (
    id text NOT NULL,
    user_id text NOT NULL,
    name character varying(100) NOT NULL,
    key_prefix character varying(32) NOT NULL,
    key_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_used_at timestamp with time zone,
    expires_at timestamp with time zone,
    revoked_at timestamp with time zone
);


--
-- Name: user_commits; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_commits (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    user_id text NOT NULL,
    session_id text NOT NULL,
    cwd text,
    branch text,
    commit_hash text NOT NULL,
    message text DEFAULT ''::text NOT NULL,
    files_changed integer,
    insertions integer,
    deletions integer,
    committed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_contexts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_contexts (
    context_id text NOT NULL,
    user_id text NOT NULL,
    session_id text,
    name text NOT NULL,
    kind text DEFAULT 'user'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: user_device_certs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_device_certs (
    id text NOT NULL,
    user_id text NOT NULL,
    fingerprint character varying(128) NOT NULL,
    label character varying(100) NOT NULL,
    enrolled_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    revoked_at timestamp with time zone
);


--
-- Name: user_encryption_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_encryption_keys (
    id text NOT NULL,
    user_id text NOT NULL,
    encrypted_dek bytea NOT NULL,
    dek_nonce bytea NOT NULL,
    key_version integer DEFAULT 1 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    rotated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id text NOT NULL,
    name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    full_name character varying(255),
    display_name character varying(255),
    status text DEFAULT 'active'::text NOT NULL,
    email_verified boolean DEFAULT false NOT NULL,
    roles text[] DEFAULT ARRAY['user'::text] NOT NULL,
    is_bot boolean DEFAULT false NOT NULL,
    is_scanner boolean DEFAULT false NOT NULL,
    avatar_url text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT users_email_normalised CHECK (((email)::text = lower(TRIM(BOTH FROM email)))),
    CONSTRAINT users_status_check CHECK ((status = ANY (ARRAY['active'::text, 'inactive'::text, 'suspended'::text, 'pending'::text, 'deleted'::text, 'temporary'::text])))
);


--
-- Name: user_groups; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.user_groups AS
 SELECT DISTINCT gm.user_id,
    gm.group_id
   FROM public.group_members gm
UNION ALL
 SELECT u.id AS user_id,
    'unassigned'::text AS group_id
   FROM public.users u
  WHERE ((NOT ('anonymous'::text = ANY (u.roles))) AND (NOT (EXISTS ( SELECT 1
           FROM public.group_members gm
          WHERE (gm.user_id = u.id)))));


--
-- Name: user_manual_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_manual_roles (
    user_id text NOT NULL,
    role text NOT NULL,
    granted_by text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT user_manual_roles_role_check CHECK ((role = ANY (ARRAY['platform_admin'::text, 'admin'::text, 'developer'::text, 'user'::text, 'project_manager'::text, 'knowledge_worker'::text, 'super_admin'::text])))
);


--
-- Name: user_profile_ext; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_profile_ext (
    user_id text NOT NULL,
    share_token_version integer DEFAULT 1 NOT NULL
);


--
-- Name: user_profile_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_profile_reports (
    user_id text NOT NULL,
    archetype text DEFAULT ''::text NOT NULL,
    archetype_description text DEFAULT ''::text NOT NULL,
    archetype_confidence smallint DEFAULT 0 NOT NULL,
    strengths jsonb,
    weaknesses jsonb,
    ai_narrative text,
    ai_style_analysis text,
    ai_comparison text,
    ai_patterns text,
    ai_improvements text,
    ai_tips text,
    metrics_snapshot jsonb,
    period_days integer DEFAULT 30 NOT NULL,
    generated_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_rate_limit_buckets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_rate_limit_buckets (
    user_id character varying(255) NOT NULL,
    scope text NOT NULL,
    window_start timestamp with time zone NOT NULL,
    hits bigint DEFAULT 0 NOT NULL
);


--
-- Name: user_scope_defaults; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_scope_defaults (
    user_id text NOT NULL,
    primary_group_id text,
    primary_project_id text,
    source text DEFAULT 'auto'::text NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT user_scope_defaults_source_check CHECK ((source = ANY (ARRAY['auto'::text, 'manual'::text])))
);


--
-- Name: user_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_sessions (
    session_id text NOT NULL,
    user_id character varying(255),
    started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_activity_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ended_at timestamp with time zone,
    duration_seconds integer,
    user_type character varying(255) DEFAULT 'registered'::character varying,
    converted_at timestamp with time zone,
    expires_at timestamp with time zone DEFAULT (CURRENT_TIMESTAMP + '7 days'::interval),
    client_id character varying(255) DEFAULT 'sp_web'::character varying NOT NULL,
    client_type character varying(255) DEFAULT 'firstparty'::character varying NOT NULL,
    request_count integer DEFAULT 0 NOT NULL,
    avg_response_time_ms double precision DEFAULT 0 NOT NULL,
    success_rate double precision DEFAULT 1.0 NOT NULL,
    error_count integer DEFAULT 0 NOT NULL,
    task_count integer DEFAULT 0 NOT NULL,
    message_count integer DEFAULT 0 NOT NULL,
    ai_request_count integer DEFAULT 0 NOT NULL,
    total_tokens_used integer DEFAULT 0 NOT NULL,
    total_ai_cost_microdollars bigint DEFAULT 0 NOT NULL,
    ip_address text,
    user_agent text,
    device_type character varying(255),
    browser text,
    os text,
    country text,
    region text,
    city text,
    preferred_locale text,
    referrer_source character varying(255),
    referrer_url text,
    landing_page text,
    entry_url text,
    utm_source character varying(100),
    utm_medium character varying(100),
    utm_campaign character varying(100),
    utm_content character varying(100),
    utm_term character varying(100),
    endpoints_accessed text DEFAULT '[]'::text,
    fingerprint_hash text,
    is_bot boolean DEFAULT false NOT NULL,
    is_ai_crawler boolean DEFAULT false NOT NULL,
    is_scanner boolean DEFAULT false NOT NULL,
    is_behavioral_bot boolean DEFAULT false NOT NULL,
    behavioral_bot_reason text,
    behavioral_bot_score integer DEFAULT 0 NOT NULL,
    session_source character varying(50) DEFAULT 'web'::character varying,
    revoked_at timestamp with time zone,
    CONSTRAINT user_sessions_client_type_check CHECK (((client_type)::text = ANY ((ARRAY['cimd'::character varying, 'firstparty'::character varying, 'thirdparty'::character varying, 'system'::character varying, 'unknown'::character varying])::text[]))),
    CONSTRAINT user_sessions_session_source_check CHECK (((session_source)::text = ANY ((ARRAY['web'::character varying, 'api'::character varying, 'cli'::character varying, 'oauth'::character varying, 'mcp'::character varying, 'bridge'::character varying, 'unknown'::character varying])::text[]))),
    CONSTRAINT user_sessions_user_type_check CHECK (((user_type)::text = ANY ((ARRAY['anon'::character varying, 'registered'::character varying])::text[])))
);


--
-- Name: user_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_settings (
    user_id text NOT NULL,
    display_name text,
    avatar_url text,
    timezone text DEFAULT 'UTC'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: v_all_activity; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_all_activity AS
 SELECT a.id,
    a.user_id,
    COALESCE(u.display_name, u.full_name, u.name, u.email, (a.user_id)::character varying) AS display_name,
    a.category,
    a.action,
    a.entity_type,
    a.entity_name,
    a.description,
    a.created_at,
    ((NOT ('anonymous'::text = ANY (u.roles))) AND ((u.email)::text !~~ '%@anonymous.local'::text)) AS is_real_user
   FROM (public.user_activity a
     JOIN public.users u ON ((u.id = a.user_id)))
UNION ALL
 SELECT s.id,
    s.user_id,
    COALESCE(u2.display_name, u2.full_name, u2.name, u2.email, (s.user_id)::character varying) AS display_name,
    'session'::text AS category,
        CASE
            WHEN (s.ended_at IS NOT NULL) THEN 'completed'::text
            ELSE 'started'::text
        END AS action,
    'session'::text AS entity_type,
    s.session_id AS entity_name,
        CASE
            WHEN ((s.ended_at IS NOT NULL) AND (sa.title IS NOT NULL) AND (sa.title <> ''::text)) THEN sa.title
            WHEN ((s.ended_at IS NOT NULL) AND (s.ai_title IS NOT NULL) AND (s.ai_title <> ''::text)) THEN s.ai_title
            WHEN (s.ended_at IS NOT NULL) THEN concat('Completed AI session (', s.prompts, ' prompts, ', s.tool_uses, ' tool calls)')
            ELSE 'Started AI session'::text
        END AS description,
    COALESCE(s.ended_at, s.started_at, s.created_at) AS created_at,
    ((NOT ('anonymous'::text = ANY (u2.roles))) AND ((u2.email)::text !~~ '%@anonymous.local'::text)) AS is_real_user
   FROM ((public.plugin_session_summaries s
     JOIN public.users u2 ON ((u2.id = s.user_id)))
     LEFT JOIN public.session_analyses sa ON ((sa.session_id = s.session_id)))
UNION ALL
 SELECT r.id,
    r.user_id,
    COALESCE(u3.display_name, u3.full_name, u3.name, u3.email, (r.user_id)::character varying) AS display_name,
    'session_rated'::text AS category,
    'rated'::text AS action,
    'session'::text AS entity_type,
    r.session_id AS entity_name,
    concat('Rated session ', repeat('★'::text, (r.rating)::integer), repeat('☆'::text, (5 - (r.rating)::integer))) AS description,
    r.created_at,
    ((NOT ('anonymous'::text = ANY (u3.roles))) AND ((u3.email)::text !~~ '%@anonymous.local'::text)) AS is_real_user
   FROM (public.session_ratings r
     JOIN public.users u3 ON ((u3.id = r.user_id)));


--
-- Name: v_bot_sessions; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_bot_sessions AS
 SELECT session_id,
    user_id,
    started_at,
    last_activity_at,
    ended_at,
    duration_seconds,
    user_type,
    converted_at,
    expires_at,
    client_id,
    client_type,
    request_count,
    avg_response_time_ms,
    success_rate,
    error_count,
    task_count,
    message_count,
    ai_request_count,
    total_tokens_used,
    total_ai_cost_microdollars,
    ip_address,
    user_agent,
    device_type,
    browser,
    os,
    country,
    region,
    city,
    preferred_locale,
    referrer_source,
    referrer_url,
    landing_page,
    entry_url,
    utm_source,
    utm_medium,
    utm_campaign,
    utm_content,
    utm_term,
    endpoints_accessed,
    fingerprint_hash,
    is_bot,
    is_ai_crawler,
    is_scanner,
    is_behavioral_bot,
    behavioral_bot_reason,
    behavioral_bot_score,
    session_source,
    revoked_at,
        CASE
            WHEN ((user_agent ~~* '%googlebot%'::text) OR (user_agent ~~* '%google-inspectiontool%'::text) OR (user_agent ~~* '%adsbot-google%'::text)) THEN 'Google'::text
            WHEN ((user_agent ~~* '%bingbot%'::text) OR (user_agent ~~* '%bingpreview%'::text) OR (user_agent ~~* '%msnbot%'::text)) THEN 'Bing'::text
            WHEN ((user_agent ~~* '%chatgpt%'::text) OR (user_agent ~~* '%gptbot%'::text)) THEN 'OpenAI'::text
            WHEN ((user_agent ~~* '%claude%'::text) OR (user_agent ~~* '%anthropic%'::text)) THEN 'Anthropic'::text
            WHEN (user_agent ~~* '%perplexity%'::text) THEN 'Perplexity'::text
            WHEN (user_agent ~~* '%baiduspider%'::text) THEN 'Baidu'::text
            WHEN (user_agent ~~* '%yandexbot%'::text) THEN 'Yandex'::text
            WHEN ((user_agent ~~* '%facebookexternalhit%'::text) OR (user_agent ~~* '%facebot%'::text) OR (user_agent ~~* '%meta-externalagent%'::text)) THEN 'Meta'::text
            WHEN (user_agent ~~* '%twitterbot%'::text) THEN 'Twitter/X'::text
            WHEN (user_agent ~~* '%linkedinbot%'::text) THEN 'LinkedIn'::text
            WHEN ((user_agent ~~* '%semrushbot%'::text) OR (user_agent ~~* '%ahrefsbot%'::text) OR (user_agent ~~* '%mj12bot%'::text) OR (user_agent ~~* '%dotbot%'::text)) THEN 'SEO Crawlers'::text
            WHEN (user_agent ~~* '%bytespider%'::text) THEN 'ByteDance'::text
            WHEN ((user_agent ~~* '%amazonbot%'::text) OR (user_agent ~~* '%applebot%'::text)) THEN 'Tech Giants'::text
            WHEN ((user_agent ~~* '%python%'::text) OR (user_agent ~~* '%scrapy%'::text) OR (user_agent ~~* '%httpx%'::text)) THEN 'Python Scrapers'::text
            WHEN ((user_agent ~~* '%curl%'::text) OR (user_agent ~~* '%wget%'::text) OR (user_agent ~~* '%node-fetch%'::text) OR (user_agent ~~* '%axios%'::text)) THEN 'CLI/HTTP Tools'::text
            WHEN ((user_agent ~~* '%headless%'::text) OR (user_agent ~~* '%phantom%'::text) OR (user_agent ~~* '%selenium%'::text) OR (user_agent ~~* '%puppeteer%'::text)) THEN 'Headless Browsers'::text
            WHEN ((user_agent ~~* '%uptimerobot%'::text) OR (user_agent ~~* '%pingdom%'::text) OR (user_agent ~~* '%statuscake%'::text) OR (user_agent ~~* '%lighthouse%'::text)) THEN 'Monitoring'::text
            WHEN (is_ai_crawler = true) THEN 'AI Crawler'::text
            WHEN (is_behavioral_bot = true) THEN 'Behavioral Bot'::text
            WHEN (is_scanner = true) THEN 'Scanner'::text
            ELSE 'Other'::text
        END AS bot_type
   FROM public.user_sessions
  WHERE ((is_bot = true) OR (is_ai_crawler = true) OR (is_scanner = true) OR (is_behavioral_bot = true));


--
-- Name: v_clean_traffic; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_clean_traffic AS
 SELECT session_id,
    user_id,
    started_at,
    last_activity_at,
    ended_at,
    duration_seconds,
    user_type,
    converted_at,
    expires_at,
    client_id,
    client_type,
    request_count,
    avg_response_time_ms,
    success_rate,
    error_count,
    task_count,
    message_count,
    ai_request_count,
    total_tokens_used,
    total_ai_cost_microdollars,
    ip_address,
    user_agent,
    device_type,
    browser,
    os,
    country,
    region,
    city,
    preferred_locale,
    referrer_source,
    referrer_url,
    landing_page,
    entry_url,
    utm_source,
    utm_medium,
    utm_campaign,
    utm_content,
    utm_term,
    endpoints_accessed,
    fingerprint_hash,
    is_bot,
    is_ai_crawler,
    is_scanner,
    is_behavioral_bot,
    behavioral_bot_reason,
    behavioral_bot_score,
    session_source,
    revoked_at
   FROM public.user_sessions
  WHERE ((is_bot = false) AND (is_ai_crawler = false) AND (is_scanner = false) AND (is_behavioral_bot = false));


--
-- Name: v_engaged_traffic; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_engaged_traffic AS
 SELECT session_id,
    user_id,
    started_at,
    last_activity_at,
    ended_at,
    duration_seconds,
    user_type,
    converted_at,
    expires_at,
    client_id,
    client_type,
    request_count,
    avg_response_time_ms,
    success_rate,
    error_count,
    task_count,
    message_count,
    ai_request_count,
    total_tokens_used,
    total_ai_cost_microdollars,
    ip_address,
    user_agent,
    device_type,
    browser,
    os,
    country,
    region,
    city,
    preferred_locale,
    referrer_source,
    referrer_url,
    landing_page,
    entry_url,
    utm_source,
    utm_medium,
    utm_campaign,
    utm_content,
    utm_term,
    endpoints_accessed,
    fingerprint_hash,
    is_bot,
    is_ai_crawler,
    is_scanner,
    is_behavioral_bot,
    behavioral_bot_reason,
    behavioral_bot_score,
    session_source,
    revoked_at
   FROM public.user_sessions
  WHERE ((is_bot = false) AND (is_ai_crawler = false) AND (is_scanner = false) AND (is_behavioral_bot = false) AND (landing_page IS NOT NULL) AND (request_count > 0));


--
-- Name: webauthn_challenges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.webauthn_challenges (
    challenge text NOT NULL,
    user_id character varying(255),
    challenge_type character varying(255) NOT NULL,
    session_state jsonb,
    oauth_state text,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: webauthn_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.webauthn_credentials (
    id text NOT NULL,
    user_id character varying(255) NOT NULL,
    credential_id bytea NOT NULL,
    public_key bytea NOT NULL,
    counter integer DEFAULT 0 NOT NULL,
    display_name character varying(255) NOT NULL,
    device_type text DEFAULT 'platform'::text NOT NULL,
    transports text DEFAULT '["internal"]'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_used_at timestamp with time zone,
    CONSTRAINT webauthn_credentials_device_type_check CHECK ((device_type = ANY (ARRAY['platform'::text, 'cross-platform'::text])))
);


--
-- Name: webauthn_setup_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.webauthn_setup_tokens (
    id text NOT NULL,
    user_id character varying(255) NOT NULL,
    token_hash text NOT NULL,
    purpose character varying(50) DEFAULT 'credential_link'::character varying NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT webauthn_setup_tokens_purpose_check CHECK (((purpose)::text = ANY ((ARRAY['credential_link'::character varying, 'recovery'::character varying])::text[])))
);


--
-- Name: paddle_webhook_events id; Type: DEFAULT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.paddle_webhook_events ALTER COLUMN id SET DEFAULT nextval('marketplace.paddle_webhook_events_id_seq'::regclass);


--
-- Name: artifact_parts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.artifact_parts ALTER COLUMN id SET DEFAULT nextval('public.artifact_parts_id_seq'::regclass);


--
-- Name: content_files id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_files ALTER COLUMN id SET DEFAULT nextval('public.content_files_id_seq'::regclass);


--
-- Name: context_agents id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_agents ALTER COLUMN id SET DEFAULT nextval('public.context_agents_id_seq'::regclass);


--
-- Name: context_notifications id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_notifications ALTER COLUMN id SET DEFAULT nextval('public.context_notifications_id_seq'::regclass);


--
-- Name: message_parts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_parts ALTER COLUMN id SET DEFAULT nextval('public.message_parts_id_seq'::regclass);


--
-- Name: task_artifacts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_artifacts ALTER COLUMN id SET DEFAULT nextval('public.task_artifacts_id_seq'::regclass);


--
-- Name: task_messages id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_messages ALTER COLUMN id SET DEFAULT nextval('public.task_messages_id_seq'::regclass);


--
-- Name: tenant_activity id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_activity ALTER COLUMN id SET DEFAULT nextval('public.tenant_activity_id_seq'::regclass);


--
-- Name: paddle_customers paddle_customers_paddle_customer_id_key; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.paddle_customers
    ADD CONSTRAINT paddle_customers_paddle_customer_id_key UNIQUE (paddle_customer_id);


--
-- Name: paddle_customers paddle_customers_pkey; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.paddle_customers
    ADD CONSTRAINT paddle_customers_pkey PRIMARY KEY (id);


--
-- Name: paddle_customers paddle_customers_user_id_key; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.paddle_customers
    ADD CONSTRAINT paddle_customers_user_id_key UNIQUE (user_id);


--
-- Name: paddle_webhook_events paddle_webhook_events_event_id_key; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.paddle_webhook_events
    ADD CONSTRAINT paddle_webhook_events_event_id_key UNIQUE (event_id);


--
-- Name: paddle_webhook_events paddle_webhook_events_pkey; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.paddle_webhook_events
    ADD CONSTRAINT paddle_webhook_events_pkey PRIMARY KEY (id);


--
-- Name: plans plans_name_key; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.plans
    ADD CONSTRAINT plans_name_key UNIQUE (name);


--
-- Name: plans plans_pkey; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.plans
    ADD CONSTRAINT plans_pkey PRIMARY KEY (id);


--
-- Name: subscriptions subscriptions_paddle_subscription_id_key; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.subscriptions
    ADD CONSTRAINT subscriptions_paddle_subscription_id_key UNIQUE (paddle_subscription_id);


--
-- Name: subscriptions subscriptions_pkey; Type: CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.subscriptions
    ADD CONSTRAINT subscriptions_pkey PRIMARY KEY (id);


--
-- Name: access_control_entities access_control_entities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_control_entities
    ADD CONSTRAINT access_control_entities_pkey PRIMARY KEY (entity_type, entity_id);


--
-- Name: access_control_rules access_control_rules_entity_type_entity_id_rule_type_rule_v_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_control_rules
    ADD CONSTRAINT access_control_rules_entity_type_entity_id_rule_type_rule_v_key UNIQUE (entity_type, entity_id, rule_type, rule_value);


--
-- Name: access_control_rules access_control_rules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_control_rules
    ADD CONSTRAINT access_control_rules_pkey PRIMARY KEY (id);


--
-- Name: admin_traffic_reports admin_traffic_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_traffic_reports
    ADD CONSTRAINT admin_traffic_reports_pkey PRIMARY KEY (id);


--
-- Name: admin_traffic_reports admin_traffic_reports_report_date_report_period_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_traffic_reports
    ADD CONSTRAINT admin_traffic_reports_report_date_report_period_key UNIQUE (report_date, report_period);


--
-- Name: admin_usage_daily_rollups admin_usage_daily_rollups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.admin_usage_daily_rollups
    ADD CONSTRAINT admin_usage_daily_rollups_pkey PRIMARY KEY (user_id, date);


--
-- Name: agent_tasks agent_tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.agent_tasks
    ADD CONSTRAINT agent_tasks_pkey PRIMARY KEY (task_id);


--
-- Name: ai_gateway_policies ai_gateway_policies_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_gateway_policies
    ADD CONSTRAINT ai_gateway_policies_name_key UNIQUE (name);


--
-- Name: ai_gateway_policies ai_gateway_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_gateway_policies
    ADD CONSTRAINT ai_gateway_policies_pkey PRIMARY KEY (id);


--
-- Name: ai_gateway_thought_signatures ai_gateway_thought_signatures_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_gateway_thought_signatures
    ADD CONSTRAINT ai_gateway_thought_signatures_pkey PRIMARY KEY (user_id, conversation_id, tool_use_id);


--
-- Name: ai_quota_buckets ai_quota_buckets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_quota_buckets
    ADD CONSTRAINT ai_quota_buckets_pkey PRIMARY KEY (id);


--
-- Name: ai_quota_buckets ai_quota_buckets_subject_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_quota_buckets
    ADD CONSTRAINT ai_quota_buckets_subject_key UNIQUE (subject_kind, subject_id, window_seconds, window_start);


--
-- Name: ai_request_messages ai_request_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_messages
    ADD CONSTRAINT ai_request_messages_pkey PRIMARY KEY (id);


--
-- Name: ai_request_messages ai_request_messages_request_id_sequence_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_messages
    ADD CONSTRAINT ai_request_messages_request_id_sequence_number_key UNIQUE (request_id, sequence_number);


--
-- Name: ai_request_payloads ai_request_payloads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_payloads
    ADD CONSTRAINT ai_request_payloads_pkey PRIMARY KEY (ai_request_id);


--
-- Name: ai_request_tool_calls ai_request_tool_calls_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_tool_calls
    ADD CONSTRAINT ai_request_tool_calls_pkey PRIMARY KEY (id);


--
-- Name: ai_request_tool_calls ai_request_tool_calls_request_id_sequence_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_tool_calls
    ADD CONSTRAINT ai_request_tool_calls_request_id_sequence_number_key UNIQUE (request_id, sequence_number);


--
-- Name: ai_requests ai_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_requests
    ADD CONSTRAINT ai_requests_pkey PRIMARY KEY (id);


--
-- Name: ai_requests ai_requests_request_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_requests
    ADD CONSTRAINT ai_requests_request_id_key UNIQUE (request_id);


--
-- Name: ai_safety_findings ai_safety_findings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_safety_findings
    ADD CONSTRAINT ai_safety_findings_pkey PRIMARY KEY (id);


--
-- Name: analytics_events analytics_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.analytics_events
    ADD CONSTRAINT analytics_events_pkey PRIMARY KEY (id);


--
-- Name: anomaly_thresholds anomaly_thresholds_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.anomaly_thresholds
    ADD CONSTRAINT anomaly_thresholds_pkey PRIMARY KEY (metric_name);


--
-- Name: approval_requests approval_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.approval_requests
    ADD CONSTRAINT approval_requests_pkey PRIMARY KEY (call_id);


--
-- Name: artifact_parts artifact_parts_artifact_id_sequence_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.artifact_parts
    ADD CONSTRAINT artifact_parts_artifact_id_sequence_number_key UNIQUE (artifact_id, sequence_number);


--
-- Name: artifact_parts artifact_parts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.artifact_parts
    ADD CONSTRAINT artifact_parts_pkey PRIMARY KEY (id);


--
-- Name: banned_ips banned_ips_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.banned_ips
    ADD CONSTRAINT banned_ips_pkey PRIMARY KEY (ip_address);


--
-- Name: bridge_exchange_codes bridge_exchange_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_exchange_codes
    ADD CONSTRAINT bridge_exchange_codes_pkey PRIMARY KEY (code_hash);


--
-- Name: bridge_sessions bridge_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_sessions
    ADD CONSTRAINT bridge_sessions_pkey PRIMARY KEY (session_id);


--
-- Name: bridge_user_host_model_prefs bridge_user_host_model_prefs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_user_host_model_prefs
    ADD CONSTRAINT bridge_user_host_model_prefs_pkey PRIMARY KEY (user_id, host_id);


--
-- Name: bridge_user_host_prefs bridge_user_host_prefs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_user_host_prefs
    ADD CONSTRAINT bridge_user_host_prefs_pkey PRIMARY KEY (user_id, host_id);


--
-- Name: campaign_links campaign_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.campaign_links
    ADD CONSTRAINT campaign_links_pkey PRIMARY KEY (id);


--
-- Name: campaign_links campaign_links_short_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.campaign_links
    ADD CONSTRAINT campaign_links_short_code_key UNIQUE (short_code);


--
-- Name: content_files content_files_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_files
    ADD CONSTRAINT content_files_pkey PRIMARY KEY (id);


--
-- Name: content_files content_files_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_files
    ADD CONSTRAINT content_files_unique UNIQUE (content_id, file_id, role);


--
-- Name: content_performance_metrics content_performance_metrics_content_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_performance_metrics
    ADD CONSTRAINT content_performance_metrics_content_id_key UNIQUE (content_id);


--
-- Name: content_performance_metrics content_performance_metrics_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_performance_metrics
    ADD CONSTRAINT content_performance_metrics_pkey PRIMARY KEY (id);


--
-- Name: context_agents context_agents_context_id_agent_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_agents
    ADD CONSTRAINT context_agents_context_id_agent_name_key UNIQUE (context_id, agent_name);


--
-- Name: context_agents context_agents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_agents
    ADD CONSTRAINT context_agents_pkey PRIMARY KEY (id);


--
-- Name: context_notifications context_notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_notifications
    ADD CONSTRAINT context_notifications_pkey PRIMARY KEY (id);


--
-- Name: daily_summaries daily_summaries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_summaries
    ADD CONSTRAINT daily_summaries_pkey PRIMARY KEY (user_id, summary_date);


--
-- Name: dev_login_codes dev_login_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dev_login_codes
    ADD CONSTRAINT dev_login_codes_pkey PRIMARY KEY (code_hash);


--
-- Name: device_app_links device_app_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.device_app_links
    ADD CONSTRAINT device_app_links_pkey PRIMARY KEY (device_id);


--
-- Name: engagement_events engagement_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.engagement_events
    ADD CONSTRAINT engagement_events_pkey PRIMARY KEY (id);


--
-- Name: eval_approved_operation_receipts eval_approved_operation_receipts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_approved_operation_receipts
    ADD CONSTRAINT eval_approved_operation_receipts_pkey PRIMARY KEY (approval_id);


--
-- Name: eval_budget_accounts eval_budget_accounts_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_accounts
    ADD CONSTRAINT eval_budget_accounts_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: eval_budget_accounts eval_budget_accounts_owner_id_operation_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_accounts
    ADD CONSTRAINT eval_budget_accounts_owner_id_operation_key_key UNIQUE (owner_id, operation_key);


--
-- Name: eval_budget_accounts eval_budget_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_accounts
    ADD CONSTRAINT eval_budget_accounts_pkey PRIMARY KEY (id);


--
-- Name: eval_budget_reservations eval_budget_reservations_account_id_operation_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_reservations
    ADD CONSTRAINT eval_budget_reservations_account_id_operation_key_key UNIQUE (account_id, operation_key);


--
-- Name: eval_budget_reservations eval_budget_reservations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_reservations
    ADD CONSTRAINT eval_budget_reservations_pkey PRIMARY KEY (id);


--
-- Name: eval_budget_reservations eval_budget_reservations_request_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_reservations
    ADD CONSTRAINT eval_budget_reservations_request_id_key UNIQUE (request_id);


--
-- Name: eval_cases eval_cases_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_cases
    ADD CONSTRAINT eval_cases_pkey PRIMARY KEY (id);


--
-- Name: eval_execution_approvals eval_execution_approvals_execution_id_operation_digest_prec_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_approvals
    ADD CONSTRAINT eval_execution_approvals_execution_id_operation_digest_prec_key UNIQUE (execution_id, operation_digest, precondition_digest);


--
-- Name: eval_execution_approvals eval_execution_approvals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_approvals
    ADD CONSTRAINT eval_execution_approvals_pkey PRIMARY KEY (id);


--
-- Name: eval_execution_artifacts eval_execution_artifacts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_artifacts
    ADD CONSTRAINT eval_execution_artifacts_pkey PRIMARY KEY (execution_id);


--
-- Name: eval_execution_capabilities eval_execution_capabilities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_capabilities
    ADD CONSTRAINT eval_execution_capabilities_pkey PRIMARY KEY (token_hash);


--
-- Name: eval_execution_cleanup eval_execution_cleanup_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_cleanup
    ADD CONSTRAINT eval_execution_cleanup_pkey PRIMARY KEY (execution_id);


--
-- Name: eval_execution_events eval_execution_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_events
    ADD CONSTRAINT eval_execution_events_pkey PRIMARY KEY (execution_id, sequence);


--
-- Name: eval_execution_evidence eval_execution_evidence_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_evidence
    ADD CONSTRAINT eval_execution_evidence_pkey PRIMARY KEY (execution_id);


--
-- Name: eval_execution_measurements eval_execution_measurements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_measurements
    ADD CONSTRAINT eval_execution_measurements_pkey PRIMARY KEY (execution_id);


--
-- Name: eval_executions eval_executions_experiment_id_variant_index_case_revision_i_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_executions
    ADD CONSTRAINT eval_executions_experiment_id_variant_index_case_revision_i_key UNIQUE (experiment_id, variant_index, case_revision_id, repetition);


--
-- Name: eval_executions eval_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_executions
    ADD CONSTRAINT eval_executions_pkey PRIMARY KEY (id);


--
-- Name: eval_experiments eval_experiments_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_experiments
    ADD CONSTRAINT eval_experiments_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: eval_experiments eval_experiments_owner_id_idempotency_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_experiments
    ADD CONSTRAINT eval_experiments_owner_id_idempotency_key_key UNIQUE (owner_id, idempotency_key);


--
-- Name: eval_experiments eval_experiments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_experiments
    ADD CONSTRAINT eval_experiments_pkey PRIMARY KEY (id);


--
-- Name: eval_fixture_payloads eval_fixture_payloads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_fixture_payloads
    ADD CONSTRAINT eval_fixture_payloads_pkey PRIMARY KEY (owner_id, fixture_key, digest);


--
-- Name: eval_fixture_test_records eval_fixture_test_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_fixture_test_records
    ADD CONSTRAINT eval_fixture_test_records_pkey PRIMARY KEY (owner_id, record_key);


--
-- Name: eval_holdout_consumption eval_holdout_consumption_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_holdout_consumption
    ADD CONSTRAINT eval_holdout_consumption_pkey PRIMARY KEY (owner_id, case_revision_id);


--
-- Name: eval_managed_workspace_assets eval_managed_workspace_assets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_managed_workspace_assets
    ADD CONSTRAINT eval_managed_workspace_assets_pkey PRIMARY KEY (owner_id, workspace_digest, path);


--
-- Name: eval_managed_workspace_projections eval_managed_workspace_projections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_managed_workspace_projections
    ADD CONSTRAINT eval_managed_workspace_projections_pkey PRIMARY KEY (owner_id, digest);


--
-- Name: eval_request_reservations eval_request_reservations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_request_reservations
    ADD CONSTRAINT eval_request_reservations_pkey PRIMARY KEY (request_id);


--
-- Name: eval_request_reservations eval_request_reservations_reservation_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_request_reservations
    ADD CONSTRAINT eval_request_reservations_reservation_id_key UNIQUE (reservation_id);


--
-- Name: eval_resource_revisions eval_resource_revisions_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_resource_revisions
    ADD CONSTRAINT eval_resource_revisions_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: eval_resource_revisions eval_resource_revisions_owner_id_resource_kind_resource_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_resource_revisions
    ADD CONSTRAINT eval_resource_revisions_owner_id_resource_kind_resource_key_key UNIQUE (owner_id, resource_kind, resource_key, digest);


--
-- Name: eval_resource_revisions eval_resource_revisions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_resource_revisions
    ADD CONSTRAINT eval_resource_revisions_pkey PRIMARY KEY (id);


--
-- Name: eval_session_bindings eval_session_bindings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_session_bindings
    ADD CONSTRAINT eval_session_bindings_pkey PRIMARY KEY (session_id);


--
-- Name: eval_suggestions eval_suggestions_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_suggestions
    ADD CONSTRAINT eval_suggestions_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: eval_suggestions eval_suggestions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_suggestions
    ADD CONSTRAINT eval_suggestions_pkey PRIMARY KEY (id);


--
-- Name: eval_workers eval_workers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_workers
    ADD CONSTRAINT eval_workers_pkey PRIMARY KEY (id);


--
-- Name: eval_workers eval_workers_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_workers
    ADD CONSTRAINT eval_workers_token_hash_key UNIQUE (token_hash);


--
-- Name: event_outbox event_outbox_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.event_outbox
    ADD CONSTRAINT event_outbox_pkey PRIMARY KEY (id);


--
-- Name: extension_migrations extension_migrations_extension_id_version_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.extension_migrations
    ADD CONSTRAINT extension_migrations_extension_id_version_key UNIQUE (extension_id, version);


--
-- Name: extension_migrations extension_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.extension_migrations
    ADD CONSTRAINT extension_migrations_pkey PRIMARY KEY (id);


--
-- Name: federated_identities federated_identities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.federated_identities
    ADD CONSTRAINT federated_identities_pkey PRIMARY KEY (issuer, external_sub);


--
-- Name: files files_path_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_path_key UNIQUE (path);


--
-- Name: files files_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.files
    ADD CONSTRAINT files_pkey PRIMARY KEY (id);


--
-- Name: fingerprint_reputation fingerprint_reputation_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.fingerprint_reputation
    ADD CONSTRAINT fingerprint_reputation_pkey PRIMARY KEY (fingerprint_hash);


--
-- Name: funnel_progress funnel_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funnel_progress
    ADD CONSTRAINT funnel_progress_pkey PRIMARY KEY (id);


--
-- Name: funnel_steps funnel_steps_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funnel_steps
    ADD CONSTRAINT funnel_steps_pkey PRIMARY KEY (funnel_id, step_order);


--
-- Name: funnels funnels_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funnels
    ADD CONSTRAINT funnels_name_key UNIQUE (name);


--
-- Name: funnels funnels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funnels
    ADD CONSTRAINT funnels_pkey PRIMARY KEY (id);


--
-- Name: governance_decisions governance_decisions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.governance_decisions
    ADD CONSTRAINT governance_decisions_pkey PRIMARY KEY (id);


--
-- Name: group_ad_mappings group_ad_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.group_ad_mappings
    ADD CONSTRAINT group_ad_mappings_pkey PRIMARY KEY (ad_group, group_id);


--
-- Name: group_members group_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.group_members
    ADD CONSTRAINT group_members_pkey PRIMARY KEY (group_id, user_id, source);


--
-- Name: groups groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.groups
    ADD CONSTRAINT groups_pkey PRIMARY KEY (id);


--
-- Name: id_jag_replay id_jag_replay_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.id_jag_replay
    ADD CONSTRAINT id_jag_replay_pkey PRIMARY KEY (jti);


--
-- Name: ingestion_event_receipts ingestion_event_receipts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ingestion_event_receipts
    ADD CONSTRAINT ingestion_event_receipts_pkey PRIMARY KEY (dedup_key);


--
-- Name: ingestion_outbox ingestion_outbox_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ingestion_outbox
    ADD CONSTRAINT ingestion_outbox_pkey PRIMARY KEY (event_id);


--
-- Name: ingestion_repairs ingestion_repairs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ingestion_repairs
    ADD CONSTRAINT ingestion_repairs_pkey PRIMARY KEY (request_id);


--
-- Name: ingestion_session_owners ingestion_session_owners_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ingestion_session_owners
    ADD CONSTRAINT ingestion_session_owners_pkey PRIMARY KEY (session_id);


--
-- Name: link_clicks link_clicks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.link_clicks
    ADD CONSTRAINT link_clicks_pkey PRIMARY KEY (id);


--
-- Name: logs logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.logs
    ADD CONSTRAINT logs_pkey PRIMARY KEY (id);


--
-- Name: managed_assets managed_assets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_assets
    ADD CONSTRAINT managed_assets_pkey PRIMARY KEY (owner_id, digest);


--
-- Name: managed_distribution_deliveries managed_distribution_deliveries_outbox_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_deliveries
    ADD CONSTRAINT managed_distribution_deliveries_outbox_id_key UNIQUE (outbox_id);


--
-- Name: managed_distribution_deliveries managed_distribution_deliveries_owner_id_claim_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_deliveries
    ADD CONSTRAINT managed_distribution_deliveries_owner_id_claim_token_key UNIQUE (owner_id, claim_token);


--
-- Name: managed_distribution_deliveries managed_distribution_deliveries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_deliveries
    ADD CONSTRAINT managed_distribution_deliveries_pkey PRIMARY KEY (id);


--
-- Name: managed_distribution_outbox managed_distribution_outbox_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_outbox
    ADD CONSTRAINT managed_distribution_outbox_pkey PRIMARY KEY (id);


--
-- Name: managed_distribution_outbox managed_distribution_outbox_publication_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_outbox
    ADD CONSTRAINT managed_distribution_outbox_publication_id_key UNIQUE (publication_id);


--
-- Name: managed_installation_receipts managed_installation_receipts_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_installation_receipts managed_installation_receipts_owner_id_installation_id_reso_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_owner_id_installation_id_reso_key UNIQUE (owner_id, installation_id, resource_id, generation);


--
-- Name: managed_installation_receipts managed_installation_receipts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_pkey PRIMARY KEY (id);


--
-- Name: managed_invocation_attributions managed_invocation_attributions_owner_id_invocation_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_invocation_attributions
    ADD CONSTRAINT managed_invocation_attributions_owner_id_invocation_id_key UNIQUE (owner_id, invocation_id);


--
-- Name: managed_invocation_attributions managed_invocation_attributions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_invocation_attributions
    ADD CONSTRAINT managed_invocation_attributions_pkey PRIMARY KEY (id);


--
-- Name: managed_publication_reviews managed_publication_reviews_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_publication_reviews managed_publication_reviews_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_pkey PRIMARY KEY (id);


--
-- Name: managed_publication_selections managed_publication_selections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_selections
    ADD CONSTRAINT managed_publication_selections_pkey PRIMARY KEY (owner_id, resource_id);


--
-- Name: managed_publications managed_publications_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_publications managed_publications_owner_id_operation_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_operation_key_key UNIQUE (owner_id, operation_key);


--
-- Name: managed_publications managed_publications_owner_id_resource_id_generation_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_resource_id_generation_key UNIQUE (owner_id, resource_id, generation);


--
-- Name: managed_publications managed_publications_owner_id_resource_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_resource_id_id_key UNIQUE (owner_id, resource_id, id);


--
-- Name: managed_publications managed_publications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_pkey PRIMARY KEY (id);


--
-- Name: managed_reconciliation_conflicts managed_reconciliation_conflicts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliation_conflicts
    ADD CONSTRAINT managed_reconciliation_conflicts_pkey PRIMARY KEY (reconciliation_id, path);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_managed_candid_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_managed_candid_key UNIQUE (owner_id, resource_id, managed_candidate_revision_id, incoming_revision_id);


--
-- Name: managed_reconciliations managed_reconciliations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_pkey PRIMARY KEY (id);


--
-- Name: managed_resources managed_resources_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_resources
    ADD CONSTRAINT managed_resources_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_resources managed_resources_owner_id_kind_resource_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_resources
    ADD CONSTRAINT managed_resources_owner_id_kind_resource_key_key UNIQUE (owner_id, kind, resource_key);


--
-- Name: managed_resources managed_resources_owner_id_source_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_resources
    ADD CONSTRAINT managed_resources_owner_id_source_id_id_key UNIQUE (owner_id, source_id, id);


--
-- Name: managed_resources managed_resources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_resources
    ADD CONSTRAINT managed_resources_pkey PRIMARY KEY (id);


--
-- Name: managed_resources managed_resources_source_id_upstream_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_resources
    ADD CONSTRAINT managed_resources_source_id_upstream_key_key UNIQUE (source_id, upstream_key);


--
-- Name: managed_revision_assets managed_revision_assets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revision_assets
    ADD CONSTRAINT managed_revision_assets_pkey PRIMARY KEY (revision_id, path);


--
-- Name: managed_revision_dependencies managed_revision_dependencies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revision_dependencies
    ADD CONSTRAINT managed_revision_dependencies_pkey PRIMARY KEY (revision_id, dependency_id);


--
-- Name: managed_revisions managed_revisions_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_revisions managed_revisions_owner_id_resource_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_owner_id_resource_id_id_key UNIQUE (owner_id, resource_id, id);


--
-- Name: managed_revisions managed_revisions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_pkey PRIMARY KEY (id);


--
-- Name: managed_revisions managed_revisions_resource_id_digest_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_resource_id_digest_key UNIQUE (resource_id, digest);


--
-- Name: managed_source_snapshots managed_source_snapshots_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_source_snapshots
    ADD CONSTRAINT managed_source_snapshots_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_source_snapshots managed_source_snapshots_owner_id_source_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_source_snapshots
    ADD CONSTRAINT managed_source_snapshots_owner_id_source_id_id_key UNIQUE (owner_id, source_id, id);


--
-- Name: managed_source_snapshots managed_source_snapshots_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_source_snapshots
    ADD CONSTRAINT managed_source_snapshots_pkey PRIMARY KEY (id);


--
-- Name: managed_sources managed_sources_owner_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_sources
    ADD CONSTRAINT managed_sources_owner_id_id_key UNIQUE (owner_id, id);


--
-- Name: managed_sources managed_sources_owner_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_sources
    ADD CONSTRAINT managed_sources_owner_id_name_key UNIQUE (owner_id, name);


--
-- Name: managed_sources managed_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_sources
    ADD CONSTRAINT managed_sources_pkey PRIMARY KEY (id);


--
-- Name: managed_withdrawal_proposals managed_withdrawal_proposals_owner_id_resource_id_snapshot__key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_withdrawal_proposals
    ADD CONSTRAINT managed_withdrawal_proposals_owner_id_resource_id_snapshot__key UNIQUE (owner_id, resource_id, snapshot_id);


--
-- Name: managed_withdrawal_proposals managed_withdrawal_proposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_withdrawal_proposals
    ADD CONSTRAINT managed_withdrawal_proposals_pkey PRIMARY KEY (id);


--
-- Name: markdown_categories markdown_categories_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_categories
    ADD CONSTRAINT markdown_categories_name_key UNIQUE (name);


--
-- Name: markdown_categories markdown_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_categories
    ADD CONSTRAINT markdown_categories_pkey PRIMARY KEY (id);


--
-- Name: markdown_categories markdown_categories_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_categories
    ADD CONSTRAINT markdown_categories_slug_key UNIQUE (slug);


--
-- Name: markdown_content_enrichment markdown_content_enrichment_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_content_enrichment
    ADD CONSTRAINT markdown_content_enrichment_pkey PRIMARY KEY (content_id);


--
-- Name: markdown_content markdown_content_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_content
    ADD CONSTRAINT markdown_content_pkey PRIMARY KEY (id);


--
-- Name: markdown_fts markdown_fts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_fts
    ADD CONSTRAINT markdown_fts_pkey PRIMARY KEY (content_id);


--
-- Name: mcp_artifacts mcp_artifacts_artifact_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_artifacts
    ADD CONSTRAINT mcp_artifacts_artifact_id_key UNIQUE (artifact_id);


--
-- Name: mcp_artifacts mcp_artifacts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_artifacts
    ADD CONSTRAINT mcp_artifacts_pkey PRIMARY KEY (id);


--
-- Name: mcp_connector_accounts mcp_connector_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_connector_accounts
    ADD CONSTRAINT mcp_connector_accounts_pkey PRIMARY KEY (user_id, provider);


--
-- Name: mcp_connector_credentials mcp_connector_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_connector_credentials
    ADD CONSTRAINT mcp_connector_credentials_pkey PRIMARY KEY (user_id, provider);


--
-- Name: mcp_connector_oauth_states mcp_connector_oauth_states_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_connector_oauth_states
    ADD CONSTRAINT mcp_connector_oauth_states_pkey PRIMARY KEY (state);


--
-- Name: mcp_external_sessions mcp_external_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_external_sessions
    ADD CONSTRAINT mcp_external_sessions_pkey PRIMARY KEY (server_name, session_id);


--
-- Name: mcp_proxy_identities mcp_proxy_identities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_proxy_identities
    ADD CONSTRAINT mcp_proxy_identities_pkey PRIMARY KEY (session_id);


--
-- Name: mcp_sessions mcp_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_sessions
    ADD CONSTRAINT mcp_sessions_pkey PRIMARY KEY (session_id);


--
-- Name: mcp_tool_executions mcp_tool_executions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_tool_executions
    ADD CONSTRAINT mcp_tool_executions_pkey PRIMARY KEY (mcp_execution_id);


--
-- Name: mcp_tool_executions mcp_tool_executions_user_id_mcp_execution_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_tool_executions
    ADD CONSTRAINT mcp_tool_executions_user_id_mcp_execution_id_key UNIQUE (user_id, mcp_execution_id);


--
-- Name: message_parts message_parts_message_id_sequence_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_parts
    ADD CONSTRAINT message_parts_message_id_sequence_number_key UNIQUE (message_id, sequence_number);


--
-- Name: message_parts message_parts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_parts
    ADD CONSTRAINT message_parts_pkey PRIMARY KEY (id);


--
-- Name: oauth_auth_codes oauth_auth_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_auth_codes
    ADD CONSTRAINT oauth_auth_codes_pkey PRIMARY KEY (code);


--
-- Name: oauth_client_contacts oauth_client_contacts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_contacts
    ADD CONSTRAINT oauth_client_contacts_pkey PRIMARY KEY (client_id, contact_email);


--
-- Name: oauth_client_grant_types oauth_client_grant_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_grant_types
    ADD CONSTRAINT oauth_client_grant_types_pkey PRIMARY KEY (client_id, grant_type);


--
-- Name: oauth_client_redirect_uris oauth_client_redirect_uris_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_redirect_uris
    ADD CONSTRAINT oauth_client_redirect_uris_pkey PRIMARY KEY (client_id, redirect_uri);


--
-- Name: oauth_client_response_types oauth_client_response_types_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_response_types
    ADD CONSTRAINT oauth_client_response_types_pkey PRIMARY KEY (client_id, response_type);


--
-- Name: oauth_client_scopes oauth_client_scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_scopes
    ADD CONSTRAINT oauth_client_scopes_pkey PRIMARY KEY (client_id, scope);


--
-- Name: oauth_clients oauth_clients_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_clients
    ADD CONSTRAINT oauth_clients_pkey PRIMARY KEY (client_id);


--
-- Name: oauth_jti_revocations oauth_jti_revocations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_jti_revocations
    ADD CONSTRAINT oauth_jti_revocations_pkey PRIMARY KEY (jti);


--
-- Name: oauth_refresh_tokens oauth_refresh_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_refresh_tokens
    ADD CONSTRAINT oauth_refresh_tokens_pkey PRIMARY KEY (token_id);


--
-- Name: oauth_state_bindings oauth_state_bindings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_state_bindings
    ADD CONSTRAINT oauth_state_bindings_pkey PRIMARY KEY (state_token_hash);


--
-- Name: plugin_env_vars plugin_env_vars_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_env_vars
    ADD CONSTRAINT plugin_env_vars_pkey PRIMARY KEY (id);


--
-- Name: plugin_env_vars plugin_env_vars_user_id_plugin_id_var_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_env_vars
    ADD CONSTRAINT plugin_env_vars_user_id_plugin_id_var_name_key UNIQUE (user_id, plugin_id, var_name);


--
-- Name: plugin_session_summaries plugin_session_summaries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_session_summaries
    ADD CONSTRAINT plugin_session_summaries_pkey PRIMARY KEY (id);


--
-- Name: plugin_session_summaries plugin_session_summaries_session_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_session_summaries
    ADD CONSTRAINT plugin_session_summaries_session_id_key UNIQUE (session_id);


--
-- Name: plugin_usage_daily plugin_usage_daily_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_usage_daily
    ADD CONSTRAINT plugin_usage_daily_pkey PRIMARY KEY (id);


--
-- Name: plugin_usage_events plugin_usage_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_usage_events
    ADD CONSTRAINT plugin_usage_events_pkey PRIMARY KEY (id);


--
-- Name: plugin_usage_events plugin_usage_events_user_id_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plugin_usage_events
    ADD CONSTRAINT plugin_usage_events_user_id_id_key UNIQUE (user_id, id);


--
-- Name: project_ad_mappings project_ad_mappings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_ad_mappings
    ADD CONSTRAINT project_ad_mappings_pkey PRIMARY KEY (ad_group, project_id);


--
-- Name: project_members project_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_pkey PRIMARY KEY (project_id, user_id, source);


--
-- Name: projects projects_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_pkey PRIMARY KEY (id);


--
-- Name: reviewed_production_failures reviewed_production_failures_owner_id_invocation_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_owner_id_invocation_id_key UNIQUE (owner_id, invocation_id);


--
-- Name: reviewed_production_failures reviewed_production_failures_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_pkey PRIMARY KEY (id);


--
-- Name: salesforce_user_identities salesforce_user_identities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salesforce_user_identities
    ADD CONSTRAINT salesforce_user_identities_pkey PRIMARY KEY (user_id, provider);


--
-- Name: scheduled_jobs scheduled_jobs_job_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scheduled_jobs
    ADD CONSTRAINT scheduled_jobs_job_name_key UNIQUE (job_name);


--
-- Name: scheduled_jobs scheduled_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scheduled_jobs
    ADD CONSTRAINT scheduled_jobs_pkey PRIMARY KEY (id);


--
-- Name: secret_audit_log secret_audit_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.secret_audit_log
    ADD CONSTRAINT secret_audit_log_pkey PRIMARY KEY (id);


--
-- Name: secret_resolution_tokens secret_resolution_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.secret_resolution_tokens
    ADD CONSTRAINT secret_resolution_tokens_pkey PRIMARY KEY (id);


--
-- Name: secret_resolution_tokens secret_resolution_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.secret_resolution_tokens
    ADD CONSTRAINT secret_resolution_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: services services_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.services
    ADD CONSTRAINT services_pkey PRIMARY KEY (instance_id, name);


--
-- Name: session_analyses session_analyses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_analyses
    ADD CONSTRAINT session_analyses_pkey PRIMARY KEY (session_id);


--
-- Name: session_cost_snapshots session_cost_snapshots_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_cost_snapshots
    ADD CONSTRAINT session_cost_snapshots_pkey PRIMARY KEY (session_id);


--
-- Name: session_entity_links session_entity_links_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_entity_links
    ADD CONSTRAINT session_entity_links_pkey PRIMARY KEY (id);


--
-- Name: session_entity_links session_entity_links_user_id_session_id_entity_type_entity__key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_entity_links
    ADD CONSTRAINT session_entity_links_user_id_session_id_entity_type_entity__key UNIQUE (user_id, session_id, entity_type, entity_name);


--
-- Name: session_ratings session_ratings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_ratings
    ADD CONSTRAINT session_ratings_pkey PRIMARY KEY (id);


--
-- Name: session_ratings session_ratings_user_id_session_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_ratings
    ADD CONSTRAINT session_ratings_user_id_session_id_key UNIQUE (user_id, session_id);


--
-- Name: session_transcripts session_transcripts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.session_transcripts
    ADD CONSTRAINT session_transcripts_pkey PRIMARY KEY (id);


--
-- Name: skill_ratings skill_ratings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.skill_ratings
    ADD CONSTRAINT skill_ratings_pkey PRIMARY KEY (id);


--
-- Name: skill_ratings skill_ratings_user_id_skill_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.skill_ratings
    ADD CONSTRAINT skill_ratings_user_id_skill_name_key UNIQUE (user_id, skill_name);


--
-- Name: task_artifacts task_artifacts_context_id_artifact_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_artifacts
    ADD CONSTRAINT task_artifacts_context_id_artifact_id_key UNIQUE (context_id, artifact_id);


--
-- Name: task_artifacts task_artifacts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_artifacts
    ADD CONSTRAINT task_artifacts_pkey PRIMARY KEY (id);


--
-- Name: task_artifacts task_artifacts_task_id_artifact_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_artifacts
    ADD CONSTRAINT task_artifacts_task_id_artifact_id_key UNIQUE (task_id, artifact_id);


--
-- Name: task_execution_steps task_execution_steps_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_execution_steps
    ADD CONSTRAINT task_execution_steps_pkey PRIMARY KEY (step_id);


--
-- Name: task_messages task_messages_message_id_task_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_messages
    ADD CONSTRAINT task_messages_message_id_task_id_key UNIQUE (message_id, task_id);


--
-- Name: task_messages task_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_messages
    ADD CONSTRAINT task_messages_pkey PRIMARY KEY (id);


--
-- Name: task_messages task_messages_task_id_message_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_messages
    ADD CONSTRAINT task_messages_task_id_message_id_key UNIQUE (task_id, message_id);


--
-- Name: task_messages task_messages_task_id_sequence_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_messages
    ADD CONSTRAINT task_messages_task_id_sequence_number_key UNIQUE (task_id, sequence_number);


--
-- Name: tenant_activity tenant_activity_external_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_activity
    ADD CONSTRAINT tenant_activity_external_id_key UNIQUE (external_id);


--
-- Name: tenant_activity tenant_activity_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tenant_activity
    ADD CONSTRAINT tenant_activity_pkey PRIMARY KEY (id);


--
-- Name: usage_anomalies usage_anomalies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.usage_anomalies
    ADD CONSTRAINT usage_anomalies_pkey PRIMARY KEY (metric, window_start);


--
-- Name: user_activity user_activity_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity
    ADD CONSTRAINT user_activity_pkey PRIMARY KEY (id);


--
-- Name: user_api_keys user_api_keys_key_prefix_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_api_keys
    ADD CONSTRAINT user_api_keys_key_prefix_key UNIQUE (key_prefix);


--
-- Name: user_api_keys user_api_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_api_keys
    ADD CONSTRAINT user_api_keys_pkey PRIMARY KEY (id);


--
-- Name: user_commits user_commits_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_commits
    ADD CONSTRAINT user_commits_pkey PRIMARY KEY (id);


--
-- Name: user_contexts user_contexts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_contexts
    ADD CONSTRAINT user_contexts_pkey PRIMARY KEY (context_id);


--
-- Name: user_device_certs user_device_certs_fingerprint_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_device_certs
    ADD CONSTRAINT user_device_certs_fingerprint_key UNIQUE (fingerprint);


--
-- Name: user_device_certs user_device_certs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_device_certs
    ADD CONSTRAINT user_device_certs_pkey PRIMARY KEY (id);


--
-- Name: user_encryption_keys user_encryption_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_encryption_keys
    ADD CONSTRAINT user_encryption_keys_pkey PRIMARY KEY (id);


--
-- Name: user_encryption_keys user_encryption_keys_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_encryption_keys
    ADD CONSTRAINT user_encryption_keys_user_id_key UNIQUE (user_id);


--
-- Name: user_manual_roles user_manual_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_manual_roles
    ADD CONSTRAINT user_manual_roles_pkey PRIMARY KEY (user_id, role);


--
-- Name: user_profile_ext user_profile_ext_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_profile_ext
    ADD CONSTRAINT user_profile_ext_pkey PRIMARY KEY (user_id);


--
-- Name: user_profile_reports user_profile_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_profile_reports
    ADD CONSTRAINT user_profile_reports_pkey PRIMARY KEY (user_id);


--
-- Name: user_rate_limit_buckets user_rate_limit_buckets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_rate_limit_buckets
    ADD CONSTRAINT user_rate_limit_buckets_pkey PRIMARY KEY (user_id, scope, window_start);


--
-- Name: user_scope_defaults user_scope_defaults_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_scope_defaults
    ADD CONSTRAINT user_scope_defaults_pkey PRIMARY KEY (user_id);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (session_id);


--
-- Name: user_settings user_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_settings
    ADD CONSTRAINT user_settings_pkey PRIMARY KEY (user_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: webauthn_challenges webauthn_challenges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_challenges
    ADD CONSTRAINT webauthn_challenges_pkey PRIMARY KEY (challenge);


--
-- Name: webauthn_credentials webauthn_credentials_credential_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_credential_id_key UNIQUE (credential_id);


--
-- Name: webauthn_credentials webauthn_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_pkey PRIMARY KEY (id);


--
-- Name: webauthn_setup_tokens webauthn_setup_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_setup_tokens
    ADD CONSTRAINT webauthn_setup_tokens_pkey PRIMARY KEY (id);


--
-- Name: webauthn_setup_tokens webauthn_setup_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_setup_tokens
    ADD CONSTRAINT webauthn_setup_tokens_token_hash_key UNIQUE (token_hash);


--
-- Name: idx_mpc_paddle; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_mpc_paddle ON marketplace.paddle_customers USING btree (paddle_customer_id);


--
-- Name: idx_mpc_user; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_mpc_user ON marketplace.paddle_customers USING btree (user_id);


--
-- Name: idx_mpwe_event; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_mpwe_event ON marketplace.paddle_webhook_events USING btree (event_id);


--
-- Name: idx_mpwe_type; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_mpwe_type ON marketplace.paddle_webhook_events USING btree (event_type);


--
-- Name: idx_ms_paddle; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_ms_paddle ON marketplace.subscriptions USING btree (paddle_subscription_id);


--
-- Name: idx_ms_status; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_ms_status ON marketplace.subscriptions USING btree (status);


--
-- Name: idx_ms_user; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE INDEX idx_ms_user ON marketplace.subscriptions USING btree (user_id);


--
-- Name: idx_plans_role_name; Type: INDEX; Schema: marketplace; Owner: -
--

CREATE UNIQUE INDEX idx_plans_role_name ON marketplace.plans USING btree (role_name) WHERE (role_name IS NOT NULL);


--
-- Name: eval_capabilities_execution; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX eval_capabilities_execution ON public.eval_execution_capabilities USING btree (execution_id);


--
-- Name: eval_executions_queue; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX eval_executions_queue ON public.eval_executions USING btree (status, created_at);


--
-- Name: eval_experiments_owner_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX eval_experiments_owner_created ON public.eval_experiments USING btree (owner_id, created_at DESC);


--
-- Name: eval_resource_revisions_owner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX eval_resource_revisions_owner_id ON public.eval_resource_revisions USING btree (owner_id, id);


--
-- Name: idx_access_control_entities_default; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_control_entities_default ON public.access_control_entities USING btree (default_included) WHERE (default_included = true);


--
-- Name: idx_access_control_rules_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_control_rules_source ON public.access_control_rules USING btree (source);


--
-- Name: idx_acl_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_acl_entity ON public.access_control_rules USING btree (entity_type, entity_id);


--
-- Name: idx_acl_rule; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_acl_rule ON public.access_control_rules USING btree (rule_type, rule_value);


--
-- Name: idx_admin_traffic_reports_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_traffic_reports_date ON public.admin_traffic_reports USING btree (report_date DESC);


--
-- Name: idx_admin_usage_rollups_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_usage_rollups_date ON public.admin_usage_daily_rollups USING btree (date DESC);


--
-- Name: idx_admin_usage_rollups_group; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_usage_rollups_group ON public.admin_usage_daily_rollups USING btree (group_id, date DESC);


--
-- Name: idx_admin_usage_rollups_project; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_admin_usage_rollups_project ON public.admin_usage_daily_rollups USING btree (project_id, date DESC);


--
-- Name: idx_agent_tasks_agent_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_agent_name ON public.agent_tasks USING btree (agent_name);


--
-- Name: idx_agent_tasks_completed_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_completed_at ON public.agent_tasks USING btree (completed_at DESC);


--
-- Name: idx_agent_tasks_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_context_id ON public.agent_tasks USING btree (context_id);


--
-- Name: idx_agent_tasks_context_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_context_status ON public.agent_tasks USING btree (context_id, status);


--
-- Name: idx_agent_tasks_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_created_at ON public.agent_tasks USING btree (created_at);


--
-- Name: idx_agent_tasks_error_message; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_error_message ON public.agent_tasks USING btree (error_message) WHERE (error_message IS NOT NULL);


--
-- Name: idx_agent_tasks_execution_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_execution_time ON public.agent_tasks USING btree (execution_time_ms DESC);


--
-- Name: idx_agent_tasks_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_session_id ON public.agent_tasks USING btree (session_id);


--
-- Name: idx_agent_tasks_started_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_started_at ON public.agent_tasks USING btree (started_at DESC);


--
-- Name: idx_agent_tasks_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_status ON public.agent_tasks USING btree (status);


--
-- Name: idx_agent_tasks_status_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_status_timestamp ON public.agent_tasks USING btree (status_timestamp);


--
-- Name: idx_agent_tasks_trace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_trace_id ON public.agent_tasks USING btree (trace_id);


--
-- Name: idx_agent_tasks_updated_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_updated_at ON public.agent_tasks USING btree (updated_at);


--
-- Name: idx_agent_tasks_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_user_created ON public.agent_tasks USING btree (user_id, created_at);


--
-- Name: idx_agent_tasks_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_agent_tasks_user_id ON public.agent_tasks USING btree (user_id);


--
-- Name: idx_ai_gateway_policies_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_gateway_policies_enabled ON public.ai_gateway_policies USING btree (enabled);


--
-- Name: idx_ai_gateway_thought_signatures_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_gateway_thought_signatures_expires_at ON public.ai_gateway_thought_signatures USING btree (expires_at);


--
-- Name: idx_ai_quota_buckets_subject; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_quota_buckets_subject ON public.ai_quota_buckets USING btree (subject_kind, subject_id);


--
-- Name: idx_ai_quota_buckets_window; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_quota_buckets_window ON public.ai_quota_buckets USING btree (window_start);


--
-- Name: idx_ai_request_messages_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_messages_request_id ON public.ai_request_messages USING btree (request_id);


--
-- Name: idx_ai_request_messages_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_messages_role ON public.ai_request_messages USING btree (role);


--
-- Name: idx_ai_request_messages_sequence; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_messages_sequence ON public.ai_request_messages USING btree (request_id, sequence_number);


--
-- Name: idx_ai_request_payloads_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_payloads_created_at ON public.ai_request_payloads USING btree (created_at);


--
-- Name: idx_ai_request_tool_calls_ai_tool_call_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_tool_calls_ai_tool_call_id ON public.ai_request_tool_calls USING btree (ai_tool_call_id) WHERE (ai_tool_call_id IS NOT NULL);


--
-- Name: idx_ai_request_tool_calls_mcp_execution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_tool_calls_mcp_execution_id ON public.ai_request_tool_calls USING btree (mcp_execution_id);


--
-- Name: idx_ai_request_tool_calls_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_tool_calls_request_id ON public.ai_request_tool_calls USING btree (request_id);


--
-- Name: idx_ai_request_tool_calls_tool_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_tool_calls_tool_name ON public.ai_request_tool_calls USING btree (tool_name);


--
-- Name: idx_ai_requests_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_actor ON public.ai_requests USING btree (actor_kind, actor_id);


--
-- Name: idx_ai_requests_client_session_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_client_session_kind ON public.ai_requests USING btree (client_session_id, request_kind);


--
-- Name: idx_ai_requests_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_context_id ON public.ai_requests USING btree (context_id);


--
-- Name: idx_ai_requests_cost; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_cost ON public.ai_requests USING btree (cost_microdollars);


--
-- Name: idx_ai_requests_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_created_at ON public.ai_requests USING btree (created_at);


--
-- Name: idx_ai_requests_gateway_conversation_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_gateway_conversation_id ON public.ai_requests USING btree (gateway_conversation_id);


--
-- Name: idx_ai_requests_mcp_execution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_mcp_execution_id ON public.ai_requests USING btree (mcp_execution_id);


--
-- Name: idx_ai_requests_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_provider ON public.ai_requests USING btree (provider);


--
-- Name: idx_ai_requests_provider_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_provider_request_id ON public.ai_requests USING btree (provider_request_id);


--
-- Name: idx_ai_requests_provider_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_provider_status ON public.ai_requests USING btree (provider, status);


--
-- Name: idx_ai_requests_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_request_id ON public.ai_requests USING btree (request_id);


--
-- Name: idx_ai_requests_session_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_session_created ON public.ai_requests USING btree (session_id, created_at);


--
-- Name: idx_ai_requests_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_session_id ON public.ai_requests USING btree (session_id);


--
-- Name: idx_ai_requests_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_status ON public.ai_requests USING btree (status);


--
-- Name: idx_ai_requests_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_task_id ON public.ai_requests USING btree (task_id);


--
-- Name: idx_ai_requests_trace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_trace_id ON public.ai_requests USING btree (trace_id);


--
-- Name: idx_ai_requests_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_user_created ON public.ai_requests USING btree (user_id, created_at);


--
-- Name: idx_ai_requests_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_user_id ON public.ai_requests USING btree (user_id);


--
-- Name: idx_ai_requests_user_model; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_user_model ON public.ai_requests USING btree (user_id, model);


--
-- Name: idx_ai_safety_findings_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_safety_findings_category ON public.ai_safety_findings USING btree (category);


--
-- Name: idx_ai_safety_findings_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_safety_findings_created_at ON public.ai_safety_findings USING btree (created_at);


--
-- Name: idx_ai_safety_findings_request; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_safety_findings_request ON public.ai_safety_findings USING btree (ai_request_id);


--
-- Name: idx_ai_safety_findings_severity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_safety_findings_severity ON public.ai_safety_findings USING btree (severity);


--
-- Name: idx_analytics_events_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_context_id ON public.analytics_events USING btree (context_id);


--
-- Name: idx_analytics_events_event_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_event_category ON public.analytics_events USING btree (event_category);


--
-- Name: idx_analytics_events_event_data; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_event_data ON public.analytics_events USING gin (event_data);


--
-- Name: idx_analytics_events_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_event_type ON public.analytics_events USING btree (event_type);


--
-- Name: idx_analytics_events_gateway_conversation_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_gateway_conversation_id ON public.analytics_events USING btree (gateway_conversation_id);


--
-- Name: idx_analytics_events_provider_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_provider_request_id ON public.analytics_events USING btree (provider_request_id);


--
-- Name: idx_analytics_events_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_session_id ON public.analytics_events USING btree (session_id);


--
-- Name: idx_analytics_events_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_timestamp ON public.analytics_events USING btree ("timestamp");


--
-- Name: idx_analytics_events_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analytics_events_user_id ON public.analytics_events USING btree (user_id);


--
-- Name: idx_approval_requests_requester; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_approval_requests_requester ON public.approval_requests USING btree (requested_by);


--
-- Name: idx_approval_requests_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_approval_requests_status ON public.approval_requests USING btree (status, created_at DESC);


--
-- Name: idx_approval_requests_trace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_approval_requests_trace ON public.approval_requests USING btree (trace_id);


--
-- Name: idx_artifact_parts_artifact_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_parts_artifact_id ON public.artifact_parts USING btree (artifact_id);


--
-- Name: idx_artifact_parts_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_parts_context_id ON public.artifact_parts USING btree (context_id);


--
-- Name: idx_artifact_parts_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_parts_kind ON public.artifact_parts USING btree (part_kind);


--
-- Name: idx_artifact_parts_sequence; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_parts_sequence ON public.artifact_parts USING btree (artifact_id, sequence_number);


--
-- Name: idx_atr_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_atr_date ON public.admin_traffic_reports USING btree (report_date DESC, generated_at DESC);


--
-- Name: idx_auth_codes_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_codes_client ON public.oauth_auth_codes USING btree (client_id);


--
-- Name: idx_auth_codes_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_codes_expires ON public.oauth_auth_codes USING btree (expires_at);


--
-- Name: idx_auth_codes_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_codes_lookup ON public.oauth_auth_codes USING btree (code, expires_at);


--
-- Name: idx_auth_codes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_auth_codes_user ON public.oauth_auth_codes USING btree (user_id);


--
-- Name: idx_banned_ips_banned_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_banned_ips_banned_at ON public.banned_ips USING btree (banned_at);


--
-- Name: idx_banned_ips_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_banned_ips_expires ON public.banned_ips USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_banned_ips_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_banned_ips_fingerprint ON public.banned_ips USING btree (source_fingerprint) WHERE (source_fingerprint IS NOT NULL);


--
-- Name: idx_banned_ips_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_banned_ips_source ON public.banned_ips USING btree (ban_source);


--
-- Name: idx_bridge_exchange_codes_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bridge_exchange_codes_active ON public.bridge_exchange_codes USING btree (code_hash) WHERE (consumed_at IS NULL);


--
-- Name: idx_bridge_exchange_codes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bridge_exchange_codes_user ON public.bridge_exchange_codes USING btree (user_id);


--
-- Name: idx_bridge_sessions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bridge_sessions_active ON public.bridge_sessions USING btree (last_heartbeat_at DESC);


--
-- Name: idx_bridge_sessions_user_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bridge_sessions_user_active ON public.bridge_sessions USING btree (user_id, last_heartbeat_at DESC);


--
-- Name: idx_bridge_user_host_model_prefs_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bridge_user_host_model_prefs_user ON public.bridge_user_host_model_prefs USING btree (user_id);


--
-- Name: idx_bridge_user_host_prefs_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_bridge_user_host_prefs_user ON public.bridge_user_host_prefs USING btree (user_id);


--
-- Name: idx_campaign_links_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_active ON public.campaign_links USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_campaign_links_campaign_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_campaign_id ON public.campaign_links USING btree (campaign_id);


--
-- Name: idx_campaign_links_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_created ON public.campaign_links USING btree (created_at DESC);


--
-- Name: idx_campaign_links_short_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_short_code ON public.campaign_links USING btree (short_code);


--
-- Name: idx_campaign_links_source_content; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_source_content ON public.campaign_links USING btree (source_content_id);


--
-- Name: idx_campaign_links_target_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_target_url ON public.campaign_links USING btree (target_url);


--
-- Name: idx_content_files_content_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_files_content_id ON public.content_files USING btree (content_id);


--
-- Name: idx_content_files_file_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_files_file_id ON public.content_files USING btree (file_id);


--
-- Name: idx_content_files_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_files_role ON public.content_files USING btree (role);


--
-- Name: idx_content_performance_metrics_content_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_performance_metrics_content_id ON public.content_performance_metrics USING btree (content_id);


--
-- Name: idx_content_performance_metrics_total_views; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_performance_metrics_total_views ON public.content_performance_metrics USING btree (total_views DESC);


--
-- Name: idx_content_performance_metrics_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_performance_metrics_updated ON public.content_performance_metrics USING btree (updated_at DESC);


--
-- Name: idx_content_performance_metrics_views_7d; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_performance_metrics_views_7d ON public.content_performance_metrics USING btree (views_last_7_days DESC);


--
-- Name: idx_context_agents_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_context_agents_active ON public.context_agents USING btree (context_id, last_active_at DESC);


--
-- Name: idx_context_agents_agent_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_context_agents_agent_name ON public.context_agents USING btree (agent_name);


--
-- Name: idx_context_agents_context; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_context_agents_context ON public.context_agents USING btree (context_id);


--
-- Name: idx_daily_summaries_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_daily_summaries_date ON public.daily_summaries USING btree (summary_date DESC);


--
-- Name: idx_daily_summaries_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_daily_summaries_user ON public.daily_summaries USING btree (user_id, summary_date DESC);


--
-- Name: idx_dev_login_codes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_dev_login_codes_user ON public.dev_login_codes USING btree (user_id);


--
-- Name: idx_device_app_links_last_seen; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_app_links_last_seen ON public.device_app_links USING btree (last_seen_at DESC NULLS LAST);


--
-- Name: idx_device_app_links_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_device_app_links_user ON public.device_app_links USING btree (user_id);


--
-- Name: idx_engagement_events_content_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_content_id ON public.engagement_events USING btree (content_id);


--
-- Name: idx_engagement_events_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_created ON public.engagement_events USING btree (created_at);


--
-- Name: idx_engagement_events_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_event_type ON public.engagement_events USING btree (event_type);


--
-- Name: idx_engagement_events_scroll_depth; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_scroll_depth ON public.engagement_events USING btree (max_scroll_depth);


--
-- Name: idx_engagement_events_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_session ON public.engagement_events USING btree (session_id);


--
-- Name: idx_engagement_events_time_on_page; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_time_on_page ON public.engagement_events USING btree (time_on_page_ms);


--
-- Name: idx_engagement_events_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_engagement_events_user ON public.engagement_events USING btree (user_id);


--
-- Name: idx_eval_cases_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_eval_cases_enabled ON public.eval_cases USING btree (enabled);


--
-- Name: idx_eval_cases_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_eval_cases_source ON public.eval_cases USING btree (source_ai_request_id);


--
-- Name: idx_event_outbox_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_actor ON public.event_outbox USING btree (actor_kind, actor_id);


--
-- Name: idx_event_outbox_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_created_at ON public.event_outbox USING btree (created_at);


--
-- Name: idx_extension_migrations_ext_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_extension_migrations_ext_id ON public.extension_migrations USING btree (extension_id);


--
-- Name: idx_extension_migrations_ext_version; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_extension_migrations_ext_version ON public.extension_migrations USING btree (extension_id, version);


--
-- Name: idx_federated_identities_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_federated_identities_user ON public.federated_identities USING btree (user_id);


--
-- Name: idx_files_ai_content; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_files_ai_content ON public.files USING btree (ai_content) WHERE (deleted_at IS NULL);


--
-- Name: idx_files_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_files_context_id ON public.files USING btree (context_id) WHERE ((context_id IS NOT NULL) AND (deleted_at IS NULL));


--
-- Name: idx_files_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_files_created_at ON public.files USING btree (created_at DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_files_metadata; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_files_metadata ON public.files USING gin (metadata);


--
-- Name: idx_files_mime_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_files_mime_type ON public.files USING btree (mime_type) WHERE (deleted_at IS NULL);


--
-- Name: idx_files_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_files_user_id ON public.files USING btree (user_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_fingerprint_reputation_abuse; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fingerprint_reputation_abuse ON public.fingerprint_reputation USING btree (abuse_incidents) WHERE (abuse_incidents > 0);


--
-- Name: idx_fingerprint_reputation_flagged; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fingerprint_reputation_flagged ON public.fingerprint_reputation USING btree (is_flagged) WHERE (is_flagged = true);


--
-- Name: idx_fingerprint_reputation_last_seen; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fingerprint_reputation_last_seen ON public.fingerprint_reputation USING btree (last_seen_at);


--
-- Name: idx_fingerprint_reputation_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fingerprint_reputation_score ON public.fingerprint_reputation USING btree (reputation_score);


--
-- Name: idx_fingerprint_reputation_session_count; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_fingerprint_reputation_session_count ON public.fingerprint_reputation USING btree (total_session_count);


--
-- Name: idx_funnel_progress_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funnel_progress_created_at ON public.funnel_progress USING btree (created_at);


--
-- Name: idx_funnel_progress_funnel_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funnel_progress_funnel_id ON public.funnel_progress USING btree (funnel_id);


--
-- Name: idx_funnel_progress_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funnel_progress_session_id ON public.funnel_progress USING btree (session_id);


--
-- Name: idx_funnel_progress_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_funnel_progress_unique ON public.funnel_progress USING btree (funnel_id, session_id);


--
-- Name: idx_funnel_steps_funnel_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funnel_steps_funnel_id ON public.funnel_steps USING btree (funnel_id);


--
-- Name: idx_funnels_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_funnels_active ON public.funnels USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_governance_decisions_act_chain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_act_chain ON public.governance_decisions USING gin (act_chain);


--
-- Name: idx_governance_decisions_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_actor ON public.governance_decisions USING btree (actor_kind, actor_id);


--
-- Name: idx_governance_decisions_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_client ON public.governance_decisions USING btree (client_id);


--
-- Name: idx_governance_decisions_context; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_context ON public.governance_decisions USING btree (context_id);


--
-- Name: idx_governance_decisions_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_created ON public.governance_decisions USING btree (created_at);


--
-- Name: idx_governance_decisions_decision; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_decision ON public.governance_decisions USING btree (decision);


--
-- Name: idx_governance_decisions_policy_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_policy_created ON public.governance_decisions USING btree (policy, created_at);


--
-- Name: idx_governance_decisions_rate_limit; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_rate_limit ON public.governance_decisions USING btree (session_id, user_id, created_at DESC);


--
-- Name: idx_governance_decisions_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_session ON public.governance_decisions USING btree (session_id);


--
-- Name: idx_governance_decisions_tool_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_tool_name ON public.governance_decisions USING btree (tool_name);


--
-- Name: idx_governance_decisions_trace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_trace ON public.governance_decisions USING btree (trace_id);


--
-- Name: idx_governance_decisions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_user ON public.governance_decisions USING btree (user_id);


--
-- Name: idx_group_members_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_members_user ON public.group_members USING btree (user_id);


--
-- Name: idx_id_jag_replay_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_id_jag_replay_expires_at ON public.id_jag_replay USING btree (expires_at);


--
-- Name: idx_link_clicks_clicked_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_clicked_at ON public.link_clicks USING btree (clicked_at DESC);


--
-- Name: idx_link_clicks_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_context_id ON public.link_clicks USING btree (context_id);


--
-- Name: idx_link_clicks_conversion; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_conversion ON public.link_clicks USING btree (is_conversion) WHERE (is_conversion = true);


--
-- Name: idx_link_clicks_link_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_link_id ON public.link_clicks USING btree (link_id);


--
-- Name: idx_link_clicks_link_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_link_session ON public.link_clicks USING btree (link_id, session_id);


--
-- Name: idx_link_clicks_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_session_id ON public.link_clicks USING btree (session_id);


--
-- Name: idx_link_clicks_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_task_id ON public.link_clicks USING btree (task_id);


--
-- Name: idx_link_clicks_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_link_clicks_user_id ON public.link_clicks USING btree (user_id);


--
-- Name: idx_logs_client_level; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_client_level ON public.logs USING btree (client_id, level);


--
-- Name: idx_logs_client_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_client_timestamp ON public.logs USING btree (client_id, "timestamp" DESC);


--
-- Name: idx_logs_context_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_context_timestamp ON public.logs USING btree (context_id, "timestamp" DESC);


--
-- Name: idx_logs_gateway_conversation_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_gateway_conversation_id ON public.logs USING btree (gateway_conversation_id);


--
-- Name: idx_logs_level_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_level_timestamp ON public.logs USING btree (level, "timestamp" DESC);


--
-- Name: idx_logs_module; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_module ON public.logs USING btree (module);


--
-- Name: idx_logs_provider_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_provider_request_id ON public.logs USING btree (provider_request_id);


--
-- Name: idx_logs_session_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_session_timestamp ON public.logs USING btree (session_id, "timestamp" DESC);


--
-- Name: idx_logs_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_task_id ON public.logs USING btree (task_id);


--
-- Name: idx_logs_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_timestamp ON public.logs USING btree ("timestamp" DESC);


--
-- Name: idx_logs_trace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_trace_id ON public.logs USING btree (trace_id);


--
-- Name: idx_logs_user_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_logs_user_timestamp ON public.logs USING btree (user_id, "timestamp" DESC);


--
-- Name: idx_markdown_categories_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_categories_parent ON public.markdown_categories USING btree (parent_id);


--
-- Name: idx_markdown_categories_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_categories_slug ON public.markdown_categories USING btree (slug);


--
-- Name: idx_markdown_content_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_category ON public.markdown_content USING btree (category_id);


--
-- Name: idx_markdown_content_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_kind ON public.markdown_content USING btree (kind);


--
-- Name: idx_markdown_content_links; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_links ON public.markdown_content USING gin (links);


--
-- Name: idx_markdown_content_locale; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_locale ON public.markdown_content USING btree (locale);


--
-- Name: idx_markdown_content_public; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_public ON public.markdown_content USING btree (public) WHERE (public = true);


--
-- Name: idx_markdown_content_published; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_published ON public.markdown_content USING btree (published_at DESC);


--
-- Name: idx_markdown_content_slug_locale; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_markdown_content_slug_locale ON public.markdown_content USING btree (slug, locale);


--
-- Name: idx_markdown_content_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_source ON public.markdown_content USING btree (source_id);


--
-- Name: idx_markdown_content_version_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_content_version_hash ON public.markdown_content USING btree (version_hash);


--
-- Name: idx_markdown_fts_search; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_markdown_fts_search ON public.markdown_fts USING gin (search_vector);


--
-- Name: idx_mce_after_reading_this; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mce_after_reading_this ON public.markdown_content_enrichment USING gin (after_reading_this);


--
-- Name: idx_mce_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mce_category ON public.markdown_content_enrichment USING btree (category);


--
-- Name: idx_mce_related_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mce_related_code ON public.markdown_content_enrichment USING gin (related_code);


--
-- Name: idx_mce_related_docs; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mce_related_docs ON public.markdown_content_enrichment USING gin (related_docs);


--
-- Name: idx_mce_related_playbooks; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mce_related_playbooks ON public.markdown_content_enrichment USING gin (related_playbooks);


--
-- Name: idx_mcp_artifacts_artifact_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_artifact_id ON public.mcp_artifacts USING btree (artifact_id);


--
-- Name: idx_mcp_artifacts_artifact_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_artifact_type ON public.mcp_artifacts USING btree (artifact_type);


--
-- Name: idx_mcp_artifacts_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_context_id ON public.mcp_artifacts USING btree (context_id) WHERE (context_id IS NOT NULL);


--
-- Name: idx_mcp_artifacts_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_created_at ON public.mcp_artifacts USING btree (created_at DESC);


--
-- Name: idx_mcp_artifacts_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_expires_at ON public.mcp_artifacts USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_mcp_artifacts_mcp_execution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_mcp_execution_id ON public.mcp_artifacts USING btree (mcp_execution_id);


--
-- Name: idx_mcp_artifacts_server_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_server_created ON public.mcp_artifacts USING btree (server_name, created_at DESC);


--
-- Name: idx_mcp_artifacts_server_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_server_name ON public.mcp_artifacts USING btree (server_name);


--
-- Name: idx_mcp_artifacts_type_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_type_created ON public.mcp_artifacts USING btree (artifact_type, created_at DESC);


--
-- Name: idx_mcp_artifacts_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_user_id ON public.mcp_artifacts USING btree (user_id) WHERE (user_id IS NOT NULL);


--
-- Name: idx_mcp_external_sessions_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_external_sessions_expires_at ON public.mcp_external_sessions USING btree (expires_at);


--
-- Name: idx_mcp_proxy_identities_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_proxy_identities_expires_at ON public.mcp_proxy_identities USING btree (expires_at);


--
-- Name: idx_mcp_sessions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_sessions_active ON public.mcp_sessions USING btree (status) WHERE ((status)::text = 'active'::text);


--
-- Name: idx_mcp_sessions_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_sessions_expires_at ON public.mcp_sessions USING btree (expires_at);


--
-- Name: idx_mcp_sessions_last_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_sessions_last_activity ON public.mcp_sessions USING btree (last_activity_at);


--
-- Name: idx_mcp_sessions_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_sessions_status ON public.mcp_sessions USING btree (status);


--
-- Name: idx_mcp_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_sessions_user_id ON public.mcp_sessions USING btree (user_id);


--
-- Name: idx_mcp_tool_executions_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_actor ON public.mcp_tool_executions USING btree (actor_kind, actor_id);


--
-- Name: idx_mcp_tool_executions_ai_tool_call_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_mcp_tool_executions_ai_tool_call_id ON public.mcp_tool_executions USING btree (ai_tool_call_id) WHERE (ai_tool_call_id IS NOT NULL);


--
-- Name: idx_mcp_tool_executions_completed_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_completed_at ON public.mcp_tool_executions USING btree (completed_at DESC);


--
-- Name: idx_mcp_tool_executions_context_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_context_created ON public.mcp_tool_executions USING btree (context_id, created_at DESC);


--
-- Name: idx_mcp_tool_executions_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_context_id ON public.mcp_tool_executions USING btree (context_id);


--
-- Name: idx_mcp_tool_executions_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_created_at ON public.mcp_tool_executions USING btree (created_at DESC);


--
-- Name: idx_mcp_tool_executions_execution_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_execution_time ON public.mcp_tool_executions USING btree (execution_time_ms DESC);


--
-- Name: idx_mcp_tool_executions_mcp_execution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_mcp_execution_id ON public.mcp_tool_executions USING btree (mcp_execution_id);


--
-- Name: idx_mcp_tool_executions_server_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_server_name ON public.mcp_tool_executions USING btree (server_name);


--
-- Name: idx_mcp_tool_executions_server_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_server_status ON public.mcp_tool_executions USING btree (server_name, status);


--
-- Name: idx_mcp_tool_executions_server_tool; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_server_tool ON public.mcp_tool_executions USING btree (server_name, tool_name);


--
-- Name: idx_mcp_tool_executions_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_session_id ON public.mcp_tool_executions USING btree (session_id);


--
-- Name: idx_mcp_tool_executions_session_tool; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_session_tool ON public.mcp_tool_executions USING btree (session_id, tool_name);


--
-- Name: idx_mcp_tool_executions_started_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_started_at ON public.mcp_tool_executions USING btree (started_at DESC);


--
-- Name: idx_mcp_tool_executions_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_status ON public.mcp_tool_executions USING btree (status);


--
-- Name: idx_mcp_tool_executions_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_task_id ON public.mcp_tool_executions USING btree (task_id);


--
-- Name: idx_mcp_tool_executions_tool_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_tool_name ON public.mcp_tool_executions USING btree (tool_name);


--
-- Name: idx_mcp_tool_executions_tool_started; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_tool_started ON public.mcp_tool_executions USING btree (tool_name, started_at DESC);


--
-- Name: idx_mcp_tool_executions_tool_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_tool_status ON public.mcp_tool_executions USING btree (tool_name, status);


--
-- Name: idx_mcp_tool_executions_trace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_trace_id ON public.mcp_tool_executions USING btree (trace_id);


--
-- Name: idx_mcp_tool_executions_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_user_created ON public.mcp_tool_executions USING btree (user_id, created_at DESC);


--
-- Name: idx_mcp_tool_executions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_user_id ON public.mcp_tool_executions USING btree (user_id);


--
-- Name: idx_message_parts_file_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_file_id ON public.message_parts USING btree (file_id) WHERE (file_id IS NOT NULL);


--
-- Name: idx_message_parts_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_kind ON public.message_parts USING btree (part_kind);


--
-- Name: idx_message_parts_message_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_message_id ON public.message_parts USING btree (message_id);


--
-- Name: idx_message_parts_sequence; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_sequence ON public.message_parts USING btree (message_id, sequence_number);


--
-- Name: idx_message_parts_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_task_id ON public.message_parts USING btree (task_id);


--
-- Name: idx_notifications_agent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_agent ON public.context_notifications USING btree (agent_id, received_at DESC);


--
-- Name: idx_notifications_context; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_context ON public.context_notifications USING btree (context_id, received_at DESC);


--
-- Name: idx_notifications_not_broadcasted; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_not_broadcasted ON public.context_notifications USING btree (broadcasted) WHERE (broadcasted = false);


--
-- Name: idx_notifications_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_type ON public.context_notifications USING btree (notification_type);


--
-- Name: idx_oauth_auth_codes_refresh_token_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_auth_codes_refresh_token_id ON public.oauth_auth_codes USING btree (refresh_token_id);


--
-- Name: idx_oauth_client_contacts_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_contacts_client_id ON public.oauth_client_contacts USING btree (client_id);


--
-- Name: idx_oauth_client_grant_types_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_grant_types_client_id ON public.oauth_client_grant_types USING btree (client_id);


--
-- Name: idx_oauth_client_grant_types_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_grant_types_type ON public.oauth_client_grant_types USING btree (grant_type);


--
-- Name: idx_oauth_client_redirect_uris_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_redirect_uris_client_id ON public.oauth_client_redirect_uris USING btree (client_id);


--
-- Name: idx_oauth_client_response_types_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_response_types_client_id ON public.oauth_client_response_types USING btree (client_id);


--
-- Name: idx_oauth_client_scopes_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_scopes_client_id ON public.oauth_client_scopes USING btree (client_id);


--
-- Name: idx_oauth_client_scopes_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_scopes_scope ON public.oauth_client_scopes USING btree (scope);


--
-- Name: idx_oauth_clients_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_clients_active ON public.oauth_clients USING btree (is_active);


--
-- Name: idx_oauth_clients_owner_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_clients_owner_user_id ON public.oauth_clients USING btree (owner_user_id);


--
-- Name: idx_oauth_refresh_tokens_consumed_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_refresh_tokens_consumed_at ON public.oauth_refresh_tokens USING btree (consumed_at);


--
-- Name: idx_oauth_refresh_tokens_family_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_refresh_tokens_family_id ON public.oauth_refresh_tokens USING btree (family_id);


--
-- Name: idx_plugin_env_user_plugin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_plugin_env_user_plugin ON public.plugin_env_vars USING btree (user_id, plugin_id);


--
-- Name: idx_plugin_usage_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_plugin_usage_created_at ON public.plugin_usage_events USING btree (created_at DESC);


--
-- Name: idx_plugin_usage_dedup; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_plugin_usage_dedup ON public.plugin_usage_events USING btree (dedup_key) WHERE (dedup_key IS NOT NULL);


--
-- Name: idx_plugin_usage_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_plugin_usage_event_type ON public.plugin_usage_events USING btree (event_type);


--
-- Name: idx_plugin_usage_session_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_plugin_usage_session_created ON public.plugin_usage_events USING btree (session_id, created_at);


--
-- Name: idx_plugin_usage_tool_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_plugin_usage_tool_name ON public.plugin_usage_events USING btree (tool_name) WHERE (tool_name IS NOT NULL);


--
-- Name: idx_plugin_usage_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_plugin_usage_user ON public.plugin_usage_events USING btree (user_id, created_at DESC);


--
-- Name: idx_project_members_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_project_members_user ON public.project_members USING btree (user_id);


--
-- Name: idx_refresh_tokens_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_tokens_lookup ON public.oauth_refresh_tokens USING btree (token_id, expires_at);


--
-- Name: idx_scheduled_jobs_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scheduled_jobs_enabled ON public.scheduled_jobs USING btree (enabled);


--
-- Name: idx_scheduled_jobs_job_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scheduled_jobs_job_name ON public.scheduled_jobs USING btree (job_name);


--
-- Name: idx_scheduled_jobs_next_run; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scheduled_jobs_next_run ON public.scheduled_jobs USING btree (next_run);


--
-- Name: idx_secret_audit_log_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_secret_audit_log_user_created ON public.secret_audit_log USING btree (user_id, created_at DESC);


--
-- Name: idx_secret_audit_log_user_plugin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_secret_audit_log_user_plugin ON public.secret_audit_log USING btree (user_id, plugin_id);


--
-- Name: idx_secret_resolution_tokens_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_secret_resolution_tokens_expires ON public.secret_resolution_tokens USING btree (expires_at);


--
-- Name: idx_secret_resolution_tokens_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_secret_resolution_tokens_hash ON public.secret_resolution_tokens USING btree (token_hash);


--
-- Name: idx_sel_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sel_entity ON public.session_entity_links USING btree (entity_type, entity_name);


--
-- Name: idx_sel_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sel_session ON public.session_entity_links USING btree (session_id);


--
-- Name: idx_sel_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sel_user ON public.session_entity_links USING btree (user_id);


--
-- Name: idx_services_heartbeat; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_services_heartbeat ON public.services USING btree (heartbeat_at);


--
-- Name: idx_services_module; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_services_module ON public.services USING btree (module_name);


--
-- Name: idx_services_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_services_status ON public.services USING btree (status);


--
-- Name: idx_session_analyses_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_analyses_user ON public.session_analyses USING btree (user_id, created_at DESC);


--
-- Name: idx_session_cost_snapshots_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_cost_snapshots_user ON public.session_cost_snapshots USING btree (user_id, updated_at DESC);


--
-- Name: idx_session_summary_mode; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_summary_mode ON public.plugin_session_summaries USING btree (user_id, permission_mode);


--
-- Name: idx_session_summary_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_summary_session ON public.plugin_session_summaries USING btree (session_id);


--
-- Name: idx_session_summary_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_summary_source ON public.plugin_session_summaries USING btree (user_id, client_source);


--
-- Name: idx_session_summary_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_summary_user ON public.plugin_session_summaries USING btree (user_id, started_at DESC);


--
-- Name: idx_session_transcripts_fts; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_transcripts_fts ON public.session_transcripts USING gin (search_tsv);


--
-- Name: idx_session_transcripts_jsonb; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_transcripts_jsonb ON public.session_transcripts USING gin (transcript jsonb_path_ops);


--
-- Name: idx_session_transcripts_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_transcripts_session ON public.session_transcripts USING btree (session_id, captured_at DESC);


--
-- Name: idx_session_transcripts_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_session_transcripts_user ON public.session_transcripts USING btree (user_id, captured_at DESC);


--
-- Name: idx_sessions_ai_usage; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_ai_usage ON public.user_sessions USING btree (ai_request_count);


--
-- Name: idx_sessions_bot_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_bot_time ON public.user_sessions USING btree (is_bot, started_at);


--
-- Name: idx_sessions_cost; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_cost ON public.user_sessions USING btree (total_ai_cost_microdollars);


--
-- Name: idx_sessions_country; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_country ON public.user_sessions USING btree (country);


--
-- Name: idx_sessions_device_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_device_type ON public.user_sessions USING btree (device_type);


--
-- Name: idx_sessions_engagement; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_engagement ON public.user_sessions USING btree (duration_seconds, request_count, is_bot);


--
-- Name: idx_sessions_entry; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_entry ON public.user_sessions USING btree (entry_url, is_bot);


--
-- Name: idx_sessions_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_fingerprint ON public.user_sessions USING btree (fingerprint_hash);


--
-- Name: idx_sessions_fingerprint_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_fingerprint_activity ON public.user_sessions USING btree (fingerprint_hash, last_activity_at);


--
-- Name: idx_sessions_fingerprint_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_fingerprint_time ON public.user_sessions USING btree (fingerprint_hash, started_at) WHERE (is_bot = false);


--
-- Name: idx_sessions_landing; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_landing ON public.user_sessions USING btree (landing_page, is_bot);


--
-- Name: idx_sessions_last_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_last_activity ON public.user_sessions USING btree (last_activity_at);


--
-- Name: idx_sessions_quality; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_quality ON public.user_sessions USING btree (success_rate, error_count, is_bot);


--
-- Name: idx_sessions_referrer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_referrer ON public.user_sessions USING btree (referrer_source, started_at) WHERE (is_bot = false);


--
-- Name: idx_sessions_started_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_started_at ON public.user_sessions USING btree (started_at);


--
-- Name: idx_sessions_started_bot; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_started_bot ON public.user_sessions USING btree (started_at DESC, is_bot);


--
-- Name: idx_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_user_id ON public.user_sessions USING btree (user_id);


--
-- Name: idx_sessions_user_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_user_time ON public.user_sessions USING btree (user_id, started_at) WHERE (is_bot = false);


--
-- Name: idx_sessions_utm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_utm ON public.user_sessions USING btree (utm_source, utm_campaign, utm_medium, started_at) WHERE (is_bot = false);


--
-- Name: idx_skr_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_skr_user ON public.skill_ratings USING btree (user_id);


--
-- Name: idx_sr_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sr_session ON public.session_ratings USING btree (session_id);


--
-- Name: idx_sr_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sr_user ON public.session_ratings USING btree (user_id);


--
-- Name: idx_task_artifacts_artifact_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_artifact_id ON public.task_artifacts USING btree (artifact_id);


--
-- Name: idx_task_artifacts_artifact_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_artifact_type ON public.task_artifacts USING btree (artifact_type);


--
-- Name: idx_task_artifacts_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_context_id ON public.task_artifacts USING btree (context_id);


--
-- Name: idx_task_artifacts_context_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_context_type ON public.task_artifacts USING btree (context_id, artifact_type);


--
-- Name: idx_task_artifacts_fingerprint; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_fingerprint ON public.task_artifacts USING btree (fingerprint);


--
-- Name: idx_task_artifacts_mcp_execution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_mcp_execution_id ON public.task_artifacts USING btree (mcp_execution_id);


--
-- Name: idx_task_artifacts_skill_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_skill_id ON public.task_artifacts USING btree (skill_id);


--
-- Name: idx_task_artifacts_skill_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_skill_name ON public.task_artifacts USING btree (skill_name);


--
-- Name: idx_task_artifacts_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_task_id ON public.task_artifacts USING btree (task_id);


--
-- Name: idx_task_artifacts_tool_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_artifacts_tool_name ON public.task_artifacts USING btree (tool_name);


--
-- Name: idx_task_execution_steps_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_execution_steps_status ON public.task_execution_steps USING btree (status);


--
-- Name: idx_task_execution_steps_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_execution_steps_task_id ON public.task_execution_steps USING btree (task_id);


--
-- Name: idx_task_messages_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_client_id ON public.task_messages USING btree (client_message_id) WHERE (client_message_id IS NOT NULL);


--
-- Name: idx_task_messages_message_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_message_id ON public.task_messages USING btree (message_id);


--
-- Name: idx_task_messages_sequence; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_sequence ON public.task_messages USING btree (task_id, sequence_number);


--
-- Name: idx_task_messages_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_session_id ON public.task_messages USING btree (session_id);


--
-- Name: idx_task_messages_task_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_task_id ON public.task_messages USING btree (task_id);


--
-- Name: idx_task_messages_trace_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_trace_id ON public.task_messages USING btree (trace_id);


--
-- Name: idx_task_messages_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_user_id ON public.task_messages USING btree (user_id);


--
-- Name: idx_tenant_activity_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tenant_activity_event_type ON public.tenant_activity USING btree (event_type);


--
-- Name: idx_tenant_activity_remote_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tenant_activity_remote_created_at ON public.tenant_activity USING btree (remote_created_at DESC);


--
-- Name: idx_tenant_activity_sync; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tenant_activity_sync ON public.tenant_activity USING btree (remote_created_at, synced_at);


--
-- Name: idx_tenant_activity_tenant_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_tenant_activity_tenant_id ON public.tenant_activity USING btree (tenant_id);


--
-- Name: idx_usage_anomalies_detected; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_anomalies_detected ON public.usage_anomalies USING btree (detected_at DESC);


--
-- Name: idx_usage_daily_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_daily_date ON public.plugin_usage_daily USING btree (date DESC);


--
-- Name: idx_usage_daily_plugin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_daily_plugin ON public.plugin_usage_daily USING btree (plugin_id, date DESC);


--
-- Name: idx_usage_daily_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_usage_daily_unique ON public.plugin_usage_daily USING btree (date, user_id, event_type, COALESCE(tool_name, ''::text));


--
-- Name: idx_usage_daily_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_daily_user ON public.plugin_usage_daily USING btree (user_id, date DESC);


--
-- Name: idx_user_activity_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_activity_category ON public.user_activity USING btree (category, created_at DESC);


--
-- Name: idx_user_activity_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_activity_created ON public.user_activity USING btree (created_at DESC);


--
-- Name: idx_user_activity_mcp_access; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_activity_mcp_access ON public.user_activity USING btree (category, created_at DESC) WHERE (category = 'mcp_access'::text);


--
-- Name: idx_user_activity_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_activity_user ON public.user_activity USING btree (user_id, created_at DESC);


--
-- Name: idx_user_api_keys_prefix_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_api_keys_prefix_active ON public.user_api_keys USING btree (key_prefix) WHERE (revoked_at IS NULL);


--
-- Name: idx_user_api_keys_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_api_keys_user ON public.user_api_keys USING btree (user_id);


--
-- Name: idx_user_commits_day; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_commits_day ON public.user_commits USING btree (committed_at);


--
-- Name: idx_user_commits_dedup; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_user_commits_dedup ON public.user_commits USING btree (user_id, COALESCE(cwd, ''::text), commit_hash);


--
-- Name: idx_user_commits_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_commits_user ON public.user_commits USING btree (user_id, committed_at DESC);


--
-- Name: idx_user_contexts_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_contexts_session ON public.user_contexts USING btree (session_id);


--
-- Name: idx_user_contexts_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_contexts_user ON public.user_contexts USING btree (user_id);


--
-- Name: idx_user_contexts_user_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_contexts_user_updated ON public.user_contexts USING btree (user_id, updated_at DESC);


--
-- Name: idx_user_device_certs_fingerprint_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_device_certs_fingerprint_active ON public.user_device_certs USING btree (fingerprint) WHERE (revoked_at IS NULL);


--
-- Name: idx_user_device_certs_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_device_certs_user ON public.user_device_certs USING btree (user_id);


--
-- Name: idx_user_encryption_keys_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_encryption_keys_user_id ON public.user_encryption_keys USING btree (user_id);


--
-- Name: idx_user_profile_reports_generated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_profile_reports_generated ON public.user_profile_reports USING btree (generated_at DESC);


--
-- Name: idx_user_rate_limit_buckets_window; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_rate_limit_buckets_window ON public.user_rate_limit_buckets USING btree (window_start);


--
-- Name: idx_user_scope_defaults_group; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_scope_defaults_group ON public.user_scope_defaults USING btree (primary_group_id);


--
-- Name: idx_user_scope_defaults_project; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_scope_defaults_project ON public.user_scope_defaults USING btree (primary_project_id);


--
-- Name: idx_user_sessions_behavioral_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_behavioral_score ON public.user_sessions USING btree (behavioral_bot_score) WHERE (behavioral_bot_score >= 50);


--
-- Name: idx_user_sessions_bot_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_bot_activity ON public.user_sessions USING btree (is_bot, started_at) WHERE (is_bot = true);


--
-- Name: idx_user_sessions_clean_traffic; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_clean_traffic ON public.user_sessions USING btree (started_at) WHERE ((is_bot = false) AND (is_ai_crawler = false) AND (is_scanner = false) AND (is_behavioral_bot = false));


--
-- Name: idx_user_sessions_client_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_client_activity ON public.user_sessions USING btree (client_id, last_activity_at);


--
-- Name: idx_user_sessions_client_cost; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_client_cost ON public.user_sessions USING btree (client_id, total_ai_cost_microdollars);


--
-- Name: idx_user_sessions_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_client_id ON public.user_sessions USING btree (client_id);


--
-- Name: idx_user_sessions_client_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_client_type ON public.user_sessions USING btree (client_type);


--
-- Name: idx_user_sessions_converted; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_converted ON public.user_sessions USING btree (converted_at);


--
-- Name: idx_user_sessions_engaged_traffic; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_engaged_traffic ON public.user_sessions USING btree (started_at) WHERE ((is_bot = false) AND (is_ai_crawler = false) AND (is_scanner = false) AND (is_behavioral_bot = false) AND (landing_page IS NOT NULL) AND (request_count > 0));


--
-- Name: idx_user_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_expires ON public.user_sessions USING btree (expires_at);


--
-- Name: idx_user_sessions_human_activity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_human_activity ON public.user_sessions USING btree (is_bot, last_activity_at) WHERE (is_bot = false);


--
-- Name: idx_user_sessions_human_sessions; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_human_sessions ON public.user_sessions USING btree (is_bot, started_at, user_id) WHERE (is_bot = false);


--
-- Name: idx_user_sessions_is_ai_crawler; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_is_ai_crawler ON public.user_sessions USING btree (is_ai_crawler) WHERE (is_ai_crawler = true);


--
-- Name: idx_user_sessions_is_behavioral_bot; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_is_behavioral_bot ON public.user_sessions USING btree (is_behavioral_bot);


--
-- Name: idx_user_sessions_is_bot; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_is_bot ON public.user_sessions USING btree (is_bot);


--
-- Name: idx_user_sessions_is_scanner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_is_scanner ON public.user_sessions USING btree (is_scanner);


--
-- Name: idx_user_sessions_landing_page; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_landing_page ON public.user_sessions USING btree (landing_page);


--
-- Name: idx_user_sessions_referrer_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_referrer_source ON public.user_sessions USING btree (referrer_source);


--
-- Name: idx_user_sessions_revoked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_revoked ON public.user_sessions USING btree (revoked_at) WHERE (revoked_at IS NOT NULL);


--
-- Name: idx_user_sessions_session_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_session_source ON public.user_sessions USING btree (session_source);


--
-- Name: idx_user_sessions_user_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_user_type ON public.user_sessions USING btree (user_type);


--
-- Name: idx_user_sessions_utm_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_utm_source ON public.user_sessions USING btree (utm_source);


--
-- Name: idx_user_sessions_visitor_traffic; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_visitor_traffic ON public.user_sessions USING btree (started_at) WHERE (((session_source)::text = 'web'::text) AND (is_bot = false));


--
-- Name: idx_users_bot_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_bot_status ON public.users USING btree (is_bot, is_scanner);


--
-- Name: idx_users_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_email ON public.users USING btree (email);


--
-- Name: idx_users_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_name ON public.users USING btree (name);


--
-- Name: idx_webauthn_challenges_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_challenges_expires_at ON public.webauthn_challenges USING btree (expires_at);


--
-- Name: idx_webauthn_challenges_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_challenges_user_id ON public.webauthn_challenges USING btree (user_id);


--
-- Name: idx_webauthn_credentials_credential_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_credential_id ON public.webauthn_credentials USING btree (credential_id);


--
-- Name: idx_webauthn_credentials_last_used; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_last_used ON public.webauthn_credentials USING btree (last_used_at);


--
-- Name: idx_webauthn_credentials_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_credentials_user_id ON public.webauthn_credentials USING btree (user_id);


--
-- Name: idx_webauthn_setup_tokens_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_setup_tokens_expires_at ON public.webauthn_setup_tokens USING btree (expires_at);


--
-- Name: idx_webauthn_setup_tokens_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_setup_tokens_token_hash ON public.webauthn_setup_tokens USING btree (token_hash);


--
-- Name: idx_webauthn_setup_tokens_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_setup_tokens_user_id ON public.webauthn_setup_tokens USING btree (user_id);


--
-- Name: ingestion_outbox_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX ingestion_outbox_pending ON public.ingestion_outbox USING btree (created_at) WHERE (processed_at IS NULL);


--
-- Name: mcp_tool_executions_owner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX mcp_tool_executions_owner_id ON public.mcp_tool_executions USING btree (user_id, mcp_execution_id);


--
-- Name: oauth_jti_revocations_exp_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX oauth_jti_revocations_exp_idx ON public.oauth_jti_revocations USING btree (exp);


--
-- Name: oauth_jti_revocations_user_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX oauth_jti_revocations_user_idx ON public.oauth_jti_revocations USING btree (user_id);


--
-- Name: oauth_state_bindings_expires_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX oauth_state_bindings_expires_at_idx ON public.oauth_state_bindings USING btree (expires_at);


--
-- Name: plugin_usage_events_owner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX plugin_usage_events_owner_id ON public.plugin_usage_events USING btree (user_id, id);


--
-- Name: plugin_usage_events attribute_skill_version; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER attribute_skill_version AFTER INSERT ON public.plugin_usage_events FOR EACH ROW EXECUTE FUNCTION public.attribute_ingested_skill_invocation();


--
-- Name: ai_requests audit_event_notify_ai_requests_trg; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER audit_event_notify_ai_requests_trg AFTER INSERT ON public.ai_requests FOR EACH ROW EXECUTE FUNCTION public.audit_event_notify_ai_requests();


--
-- Name: governance_decisions audit_event_notify_governance_trg; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER audit_event_notify_governance_trg AFTER INSERT ON public.governance_decisions FOR EACH ROW EXECUTE FUNCTION public.audit_event_notify_governance();


--
-- Name: plugin_usage_events audit_event_notify_plugin_usage_trg; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER audit_event_notify_plugin_usage_trg AFTER INSERT ON public.plugin_usage_events FOR EACH ROW EXECUTE FUNCTION public.audit_event_notify_plugin_usage();


--
-- Name: eval_execution_approvals eval_approval_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_approval_owner_scope BEFORE INSERT OR UPDATE ON public.eval_execution_approvals FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_lifecycle_owner();


--
-- Name: eval_execution_capabilities eval_capability_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_capability_owner_scope BEFORE INSERT OR UPDATE ON public.eval_execution_capabilities FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_owner_scope();


--
-- Name: eval_executions eval_execution_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_execution_owner_scope BEFORE INSERT OR UPDATE OF experiment_id, case_revision_id ON public.eval_executions FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_owner_scope();


--
-- Name: eval_holdout_consumption eval_holdout_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_holdout_owner_scope BEFORE INSERT OR UPDATE ON public.eval_holdout_consumption FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_lifecycle_owner();


--
-- Name: eval_managed_workspace_assets eval_managed_workspace_assets_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_managed_workspace_assets_immutable BEFORE UPDATE ON public.eval_managed_workspace_assets FOR EACH ROW EXECUTE FUNCTION public.reject_eval_managed_workspace_change();


--
-- Name: eval_managed_workspace_projections eval_managed_workspace_projection_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_managed_workspace_projection_immutable BEFORE UPDATE ON public.eval_managed_workspace_projections FOR EACH ROW EXECUTE FUNCTION public.reject_eval_managed_workspace_change();


--
-- Name: eval_approved_operation_receipts eval_operation_receipts_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_operation_receipts_immutable BEFORE DELETE OR UPDATE ON public.eval_approved_operation_receipts FOR EACH ROW EXECUTE FUNCTION public.reject_eval_operation_receipt_change();


--
-- Name: eval_request_reservations eval_request_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_request_owner_scope BEFORE INSERT OR UPDATE ON public.eval_request_reservations FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_owner_scope();


--
-- Name: eval_session_bindings eval_session_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_session_owner_scope BEFORE INSERT OR UPDATE ON public.eval_session_bindings FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_owner_scope();


--
-- Name: eval_suggestions eval_suggestion_owner_scope; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER eval_suggestion_owner_scope BEFORE INSERT OR UPDATE ON public.eval_suggestions FOR EACH ROW EXECUTE FUNCTION public.enforce_eval_lifecycle_owner();


--
-- Name: governance_decisions governance_decisions_append_only; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER governance_decisions_append_only BEFORE UPDATE ON public.governance_decisions FOR EACH ROW EXECUTE FUNCTION public.governance_decisions_deny_update();


--
-- Name: plugin_usage_events ingestion_delivery; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_delivery BEFORE INSERT ON public.plugin_usage_events FOR EACH ROW EXECUTE FUNCTION public.accept_ingestion_delivery();


--
-- Name: plugin_usage_events ingestion_enqueue; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_enqueue AFTER INSERT ON public.plugin_usage_events FOR EACH ROW EXECUTE FUNCTION public.enqueue_ingestion_event();


--
-- Name: plugin_session_summaries ingestion_owner; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON public.plugin_session_summaries FOR EACH ROW EXECUTE FUNCTION public.enforce_ingestion_owner();


--
-- Name: plugin_usage_events ingestion_owner; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON public.plugin_usage_events FOR EACH ROW EXECUTE FUNCTION public.enforce_ingestion_owner();


--
-- Name: session_analyses ingestion_owner; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON public.session_analyses FOR EACH ROW EXECUTE FUNCTION public.enforce_ingestion_owner();


--
-- Name: session_cost_snapshots ingestion_owner; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON public.session_cost_snapshots FOR EACH ROW EXECUTE FUNCTION public.enforce_ingestion_owner();


--
-- Name: session_transcripts ingestion_owner; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON public.session_transcripts FOR EACH ROW EXECUTE FUNCTION public.enforce_ingestion_owner();


--
-- Name: managed_assets managed_assets_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_assets_immutable BEFORE UPDATE ON public.managed_assets FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_revision_dependencies managed_dependencies_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_dependencies_immutable BEFORE UPDATE ON public.managed_revision_dependencies FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_installation_receipts managed_installation_receipts_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_installation_receipts_immutable BEFORE UPDATE ON public.managed_installation_receipts FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_invocation_attributions managed_invocation_attributions_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_invocation_attributions_immutable BEFORE UPDATE ON public.managed_invocation_attributions FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_publication_reviews managed_publication_reviews_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_publication_reviews_immutable BEFORE UPDATE ON public.managed_publication_reviews FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_publications managed_publications_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_publications_immutable BEFORE UPDATE ON public.managed_publications FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_resources managed_resources_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_resources_immutable BEFORE UPDATE ON public.managed_resources FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_revision_assets managed_revision_assets_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_revision_assets_immutable BEFORE UPDATE ON public.managed_revision_assets FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_revisions managed_revisions_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_revisions_immutable BEFORE UPDATE ON public.managed_revisions FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_source_snapshots managed_snapshots_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_snapshots_immutable BEFORE UPDATE ON public.managed_source_snapshots FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_sources managed_sources_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_sources_immutable BEFORE UPDATE ON public.managed_sources FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: groups trg_groups_protect_system; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_groups_protect_system BEFORE DELETE ON public.groups FOR EACH ROW EXECUTE FUNCTION public.groups_protect_system();


--
-- Name: agent_tasks update_agent_tasks_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_agent_tasks_updated_at BEFORE UPDATE ON public.agent_tasks FOR EACH ROW EXECUTE FUNCTION public.update_timestamp_trigger();


--
-- Name: user_contexts update_user_contexts_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_user_contexts_updated_at BEFORE UPDATE ON public.user_contexts FOR EACH ROW EXECUTE FUNCTION public.update_timestamp_trigger();


--
-- Name: subscriptions subscriptions_plan_id_fkey; Type: FK CONSTRAINT; Schema: marketplace; Owner: -
--

ALTER TABLE ONLY marketplace.subscriptions
    ADD CONSTRAINT subscriptions_plan_id_fkey FOREIGN KEY (plan_id) REFERENCES marketplace.plans(id);


--
-- Name: access_control_rules access_control_rules_entity_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_control_rules
    ADD CONSTRAINT access_control_rules_entity_fk FOREIGN KEY (entity_type, entity_id) REFERENCES public.access_control_entities(entity_type, entity_id) ON DELETE CASCADE;


--
-- Name: agent_tasks agent_tasks_context_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.agent_tasks
    ADD CONSTRAINT agent_tasks_context_id_fkey FOREIGN KEY (context_id) REFERENCES public.user_contexts(context_id) ON DELETE CASCADE;


--
-- Name: ai_gateway_thought_signatures ai_gateway_thought_signatures_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_gateway_thought_signatures
    ADD CONSTRAINT ai_gateway_thought_signatures_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: ai_request_messages ai_request_messages_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_messages
    ADD CONSTRAINT ai_request_messages_request_id_fkey FOREIGN KEY (request_id) REFERENCES public.ai_requests(id) ON DELETE CASCADE;


--
-- Name: ai_request_payloads ai_request_payloads_ai_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_payloads
    ADD CONSTRAINT ai_request_payloads_ai_request_id_fkey FOREIGN KEY (ai_request_id) REFERENCES public.ai_requests(id) ON DELETE CASCADE;


--
-- Name: ai_request_tool_calls ai_request_tool_calls_mcp_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_tool_calls
    ADD CONSTRAINT ai_request_tool_calls_mcp_execution_id_fkey FOREIGN KEY (mcp_execution_id) REFERENCES public.mcp_tool_executions(mcp_execution_id) ON DELETE SET NULL;


--
-- Name: ai_request_tool_calls ai_request_tool_calls_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_tool_calls
    ADD CONSTRAINT ai_request_tool_calls_request_id_fkey FOREIGN KEY (request_id) REFERENCES public.ai_requests(id) ON DELETE CASCADE;


--
-- Name: ai_requests ai_requests_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_requests
    ADD CONSTRAINT ai_requests_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.user_sessions(session_id) ON DELETE SET NULL;


--
-- Name: ai_safety_findings ai_safety_findings_ai_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_safety_findings
    ADD CONSTRAINT ai_safety_findings_ai_request_id_fkey FOREIGN KEY (ai_request_id) REFERENCES public.ai_requests(id) ON DELETE CASCADE;


--
-- Name: analytics_events analytics_events_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.analytics_events
    ADD CONSTRAINT analytics_events_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.user_sessions(session_id) ON DELETE SET NULL;


--
-- Name: artifact_parts artifact_parts_context_id_artifact_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.artifact_parts
    ADD CONSTRAINT artifact_parts_context_id_artifact_id_fkey FOREIGN KEY (context_id, artifact_id) REFERENCES public.task_artifacts(context_id, artifact_id) ON DELETE CASCADE;


--
-- Name: bridge_exchange_codes bridge_exchange_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_exchange_codes
    ADD CONSTRAINT bridge_exchange_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: bridge_sessions bridge_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_sessions
    ADD CONSTRAINT bridge_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: bridge_user_host_model_prefs bridge_user_host_model_prefs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_user_host_model_prefs
    ADD CONSTRAINT bridge_user_host_model_prefs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: bridge_user_host_prefs bridge_user_host_prefs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.bridge_user_host_prefs
    ADD CONSTRAINT bridge_user_host_prefs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: campaign_links campaign_links_source_content_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.campaign_links
    ADD CONSTRAINT campaign_links_source_content_id_fkey FOREIGN KEY (source_content_id) REFERENCES public.markdown_content(id) ON DELETE SET NULL;


--
-- Name: content_performance_metrics content_performance_metrics_content_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_performance_metrics
    ADD CONSTRAINT content_performance_metrics_content_id_fkey FOREIGN KEY (content_id) REFERENCES public.markdown_content(id) ON DELETE CASCADE;


--
-- Name: context_agents context_agents_context_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_agents
    ADD CONSTRAINT context_agents_context_id_fkey FOREIGN KEY (context_id) REFERENCES public.user_contexts(context_id) ON DELETE CASCADE;


--
-- Name: context_notifications context_notifications_context_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.context_notifications
    ADD CONSTRAINT context_notifications_context_id_fkey FOREIGN KEY (context_id) REFERENCES public.user_contexts(context_id) ON DELETE CASCADE;


--
-- Name: dev_login_codes dev_login_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.dev_login_codes
    ADD CONSTRAINT dev_login_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: device_app_links device_app_links_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.device_app_links
    ADD CONSTRAINT device_app_links_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: eval_approved_operation_receipts eval_approved_operation_receipts_approval_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_approved_operation_receipts
    ADD CONSTRAINT eval_approved_operation_receipts_approval_id_fkey FOREIGN KEY (approval_id) REFERENCES public.eval_execution_approvals(id);


--
-- Name: eval_approved_operation_receipts eval_approved_operation_receipts_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_approved_operation_receipts
    ADD CONSTRAINT eval_approved_operation_receipts_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_budget_accounts eval_budget_accounts_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_accounts
    ADD CONSTRAINT eval_budget_accounts_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_budget_reservations eval_budget_reservations_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_budget_reservations
    ADD CONSTRAINT eval_budget_reservations_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.eval_budget_accounts(id);


--
-- Name: eval_execution_approvals eval_execution_approvals_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_approvals
    ADD CONSTRAINT eval_execution_approvals_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_execution_approvals eval_execution_approvals_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_approvals
    ADD CONSTRAINT eval_execution_approvals_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_execution_artifacts eval_execution_artifacts_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_artifacts
    ADD CONSTRAINT eval_execution_artifacts_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_execution_evidence(execution_id);


--
-- Name: eval_execution_capabilities eval_execution_capabilities_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_capabilities
    ADD CONSTRAINT eval_execution_capabilities_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_execution_capabilities eval_execution_capabilities_session_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_capabilities
    ADD CONSTRAINT eval_execution_capabilities_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.user_sessions(session_id);


--
-- Name: eval_execution_capabilities eval_execution_capabilities_worker_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_capabilities
    ADD CONSTRAINT eval_execution_capabilities_worker_id_fkey FOREIGN KEY (worker_id) REFERENCES public.eval_workers(id);


--
-- Name: eval_execution_cleanup eval_execution_cleanup_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_cleanup
    ADD CONSTRAINT eval_execution_cleanup_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_execution_events eval_execution_events_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_events
    ADD CONSTRAINT eval_execution_events_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_execution_evidence eval_execution_evidence_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_evidence
    ADD CONSTRAINT eval_execution_evidence_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_execution_measurements eval_execution_measurements_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_execution_measurements
    ADD CONSTRAINT eval_execution_measurements_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_executions eval_executions_case_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_executions
    ADD CONSTRAINT eval_executions_case_revision_id_fkey FOREIGN KEY (case_revision_id) REFERENCES public.eval_resource_revisions(id);


--
-- Name: eval_executions eval_executions_experiment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_executions
    ADD CONSTRAINT eval_executions_experiment_id_fkey FOREIGN KEY (experiment_id) REFERENCES public.eval_experiments(id);


--
-- Name: eval_experiments eval_experiments_budget_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_experiments
    ADD CONSTRAINT eval_experiments_budget_id_fkey FOREIGN KEY (budget_id) REFERENCES public.eval_budget_accounts(id);


--
-- Name: eval_experiments eval_experiments_owner_id_budget_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_experiments
    ADD CONSTRAINT eval_experiments_owner_id_budget_id_fkey FOREIGN KEY (owner_id, budget_id) REFERENCES public.eval_budget_accounts(owner_id, id);


--
-- Name: eval_experiments eval_experiments_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_experiments
    ADD CONSTRAINT eval_experiments_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_fixture_payloads eval_fixture_payloads_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_fixture_payloads
    ADD CONSTRAINT eval_fixture_payloads_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_fixture_test_records eval_fixture_test_records_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_fixture_test_records
    ADD CONSTRAINT eval_fixture_test_records_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_holdout_consumption eval_holdout_consumption_case_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_holdout_consumption
    ADD CONSTRAINT eval_holdout_consumption_case_revision_id_fkey FOREIGN KEY (case_revision_id) REFERENCES public.eval_resource_revisions(id);


--
-- Name: eval_holdout_consumption eval_holdout_consumption_experiment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_holdout_consumption
    ADD CONSTRAINT eval_holdout_consumption_experiment_id_fkey FOREIGN KEY (experiment_id) REFERENCES public.eval_experiments(id);


--
-- Name: eval_holdout_consumption eval_holdout_consumption_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_holdout_consumption
    ADD CONSTRAINT eval_holdout_consumption_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_managed_workspace_assets eval_managed_workspace_assets_owner_id_workspace_digest_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_managed_workspace_assets
    ADD CONSTRAINT eval_managed_workspace_assets_owner_id_workspace_digest_fkey FOREIGN KEY (owner_id, workspace_digest) REFERENCES public.eval_managed_workspace_projections(owner_id, digest);


--
-- Name: eval_request_reservations eval_request_reservations_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_request_reservations
    ADD CONSTRAINT eval_request_reservations_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_request_reservations eval_request_reservations_reservation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_request_reservations
    ADD CONSTRAINT eval_request_reservations_reservation_id_fkey FOREIGN KEY (reservation_id) REFERENCES public.eval_budget_reservations(id);


--
-- Name: eval_resource_revisions eval_resource_revisions_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_resource_revisions
    ADD CONSTRAINT eval_resource_revisions_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_session_bindings eval_session_bindings_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_session_bindings
    ADD CONSTRAINT eval_session_bindings_execution_id_fkey FOREIGN KEY (execution_id) REFERENCES public.eval_executions(id);


--
-- Name: eval_suggestions eval_suggestions_experiment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_suggestions
    ADD CONSTRAINT eval_suggestions_experiment_id_fkey FOREIGN KEY (experiment_id) REFERENCES public.eval_experiments(id);


--
-- Name: eval_suggestions eval_suggestions_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_suggestions
    ADD CONSTRAINT eval_suggestions_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: eval_suggestions eval_suggestions_reservation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_suggestions
    ADD CONSTRAINT eval_suggestions_reservation_id_fkey FOREIGN KEY (reservation_id) REFERENCES public.eval_budget_reservations(id);


--
-- Name: eval_workers eval_workers_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.eval_workers
    ADD CONSTRAINT eval_workers_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: federated_identities federated_identities_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.federated_identities
    ADD CONSTRAINT federated_identities_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: content_files fk_content_files_content; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_files
    ADD CONSTRAINT fk_content_files_content FOREIGN KEY (content_id) REFERENCES public.markdown_content(id) ON DELETE CASCADE;


--
-- Name: content_files fk_content_files_file; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.content_files
    ADD CONSTRAINT fk_content_files_file FOREIGN KEY (file_id) REFERENCES public.files(id) ON DELETE CASCADE;


--
-- Name: user_contexts fk_user_contexts_session; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_contexts
    ADD CONSTRAINT fk_user_contexts_session FOREIGN KEY (session_id) REFERENCES public.user_sessions(session_id) ON DELETE SET NULL;


--
-- Name: user_contexts fk_user_contexts_user; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_contexts
    ADD CONSTRAINT fk_user_contexts_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: funnel_progress funnel_progress_funnel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funnel_progress
    ADD CONSTRAINT funnel_progress_funnel_id_fkey FOREIGN KEY (funnel_id) REFERENCES public.funnels(id) ON DELETE CASCADE;


--
-- Name: funnel_steps funnel_steps_funnel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.funnel_steps
    ADD CONSTRAINT funnel_steps_funnel_id_fkey FOREIGN KEY (funnel_id) REFERENCES public.funnels(id) ON DELETE CASCADE;


--
-- Name: group_ad_mappings group_ad_mappings_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.group_ad_mappings
    ADD CONSTRAINT group_ad_mappings_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.groups(id) ON DELETE CASCADE;


--
-- Name: group_members group_members_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.group_members
    ADD CONSTRAINT group_members_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: group_members group_members_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.group_members
    ADD CONSTRAINT group_members_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.groups(id) ON DELETE CASCADE;


--
-- Name: group_members group_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.group_members
    ADD CONSTRAINT group_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: ingestion_outbox ingestion_outbox_event_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ingestion_outbox
    ADD CONSTRAINT ingestion_outbox_event_id_fkey FOREIGN KEY (event_id) REFERENCES public.plugin_usage_events(id) ON DELETE CASCADE;


--
-- Name: ingestion_session_owners ingestion_session_owners_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ingestion_session_owners
    ADD CONSTRAINT ingestion_session_owners_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: link_clicks link_clicks_link_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.link_clicks
    ADD CONSTRAINT link_clicks_link_id_fkey FOREIGN KEY (link_id) REFERENCES public.campaign_links(id) ON DELETE CASCADE;


--
-- Name: managed_assets managed_assets_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_assets
    ADD CONSTRAINT managed_assets_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: managed_distribution_deliveries managed_distribution_deliveries_outbox_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_deliveries
    ADD CONSTRAINT managed_distribution_deliveries_outbox_id_fkey FOREIGN KEY (outbox_id) REFERENCES public.managed_distribution_outbox(id);


--
-- Name: managed_distribution_deliveries managed_distribution_deliveries_owner_id_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_deliveries
    ADD CONSTRAINT managed_distribution_deliveries_owner_id_publication_id_fkey FOREIGN KEY (owner_id, publication_id) REFERENCES public.managed_publications(owner_id, id);


--
-- Name: managed_distribution_outbox managed_distribution_outbox_owner_id_publication_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_distribution_outbox
    ADD CONSTRAINT managed_distribution_outbox_owner_id_publication_id_fkey FOREIGN KEY (owner_id, publication_id) REFERENCES public.managed_publications(owner_id, id);


--
-- Name: managed_installation_receipts managed_installation_receipts_owner_id_resource_id_publica_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_owner_id_resource_id_publica_fkey FOREIGN KEY (owner_id, resource_id, publication_id) REFERENCES public.managed_publications(owner_id, resource_id, id);


--
-- Name: managed_invocation_attributions managed_invocation_attributions_owner_id_receipt_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_invocation_attributions
    ADD CONSTRAINT managed_invocation_attributions_owner_id_receipt_id_fkey FOREIGN KEY (owner_id, receipt_id) REFERENCES public.managed_installation_receipts(owner_id, id);


--
-- Name: managed_publication_reviews managed_publication_reviews_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: managed_publication_reviews managed_publication_reviews_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_publication_reviews managed_publication_reviews_owner_id_resource_id_revision__fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_owner_id_resource_id_revision__fkey FOREIGN KEY (owner_id, resource_id, revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_publication_reviews managed_publication_reviews_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: managed_publication_selections managed_publication_selection_owner_id_resource_id_publica_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_selections
    ADD CONSTRAINT managed_publication_selection_owner_id_resource_id_publica_fkey FOREIGN KEY (owner_id, resource_id, publication_id) REFERENCES public.managed_publications(owner_id, resource_id, id);


--
-- Name: managed_publication_selections managed_publication_selection_owner_id_resource_id_revisio_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_selections
    ADD CONSTRAINT managed_publication_selection_owner_id_resource_id_revisio_fkey FOREIGN KEY (owner_id, resource_id, revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_publications managed_publications_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_publications managed_publications_owner_id_resource_id_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_resource_id_revision_id_fkey FOREIGN KEY (owner_id, resource_id, revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_publications managed_publications_owner_id_review_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publications
    ADD CONSTRAINT managed_publications_owner_id_review_id_fkey FOREIGN KEY (owner_id, review_id) REFERENCES public.managed_publication_reviews(owner_id, id);


--
-- Name: managed_reconciliation_conflicts managed_reconciliation_conflicts_reconciliation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliation_conflicts
    ADD CONSTRAINT managed_reconciliation_conflicts_reconciliation_id_fkey FOREIGN KEY (reconciliation_id) REFERENCES public.managed_reconciliations(id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_incoming_revi_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_incoming_revi_fkey FOREIGN KEY (owner_id, resource_id, incoming_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_managed_candi_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_managed_candi_fkey FOREIGN KEY (owner_id, resource_id, managed_candidate_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_resolved_revi_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_resolved_revi_fkey FOREIGN KEY (owner_id, resource_id, resolved_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_upstream_base_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_upstream_base_fkey FOREIGN KEY (owner_id, resource_id, upstream_base_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_resolved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_resolved_by_fkey FOREIGN KEY (resolved_by) REFERENCES public.users(id);


--
-- Name: managed_resources managed_resources_owner_id_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_resources
    ADD CONSTRAINT managed_resources_owner_id_source_id_fkey FOREIGN KEY (owner_id, source_id) REFERENCES public.managed_sources(owner_id, id);


--
-- Name: managed_revision_assets managed_revision_assets_owner_id_digest_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revision_assets
    ADD CONSTRAINT managed_revision_assets_owner_id_digest_fkey FOREIGN KEY (owner_id, digest) REFERENCES public.managed_assets(owner_id, digest);


--
-- Name: managed_revision_assets managed_revision_assets_owner_id_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revision_assets
    ADD CONSTRAINT managed_revision_assets_owner_id_revision_id_fkey FOREIGN KEY (owner_id, revision_id) REFERENCES public.managed_revisions(owner_id, id);


--
-- Name: managed_revision_dependencies managed_revision_dependencies_owner_id_dependency_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revision_dependencies
    ADD CONSTRAINT managed_revision_dependencies_owner_id_dependency_id_fkey FOREIGN KEY (owner_id, dependency_id) REFERENCES public.managed_revisions(owner_id, id);


--
-- Name: managed_revision_dependencies managed_revision_dependencies_owner_id_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revision_dependencies
    ADD CONSTRAINT managed_revision_dependencies_owner_id_revision_id_fkey FOREIGN KEY (owner_id, revision_id) REFERENCES public.managed_revisions(owner_id, id);


--
-- Name: managed_revisions managed_revisions_owner_id_resource_id_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_owner_id_resource_id_parent_id_fkey FOREIGN KEY (owner_id, resource_id, parent_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_revisions managed_revisions_owner_id_source_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_owner_id_source_id_resource_id_fkey FOREIGN KEY (owner_id, source_id, resource_id) REFERENCES public.managed_resources(owner_id, source_id, id);


--
-- Name: managed_revisions managed_revisions_owner_id_source_id_snapshot_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_revisions
    ADD CONSTRAINT managed_revisions_owner_id_source_id_snapshot_id_fkey FOREIGN KEY (owner_id, source_id, snapshot_id) REFERENCES public.managed_source_snapshots(owner_id, source_id, id);


--
-- Name: managed_source_snapshots managed_source_snapshots_owner_id_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_source_snapshots
    ADD CONSTRAINT managed_source_snapshots_owner_id_source_id_fkey FOREIGN KEY (owner_id, source_id) REFERENCES public.managed_sources(owner_id, id);


--
-- Name: managed_sources managed_sources_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_sources
    ADD CONSTRAINT managed_sources_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: managed_withdrawal_proposals managed_withdrawal_proposals_decided_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_withdrawal_proposals
    ADD CONSTRAINT managed_withdrawal_proposals_decided_by_fkey FOREIGN KEY (decided_by) REFERENCES public.users(id);


--
-- Name: managed_withdrawal_proposals managed_withdrawal_proposals_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_withdrawal_proposals
    ADD CONSTRAINT managed_withdrawal_proposals_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_withdrawal_proposals managed_withdrawal_proposals_owner_id_snapshot_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_withdrawal_proposals
    ADD CONSTRAINT managed_withdrawal_proposals_owner_id_snapshot_id_fkey FOREIGN KEY (owner_id, snapshot_id) REFERENCES public.managed_source_snapshots(owner_id, id);


--
-- Name: markdown_categories markdown_categories_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_categories
    ADD CONSTRAINT markdown_categories_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.markdown_categories(id) ON DELETE CASCADE;


--
-- Name: markdown_content_enrichment markdown_content_enrichment_content_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_content_enrichment
    ADD CONSTRAINT markdown_content_enrichment_content_id_fkey FOREIGN KEY (content_id) REFERENCES public.markdown_content(id) ON DELETE CASCADE;


--
-- Name: markdown_fts markdown_fts_content_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.markdown_fts
    ADD CONSTRAINT markdown_fts_content_id_fkey FOREIGN KEY (content_id) REFERENCES public.markdown_content(id) ON DELETE CASCADE;


--
-- Name: mcp_connector_accounts mcp_connector_accounts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_connector_accounts
    ADD CONSTRAINT mcp_connector_accounts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: mcp_connector_credentials mcp_connector_credentials_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_connector_credentials
    ADD CONSTRAINT mcp_connector_credentials_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: mcp_connector_oauth_states mcp_connector_oauth_states_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_connector_oauth_states
    ADD CONSTRAINT mcp_connector_oauth_states_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: mcp_sessions mcp_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_sessions
    ADD CONSTRAINT mcp_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: message_parts message_parts_message_id_task_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_parts
    ADD CONSTRAINT message_parts_message_id_task_id_fkey FOREIGN KEY (message_id, task_id) REFERENCES public.task_messages(message_id, task_id) ON DELETE CASCADE;


--
-- Name: oauth_auth_codes oauth_auth_codes_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_auth_codes
    ADD CONSTRAINT oauth_auth_codes_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_auth_codes oauth_auth_codes_refresh_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_auth_codes
    ADD CONSTRAINT oauth_auth_codes_refresh_token_id_fkey FOREIGN KEY (refresh_token_id) REFERENCES public.oauth_refresh_tokens(token_id) ON DELETE SET NULL;


--
-- Name: oauth_auth_codes oauth_auth_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_auth_codes
    ADD CONSTRAINT oauth_auth_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: oauth_client_contacts oauth_client_contacts_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_contacts
    ADD CONSTRAINT oauth_client_contacts_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_client_grant_types oauth_client_grant_types_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_grant_types
    ADD CONSTRAINT oauth_client_grant_types_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_client_redirect_uris oauth_client_redirect_uris_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_redirect_uris
    ADD CONSTRAINT oauth_client_redirect_uris_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_client_response_types oauth_client_response_types_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_response_types
    ADD CONSTRAINT oauth_client_response_types_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_client_scopes oauth_client_scopes_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_client_scopes
    ADD CONSTRAINT oauth_client_scopes_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_clients oauth_clients_owner_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_clients
    ADD CONSTRAINT oauth_clients_owner_user_id_fkey FOREIGN KEY (owner_user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: oauth_refresh_tokens oauth_refresh_tokens_client_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_refresh_tokens
    ADD CONSTRAINT oauth_refresh_tokens_client_id_fkey FOREIGN KEY (client_id) REFERENCES public.oauth_clients(client_id) ON DELETE CASCADE;


--
-- Name: oauth_refresh_tokens oauth_refresh_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.oauth_refresh_tokens
    ADD CONSTRAINT oauth_refresh_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: project_ad_mappings project_ad_mappings_project_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_ad_mappings
    ADD CONSTRAINT project_ad_mappings_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: project_members project_members_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: project_members project_members_project_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: project_members project_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: reviewed_production_failures reviewed_production_failures_development_case_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_development_case_revision_id_fkey FOREIGN KEY (development_case_revision_id) REFERENCES public.eval_resource_revisions(id);


--
-- Name: reviewed_production_failures reviewed_production_failures_invocation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_invocation_id_fkey FOREIGN KEY (invocation_id) REFERENCES public.plugin_usage_events(id);


--
-- Name: reviewed_production_failures reviewed_production_failures_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: reviewed_production_failures reviewed_production_failures_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: salesforce_user_identities salesforce_user_identities_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.salesforce_user_identities
    ADD CONSTRAINT salesforce_user_identities_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: task_artifacts task_artifacts_mcp_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_artifacts
    ADD CONSTRAINT task_artifacts_mcp_execution_id_fkey FOREIGN KEY (mcp_execution_id) REFERENCES public.mcp_tool_executions(mcp_execution_id) ON DELETE SET NULL;


--
-- Name: task_artifacts task_artifacts_task_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_artifacts
    ADD CONSTRAINT task_artifacts_task_id_fkey FOREIGN KEY (task_id) REFERENCES public.agent_tasks(task_id) ON DELETE CASCADE;


--
-- Name: task_messages task_messages_task_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_messages
    ADD CONSTRAINT task_messages_task_id_fkey FOREIGN KEY (task_id) REFERENCES public.agent_tasks(task_id) ON DELETE CASCADE;


--
-- Name: user_activity user_activity_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity
    ADD CONSTRAINT user_activity_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_api_keys user_api_keys_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_api_keys
    ADD CONSTRAINT user_api_keys_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_device_certs user_device_certs_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_device_certs
    ADD CONSTRAINT user_device_certs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_manual_roles user_manual_roles_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_manual_roles
    ADD CONSTRAINT user_manual_roles_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: user_manual_roles user_manual_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_manual_roles
    ADD CONSTRAINT user_manual_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_profile_ext user_profile_ext_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_profile_ext
    ADD CONSTRAINT user_profile_ext_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_scope_defaults user_scope_defaults_primary_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_scope_defaults
    ADD CONSTRAINT user_scope_defaults_primary_group_id_fkey FOREIGN KEY (primary_group_id) REFERENCES public.groups(id) ON DELETE SET NULL;


--
-- Name: user_scope_defaults user_scope_defaults_primary_project_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_scope_defaults
    ADD CONSTRAINT user_scope_defaults_primary_project_id_fkey FOREIGN KEY (primary_project_id) REFERENCES public.projects(id) ON DELETE SET NULL;


--
-- Name: user_scope_defaults user_scope_defaults_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_scope_defaults
    ADD CONSTRAINT user_scope_defaults_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_sessions user_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: webauthn_challenges webauthn_challenges_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_challenges
    ADD CONSTRAINT webauthn_challenges_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: webauthn_credentials webauthn_credentials_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_credentials
    ADD CONSTRAINT webauthn_credentials_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: webauthn_setup_tokens webauthn_setup_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.webauthn_setup_tokens
    ADD CONSTRAINT webauthn_setup_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--


--
-- PostgreSQL database dump
--


-- Dumped from database version 18.6 (Debian 18.6-1.pgdg12+2)
-- Dumped by pg_dump version 18.6 (Debian 18.6-1.pgdg12+2)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: extension_migrations; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.extension_migrations VALUES ('users_001', 'users', 1, 'add_user_sessions_utm_content_term', 'c0a14059cd93c06b', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_002', 'users', 2, 'add_user_sessions_is_ai_crawler', '86dee16607f8c46b', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_003', 'users', 3, 'rebuild_clean_traffic_index', '48779a93e84bd657', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_004', 'users', 4, 'user_sessions_revoked_at', '3d4aa4464c21e784', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_005', 'users', 5, 'federated_identities', '5eb68ff5b896c3e7', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_006', 'users', 6, 'user_sessions_source_bridge_mcp', '255eda2491681b0d', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_007', 'users', 7, 'drop_session_throttle', '2e85bac9ee4c970a', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_008', 'users', 8, 'canonical_traffic_views', 'afaefa896c0c7709', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_009', 'users', 9, 'normalise_user_emails', 'c42dfe07df4c2f73', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_010', 'users', 10, 'drop_users_name_unique', 'f93a41fb813d1947', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('users_011', 'users', 11, 'user_rate_limit_buckets', '75b57bf7703a7107', '2026-09-14 14:42:39.210074+00');
INSERT INTO public.extension_migrations VALUES ('mcp_001', 'mcp', 1, 'session_initialize_params', '6e82b1867bcb9d01', '2026-09-14 14:42:39.236773+00');
INSERT INTO public.extension_migrations VALUES ('mcp_002', 'mcp', 2, 'artifact_server_name_repair', '93dc95338d6eaad2', '2026-09-14 14:42:39.236773+00');
INSERT INTO public.extension_migrations VALUES ('mcp_003', 'mcp', 3, 'tool_execution_actor', 'ce0e7dee4fefdedb', '2026-09-14 14:42:39.236773+00');
INSERT INTO public.extension_migrations VALUES ('mcp_004', 'mcp', 4, 'mcp_proxy_identities', '5805779bbc83176', '2026-09-14 14:42:39.236773+00');
INSERT INTO public.extension_migrations VALUES ('mcp_005', 'mcp', 5, 'mcp_external_sessions', '6059a4ce25f71342', '2026-09-14 14:42:39.236773+00');
INSERT INTO public.extension_migrations VALUES ('ai_001', 'ai', 1, 'gateway_governance', '6647ab24a5297e18', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_002', 'ai', 2, 'split_context_id', '283dacb72e10ace6', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_003', 'ai', 3, 'drop_runtime_tenancy', '5c00241a010f5b30', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_004', 'ai', 4, 'actor_attribution', 'b4b71c20945498ce', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_005', 'ai', 5, 'actor_attribution_lock', '41f8f608d11afe68', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_006', 'ai', 6, 'requested_model', '13873e7434ff27b0', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_007', 'ai', 7, 'system_prompt_override', '5d09e4ba6239cde6', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_008', 'ai', 8, 'route_match', '56a3adbfa1456076', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_009', 'ai', 9, 'ai_requests_session_fk', 'e6b60c61c31e56', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_010', 'ai', 10, 'nullable_rejection_routing', '9d20dabafde495b3', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_011', 'ai', 11, 'subject_quota_buckets', 'a759c8ceaf808d1e', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_012', 'ai', 12, 'payload_digests', '82477d99ca6bdc3b', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_013', 'ai', 13, 'offered_tools', '9ef63dc760df8ad6', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_014', 'ai', 14, 'ai_requests_context_not_null', '39c927186c0e6806', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_015', 'ai', 15, 'ai_requests_synthetic', 'be1e2dd5367b4f01', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_016', 'ai', 16, 'gateway_policy_priority', '721730d229fbf11', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_017', 'ai', 17, 'gateway_thought_signatures', '5ed56b0e63bdbebd', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_018', 'ai', 18, 'ai_requests_instance_id', '34ff135ba22dfb4f', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_019', 'ai', 19, 'ai_safety_findings_blocked', '2ac0ab11a784f61a', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_020', 'ai', 20, 'ai_requests_reasoning_tokens', '2bfd66340b059888', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_021', 'ai', 21, 'ai_requests_client_session_kind', 'bdfecdeeab22a043', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_022', 'ai', 22, 'ai_requests_upstream_latency', '9bc89b1cd3c7bfcb', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('ai_023', 'ai', 23, 'thought_signature_owner', '6cb1cc70ab8b3df8', '2026-09-14 14:42:39.256146+00');
INSERT INTO public.extension_migrations VALUES ('oauth_001', 'oauth', 1, 'add_rfc8707_resource_column', '5cb4f5c9da74621b', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_002', 'oauth', 2, 'rename_cowork_to_bridge', '198505fa7bcda067', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_003', 'oauth', 3, 'drop_bridge_session_tenant', '3a4abf70ba74e42e', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_004', 'oauth', 4, 'oauth_client_owner', '943cc8f5b9258513', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_005', 'oauth', 5, 'auth_code_family', '3348088297180400', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_006', 'oauth', 6, 'at_rest_pepper_hash', 'c09905a92c328ec0', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_007', 'oauth', 7, 'oauth_state_bindings', '35ec17997061c78a', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_008', 'oauth', 8, 'oauth_jti_revocations', 'a538e12a98825351', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_009', 'oauth', 9, 'refresh_token_consumed_at', '47aad1b5f738a86c', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_010', 'oauth', 10, 'backfill_oauth_client_owner_fk', '3c9a33f1ee3649d6', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_011', 'oauth', 11, 'add_application_type', '223c7839bafafc15', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_012', 'oauth', 12, 'bridge_host_model_prefs', 'd51d8b93bdd0850e', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_013', 'oauth', 13, 'id_jag_replay', '8ae2cb0e16a7c59c', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_014', 'oauth', 14, 'webauthn_challenges_state_store', '3d74b8c825f4d8e0', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('oauth_015', 'oauth', 15, 'webauthn_challenges_user_fk', '7356fd9dc1be438a', '2026-09-14 14:42:39.292447+00');
INSERT INTO public.extension_migrations VALUES ('agent_001', 'agent', 1, 'drop_playbooks', '7a26fce7c284e474', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_002', 'agent', 2, 'add_server_type', '6bc6f5ee9aa6af9e', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_003', 'agent', 3, 'a2a_v1_task_states', '66e355af4613376b', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_004', 'agent', 4, 'ai_requests_task_fk', '77036ba02a49c5e3', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_005', 'agent', 5, 'add_task_version', '7818e616975c2e50', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_006', 'agent', 6, 'drop_agent_skills', '5b8f7535da82eeb1', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_007', 'agent', 7, 'drop_agents', '663e96f3a336c0d0', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_008', 'agent', 8, 'add_user_contexts_kind', '87c13f9bf6829e42', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_009', 'agent', 9, 'drop_session_analytics_views', '11cdc3016bad0e46', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_010', 'agent', 10, 'services_instance_scope', '853a6d8c617ed926', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('agent_011', 'agent', 11, 'drop_task_push_notification_configs', 'baabea821e575b60', '2026-09-14 14:42:39.348806+00');
INSERT INTO public.extension_migrations VALUES ('analytics_001', 'analytics', 1, 'add_engagement_event_type', 'b65f680d8b8f758c', '2026-09-14 14:42:39.38356+00');
INSERT INTO public.extension_migrations VALUES ('analytics_002', 'analytics', 2, 'add_engagement_event_data', '25a139a9f89d970', '2026-09-14 14:42:39.38356+00');
INSERT INTO public.extension_migrations VALUES ('analytics_003', 'analytics', 3, 'seed_anomaly_thresholds', 'fc38b38f62e9f4fd', '2026-09-14 14:42:39.38356+00');
INSERT INTO public.extension_migrations VALUES ('analytics_004', 'analytics', 4, 'drop_high_risk_fingerprints_view', '97cddedc7c16ad6d', '2026-09-14 14:42:39.38356+00');
INSERT INTO public.extension_migrations VALUES ('authz_001', 'authz', 1, 'access_control_rules_evolution', '68193354fdf3108f', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_002', 'authz', 2, 'actor_attribution', 'ede93e6870ebb40a', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_003', 'authz', 3, 'actor_attribution_lock', 'd1104b371fd826c8', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_004', 'authz', 4, 'act_chain', '2c3b23e4e0394e93', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_005', 'authz', 5, 'actor_kind_extend', '5410ae5886cf28d', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_007', 'authz', 7, 'split_acl_entities', '4de17568362600bc', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_008', 'authz', 8, 'drop_department_acl', '92898de595a1f274', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_009', 'authz', 9, 'messaging_acl_entity_types', 'be306370e020a64e', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_010', 'authz', 10, 'governance_context_task', '742804d56c0379ad', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_011', 'authz', 11, 'open_rule_type_vocabulary', '944d5199229cf231', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_012', 'authz', 12, 'governance_decisions_context_not_null', '8ee9292b63182b9f', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_013', 'authz', 13, 'governance_decisions_trace_id', '983a850fe541d68c', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_014', 'authz', 14, 'tool_approval_requests', 'c0982044fab4deba', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_015', 'authz', 15, 'governance_decisions_client_id', '50f961043a194a21', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_016', 'authz', 16, 'governance_decisions_warn', '57e679ac802b5a00', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_017', 'authz', 17, 'access_control_rules_source', '895edddb26c7a07a', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('authz_018', 'authz', 18, 'governance_decisions_append_only', 'f3b805f30e0a4bba', '2026-09-14 14:42:39.40394+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_001', 'evaluation', 1, 'reconcile_template_evals', 'faa7a920541057e7', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_002', 'evaluation', 2, 'result_repair_hint', 'cc4be98e511e2858', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_003', 'evaluation', 3, 'execution_deadline', 'd8545c5ee8f9dcc0', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_004', 'evaluation', 4, 'shared_budget_accounts', '6bba36407aaebad5', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_005', 'evaluation', 5, 'managed_workspace_projection', '64b329d69b6d0955', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_006', 'evaluation', 6, 'execution_runtime_and_lifecycle', '59c660631fe2c8d7', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_007', 'evaluation', 7, 'owner_constraints', 'f857d035f08ffe1e', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_008', 'evaluation', 8, 'managed_workspace_immutability', '628af9acbb439bc1', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_009', 'evaluation', 9, 'allow_managed_workspace_cleanup', 'f8d6a55388d0effd', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('evaluation_010', 'evaluation', 10, 'drop_judge_run_tables', '503865428b14d28d', '2026-09-14 14:42:39.443186+00');
INSERT INTO public.extension_migrations VALUES ('files_001', 'files', 1, 'drop_ai_image_stats_view', '334c6fe3a3d35e4a', '2026-09-14 14:42:39.521143+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_001', 'managed_resources', 1, 'revision_listing_indexes', 'da2066a73c72406a', '2026-09-14 14:42:39.538294+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_002', 'managed_resources', 2, 'managed_resolution', 'faaf1cc3e38aed03', '2026-09-14 14:42:39.538294+00');
INSERT INTO public.extension_migrations VALUES ('web_029', 'web', 29, 'analytics_indexes', 'dd63d3b66cea0bfa', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_030', 'web', 30, 'usage_loc_columns', '1d4cd056fdbbb69a', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_032', 'web', 32, 'usage_anomalies', '4475db928b2e8ffd', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_033', 'web', 33, 'transcript_fts', '6234322caae1ff83', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_036', 'web', 36, 'atlassian_credentials', 'f01e20e7ec21de8a', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_037', 'web', 37, 'salesforce_identity', 'f1c9c219ed87e611', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_039', 'web', 39, 'dev_login_codes', 'd9b7a5d48f2b825a', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_040', 'web', 40, 'skill_invocation_view', '9d68dae98bdc4f90', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_041', 'web', 41, 'groups_projects', '6a9052940140c20c', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_042', 'web', 42, 'scope_defaults', 'b5813136d22a2966', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_043', 'web', 43, 'knowledge_worker_role', '113edf7aa8081b32', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_044', 'web', 44, 'connector_credentials', '770f273e304bdcd8', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_045', 'web', 45, 'connector_accounts', 'd0aab719fa26f9f5', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_046', 'web', 46, 'legacy_organization_cleanup', 'd32f62b3a841085c', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_047', 'web', 47, 'admin_marketplace_access', '75b93f90e0b09f7b', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_048', 'web', 48, 'drop_atlassian_user_credentials', 'db1b39e8a9a4a0e7', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_049', 'web', 49, 'conversation_requests', '223db4b66db7622f', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_050', 'web', 50, 'dashboard_query_scaling', 'b4c968b69746392f', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_051', 'web', 51, 'configured_connectors', '35110bcf95ffea4a', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_052', 'web', 52, 'executive_access_project_name', '3c6225724cbcf8f6', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_053', 'web', 53, 'salesforce_identity_provider', 'fd3cd8b2868449e2', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_054', 'web', 54, 'ingestion_integrity', 'b17066563102ab6', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_055', 'web', 55, 'recover_native_sessions', '36758a802db1a2f0', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_056', 'web', 56, 'skill_version_impact', 'f91e69bfe09dfd20', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_057', 'web', 57, 'version_impact_owner_constraints', '2c0acd171b77f5c1', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('web_058', 'web', 58, 'drop_independently_metered_tool_charges', '27e63f935d1c044a', '2026-09-14 14:42:39.597949+00');
INSERT INTO public.extension_migrations VALUES ('content_001', 'content', 1, 'markdown_content_locale_unique', 'a82d5a7751bbf861', '2026-09-14 14:42:39.426575+00');
INSERT INTO public.extension_migrations VALUES ('content_002', 'content', 2, 'drop_link_analytics_views', '930d20c045f9ac0c', '2026-09-14 14:42:39.426575+00');
INSERT INTO public.extension_migrations VALUES ('events_001', 'events', 1, 'actor_attribution', '4ae2194aa1e8c3ee', '2026-09-14 14:42:39.514882+00');
INSERT INTO public.extension_migrations VALUES ('events_002', 'events', 2, 'actor_attribution_lock', '260f1b865336ddd7', '2026-09-14 14:42:39.514882+00');
INSERT INTO public.extension_migrations VALUES ('events_003', 'events', 3, 'outbox_origin_instance', '9dd01b8cb5729ffb', '2026-09-14 14:42:39.514882+00');
INSERT INTO public.extension_migrations VALUES ('logging_001', 'logging', 1, 'split_context_id', 'af4412823858f9e1', '2026-09-14 14:42:39.528767+00');
INSERT INTO public.extension_migrations VALUES ('logging_002', 'logging', 2, 'analytics_event_data', '26db842d76c715f8', '2026-09-14 14:42:39.528767+00');
INSERT INTO public.extension_migrations VALUES ('logging_003', 'logging', 3, 'prune_redundant_log_indexes', 'dea386f4e03102a4', '2026-09-14 14:42:39.528767+00');
INSERT INTO public.extension_migrations VALUES ('logging_004', 'logging', 4, 'drop_client_log_views', '1ffc2da4a1ec3e2d', '2026-09-14 14:42:39.528767+00');
INSERT INTO public.extension_migrations VALUES ('logging_005', 'logging', 5, 'logs_instance_id', 'edffd3658ff44d2a', '2026-09-14 14:42:39.528767+00');
INSERT INTO public.extension_migrations VALUES ('scheduler_001', 'scheduler', 1, 'scheduled_jobs_last_instance', '2068fc4ef8c7cd71', '2026-09-14 14:42:39.592319+00');


--
-- PostgreSQL database dump complete
--


