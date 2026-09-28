-- systemprompt-template release-baseline: 0.62.0 (core v0.62.0)
-- Recorded by 'just schema-baseline' from a fresh install; the upgrade test
-- restores it and migrates forward. Re-record after every version bump.
--
-- PostgreSQL database dump
--


-- Dumped from database version 18.3
-- Dumped by pg_dump version 18.3

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
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
    IF digest IS NOT NULL THEN INSERT INTO ingestion_event_receipts(dedup_key, payload_digest) VALUES(NEW.dedup_key, digest)
        ON CONFLICT(dedup_key) DO NOTHING;
        SELECT payload_digest INTO old_digest FROM ingestion_event_receipts WHERE dedup_key = NEW.dedup_key;
        IF old_digest <> digest THEN
            RAISE EXCEPTION 'Conflicting event delivery' USING ERRCODE = '23505';
        END IF;
    END IF;
    RETURN NEW;
END $$;


--
-- Name: active_device_identity(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.active_device_identity(requested_device text) RETURNS TABLE(device_id text, consumer_id text)
    LANGUAGE sql
    SET search_path TO 'pg_catalog', 'public'
    AS $$
    SELECT d.id, d.user_id FROM public.user_device_certs d
    WHERE d.id = requested_device AND d.revoked_at IS NULL
    FOR SHARE OF d
$$;


--
-- Name: active_devices_for_consumer(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.active_devices_for_consumer(requested_consumer text) RETURNS TABLE(device_id text, consumer_id text)
    LANGUAGE sql STABLE
    SET search_path TO 'pg_catalog', 'public'
    AS $$
    SELECT d.id, d.user_id FROM public.user_device_certs d
    WHERE d.user_id = requested_consumer AND d.revoked_at IS NULL
$$;


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
-- Name: artifact_kind(text, text, boolean, boolean, text, text, boolean); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.artifact_kind(tool_name text, artifact_type text, has_ui boolean, is_structured boolean, payload_sha256 text, input text, is_builtin boolean) RETURNS text
    LANGUAGE sql IMMUTABLE
    AS $$
SELECT CASE
    WHEN tool_name IN ('Edit', 'Write', 'MultiEdit', 'NotebookEdit', 'Read')
         AND input IS NOT NULL AND position('"file_path"' IN input) > 0 THEN 'file'
    WHEN COALESCE(is_builtin, false) THEN NULL
    WHEN COALESCE(has_ui, false) THEN 'ui'
    WHEN artifact_type IS NOT NULL AND artifact_type <> 'tool_result' THEN 'card'
    WHEN payload_sha256 IS NOT NULL AND COALESCE(is_structured, false) THEN 'body'
    ELSE NULL END
$$;


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
    END IF; INSERT INTO ingestion_session_owners(session_id, user_id) VALUES(session_key, owner_key)
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
    ignored integer;
BEGIN
    IF NEW.event_type = 'UserPromptSubmit' AND NEW.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+' THEN
        raw_skill := substring(NEW.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)');
    ELSIF NEW.event_type IN ('PostToolUse','PostToolUseFailure') AND NEW.tool_name = 'Skill' THEN
        raw_skill := NEW.metadata->'tool_input'->>'skill';
    ELSE
        RETURN NEW;
    END IF;
    skill_key := replace(split_part(raw_skill, ':', 2), '-', '_');
    traffic := 'production';
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
    WITH attributed AS (INSERT INTO managed_invocation_attributions(
        id,owner_id,invocation_id,installation_id,resource_id,revision_id,
        publication_generation,traffic_class,status,receipt_id,authenticated_evidence
    ) VALUES (
        'mia_' || md5(NEW.id || NEW.user_id),NEW.user_id,NEW.id,
        CASE WHEN receipt.receipt_id IS NULL THEN NULL ELSE NEW.metadata->>'installation_id' END,
        receipt.resource_id,receipt.revision_id,receipt.generation,traffic,
        CASE WHEN receipt.receipt_id IS NULL THEN 'revision_unknown' ELSE 'verified' END,
        receipt.receipt_id,
        jsonb_build_object('session_id',NEW.session_id,'dedup_key',NEW.dedup_key,'ingestion_owner_verified',true)
    ) ON CONFLICT(owner_id,invocation_id) DO NOTHING RETURNING 1)
    SELECT 1 INTO ignored FROM attributed LIMIT 1;
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
    SELECT r.*, conversation_request_kind(r.request_kind, p.offered_tools_sha256 IS NOT NULL, r.thread_size) AS effective_kind
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
    FROM (SELECT DISTINCT session_id FROM plugin_usage_events WHERE id = ANY(event_ids) ORDER BY session_id) owners; INSERT INTO plugin_session_summaries(id,session_id,user_id,total_events,tool_uses,prompts,errors,
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
    WHERE plugin_session_summaries.user_id=EXCLUDED.user_id; UPDATE ingestion_outbox SET processed_at=now() WHERE event_id=ANY(event_ids);
    GET DIAGNOSTICS affected = ROW_COUNT;
    RETURN affected;
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
    AS $$ BEGIN INSERT INTO ingestion_outbox(event_id) VALUES(NEW.id) ON CONFLICT DO NOTHING;
    RETURN NEW;
END $$;


--
-- Name: expire_raw_evidence(timestamp with time zone); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.expire_raw_evidence(cutoff timestamp with time zone) RETURNS bigint
    LANGUAGE plpgsql
    AS $$
DECLARE deleted BIGINT;
BEGIN
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Raw evidence retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
    DELETE FROM ingestion_event_receipts WHERE accepted_at<cutoff;
    DELETE FROM plugin_usage_events WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    DELETE FROM ai_requests WHERE created_at<cutoff;
    DELETE FROM conversation_analyses WHERE created_at<cutoff;
    DELETE FROM plugin_session_summaries WHERE COALESCE(ended_at,updated_at,created_at)<cutoff;
    DELETE FROM session_entity_links WHERE last_seen_at<cutoff;
    DELETE FROM ingestion_session_owners o WHERE created_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.session_id=o.session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_session_summaries s WHERE s.session_id=o.session_id);
    PERFORM public.expire_user_sessions(cutoff);
    RETURN deleted;
END $$;


--
-- Name: expire_user_sessions(timestamp with time zone); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.expire_user_sessions(retained_after timestamp with time zone) RETURNS bigint
    LANGUAGE plpgsql
    SET search_path TO 'pg_catalog', 'public'
    AS $$
DECLARE removed BIGINT;
BEGIN
    IF retained_after IS NULL OR retained_after > NOW() THEN
        RAISE EXCEPTION 'Invalid session retention cutoff' USING ERRCODE = '22023';
    END IF;
    DELETE FROM public.user_sessions WHERE last_activity_at < retained_after
        AND (ended_at IS NOT NULL OR revoked_at IS NOT NULL OR expires_at IS NULL OR expires_at <= NOW());
    GET DIAGNOSTICS removed = ROW_COUNT;
    RETURN removed;
END
$$;


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
-- Name: marketplace_version_at(text, timestamp with time zone); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.marketplace_version_at(mid text, at timestamp with time zone) RETURNS text
    LANGUAGE sql STABLE
    AS $$
SELECT content_hash FROM marketplace_versions
WHERE marketplace_id = mid AND first_seen_at <= at
ORDER BY (effective_until IS NULL OR at < effective_until) DESC, first_seen_at DESC
LIMIT 1
$$;


--
-- Name: refresh_conversation_facts(text[]); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_conversation_facts(context_ids text[]) RETURNS bigint
    LANGUAGE plpgsql
    AS $$
DECLARE written BIGINT; stamp TIMESTAMPTZ := clock_timestamp();
BEGIN
    WITH requests AS MATERIALIZED (
        -- Why: this repo's conversation_requests (27_) does not carry the
        -- client columns, so they are read from ai_requests, joined anyway.
        -- When astound's 27_ (which exposes them) is ported, drop
        -- `ar.client_kind, ar.client_attestation` here or the CTE carries
        -- duplicate column names. Only this file changes; 091 is history.
        SELECT r.*, ar.client_kind, ar.client_attestation,
               ar.wire_protocol, ar.cache_read_tokens, ar.cache_creation_tokens,
               ar.reasoning_tokens, ar.is_streaming
        FROM conversation_requests r
        JOIN ai_requests ar ON ar.id = r.id
        WHERE context_ids IS NULL OR r.context_id = ANY(context_ids)
    ), agg AS (
        SELECT r.context_id,
            (ARRAY_AGG(r.user_id ORDER BY r.created_at DESC, r.id DESC))[1] AS user_id,
            (ARRAY_AGG(r.id ORDER BY r.created_at DESC, r.id DESC))[1] AS last_request_id,
            (ARRAY_AGG(r.session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.session_id IS NOT NULL))[1] AS session_id,
            (ARRAY_AGG(r.client_session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.client_session_id IS NOT NULL))[1] AS client_session_id,
            (ARRAY_AGG(r.client_kind ORDER BY r.created_at DESC, r.id DESC))[1] AS client_kind,
            (ARRAY_AGG(r.client_attestation ORDER BY r.created_at DESC, r.id DESC))[1] AS client_attestation,
            (ARRAY_AGG(r.wire_protocol ORDER BY r.created_at DESC, r.id DESC))[1] AS wire_protocol,
            (ARRAY_AGG(r.model ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.model IS NOT NULL))[1] AS model,
            (ARRAY_AGG(r.provider ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.provider IS NOT NULL))[1] AS provider,
            COUNT(*)::bigint AS request_count,
            COUNT(*) FILTER (WHERE r.effective_kind = 'turn')::bigint AS turn_count,
            COUNT(*) FILTER (WHERE r.effective_kind <> 'turn')::bigint AS side_call_count,
            COALESCE(SUM(r.cost_microdollars) FILTER (WHERE r.effective_kind <> 'turn'), 0)::bigint AS side_call_cost,
            COUNT(*) FILTER (WHERE r.status = 'failed')::bigint AS error_count,
            COUNT(*) FILTER (WHERE r.status = 'rejected')::bigint AS rejected_count,
            COUNT(*) FILTER (WHERE r.is_streaming)::bigint AS streaming_count,
            COALESCE(SUM(r.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(r.output_tokens), 0)::bigint AS output_tokens,
            COALESCE(SUM(r.cache_read_tokens), 0)::bigint AS cache_read_tokens,
            COALESCE(SUM(r.cache_creation_tokens), 0)::bigint AS cache_creation_tokens,
            COALESCE(SUM(r.reasoning_tokens), 0)::bigint AS reasoning_tokens,
            COALESCE(SUM(r.cost_microdollars), 0)::bigint AS cost_microdollars,
            percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p50,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p95,
            MAX(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn') AS max_latency_ms,
            COALESCE(SUM(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn'), 0)::bigint AS active_ms,
            MIN(r.created_at) AS first_at,
            MAX(COALESCE(r.completed_at, r.created_at)) AS last_at
        FROM requests r
        GROUP BY r.context_id
    ), used AS (
        SELECT r.context_id,
            ARRAY(SELECT m.model FROM (
                SELECT model, MIN(created_at) AS first_use FROM requests q
                WHERE q.context_id = r.context_id AND q.model IS NOT NULL GROUP BY model) m
                ORDER BY m.first_use, m.model) AS models,
            ARRAY(SELECT DISTINCT q.provider FROM requests q
                WHERE q.context_id = r.context_id AND q.provider IS NOT NULL ORDER BY q.provider) AS providers
        FROM (SELECT DISTINCT context_id FROM requests) r
    ), intents AS (
        SELECT r.context_id, COUNT(*)::bigint AS tool_calls_intended
        FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
        GROUP BY r.context_id
    ), safety AS (
        SELECT r.context_id, COUNT(*)::bigint AS findings,
               COUNT(*) FILTER (WHERE f.blocked)::bigint AS blocked
        FROM requests r JOIN ai_safety_findings f ON f.ai_request_id = r.id
        GROUP BY r.context_id
    ), sessions AS (
        SELECT DISTINCT r.client_session_id
        FROM requests r WHERE r.client_session_id IS NOT NULL
    ), windows AS (
        SELECT r.context_id, r.client_session_id,
               MIN(r.created_at) AS opened_at,
               MAX(COALESCE(r.completed_at, r.created_at)) AS closed_at
        FROM conversation_requests r
        WHERE r.client_session_id IN (SELECT s.client_session_id FROM sessions s)
        GROUP BY r.context_id, r.client_session_id
    ), executions AS (
        SELECT o.context_id,
               COUNT(*)::bigint AS executed,
               COUNT(*) FILTER (WHERE o.execution_status IN ('failed', 'timeout'))::bigint AS failed,
               COUNT(*) FILTER (WHERE o.artifact_kind IS NOT NULL)::bigint AS artifacts,
               COUNT(*) FILTER (WHERE o.artifact_kind = 'file')::bigint AS artifact_files,
               COUNT(*) FILTER (WHERE o.artifact_kind IN ('ui', 'card', 'body'))::bigint AS artifact_cards
        FROM (
            SELECT CASE WHEN EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                        THEN t.execution_context_id ELSE w.context_id END AS context_id,
                   t.execution_status, t.artifact_kind
            FROM tool_activity t
            LEFT JOIN LATERAL (
                SELECT x.context_id FROM windows x
                WHERE NOT EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                  AND x.client_session_id = t.execution_trace_id
                ORDER BY (COALESCE(t.executed_at, t.occurred_at) BETWEEN x.opened_at AND x.closed_at) DESC,
                         abs(EXTRACT(EPOCH FROM COALESCE(t.executed_at, t.occurred_at) - x.opened_at)), x.opened_at, x.context_id
                LIMIT 1) w ON TRUE
            WHERE t.mcp_execution_id IS NOT NULL
              AND (EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                   OR w.context_id IS NOT NULL)
        ) o
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = o.context_id)
        GROUP BY o.context_id
    ), governance AS (
        SELECT o.context_id,
               COUNT(*) FILTER (WHERE o.decision = 'allow')::bigint AS allow,
               COUNT(*) FILTER (WHERE o.decision = 'warn')::bigint AS warn,
               COUNT(*) FILTER (WHERE o.decision = 'deny')::bigint AS deny
        FROM (
            SELECT CASE WHEN EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
                        THEN g.context_id ELSE w.context_id END AS context_id, g.decision
            FROM governance_decisions g
            LEFT JOIN LATERAL (
                SELECT x.context_id FROM windows x
                WHERE NOT EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
                  AND x.client_session_id = g.session_id
                ORDER BY (g.created_at BETWEEN x.opened_at AND x.closed_at) DESC,
                         abs(EXTRACT(EPOCH FROM g.created_at - x.opened_at)), x.opened_at, x.context_id
                LIMIT 1) w ON TRUE
            WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
               OR w.context_id IS NOT NULL
        ) o
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = o.context_id)
        GROUP BY o.context_id
    ), hook_events AS (
        SELECT w.context_id, e.event_type, e.id
        FROM plugin_usage_events e
        JOIN LATERAL (
            SELECT x.context_id FROM windows x
            WHERE x.client_session_id = e.session_id
            ORDER BY (e.created_at BETWEEN x.opened_at AND x.closed_at) DESC,
                     abs(EXTRACT(EPOCH FROM e.created_at - x.opened_at)), x.opened_at, x.context_id
            LIMIT 1) w ON TRUE
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = w.context_id)
    ), hooks AS (
        SELECT h.context_id,
               COUNT(*)::bigint AS hook_events,
               COUNT(*) FILTER (WHERE h.event_type = 'UserPromptSubmit')::bigint AS prompts
        FROM hook_events h
        GROUP BY h.context_id
    ), skill_events AS (
        SELECT w.context_id, s.id, s.plugin_id, s.skill, s.invoked_at
        FROM analysis_skill_events s
        JOIN LATERAL (
            SELECT x.context_id FROM windows x
            WHERE x.client_session_id = s.session_id
            ORDER BY (s.invoked_at BETWEEN x.opened_at AND x.closed_at) DESC,
                     abs(EXTRACT(EPOCH FROM s.invoked_at - x.opened_at)), x.opened_at, x.context_id
            LIMIT 1) w ON TRUE
        WHERE s.skill IS NOT NULL AND EXISTS (SELECT 1 FROM agg a WHERE a.context_id = w.context_id)
    ), skills AS (
        SELECT k.context_id, COUNT(*)::bigint AS invocations,
               ARRAY(SELECT y.skill FROM (
                   SELECT q.skill, MIN(q.invoked_at) AS first_at FROM skill_events q
                   WHERE q.context_id = k.context_id GROUP BY q.skill) y
                   ORDER BY y.first_at, y.skill) AS skills
        FROM skill_events k
        GROUP BY k.context_id
    ), skill_upsert AS (
        INSERT INTO conversation_skill_facts AS t (
            context_id, plugin_id, skill, user_id, marketplace_id, marketplace_hash,
            invocations, failures, first_invoked_at, last_invoked_at, refreshed_at)
        SELECT k.context_id, k.plugin_id, k.skill, a.user_id, o.marketplace_id,
               marketplace_version_at(o.marketplace_id, MIN(k.invoked_at)),
               COUNT(*)::bigint,
               COUNT(*) FILTER (WHERE p.event_type = 'PostToolUseFailure')::bigint,
               MIN(k.invoked_at), MAX(k.invoked_at), stamp
        FROM skill_events k
        JOIN agg a ON a.context_id = k.context_id
        JOIN plugin_usage_events p ON p.id = k.id
        LEFT JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = k.plugin_id
        WHERE k.plugin_id IS NOT NULL
        GROUP BY k.context_id, k.plugin_id, k.skill, a.user_id, o.marketplace_id
        ON CONFLICT (context_id, plugin_id, skill) DO UPDATE SET
            user_id = EXCLUDED.user_id, marketplace_id = EXCLUDED.marketplace_id,
            marketplace_hash = EXCLUDED.marketplace_hash, invocations = EXCLUDED.invocations,
            failures = EXCLUDED.failures, first_invoked_at = EXCLUDED.first_invoked_at,
            last_invoked_at = EXCLUDED.last_invoked_at, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1
    ), upsert AS (
        INSERT INTO conversation_facts AS f (
            context_id, user_id, session_id, client_session_id, group_id, project_id,
            client_kind, client_attestation, wire_protocol, model, provider, models, providers,
            request_count, turn_count, side_call_count, side_call_cost_microdollars,
            error_count, rejected_count, streaming_count,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
            cost_microdollars, p50_latency_ms, p95_latency_ms, max_latency_ms, active_ms,
            tool_calls_intended, tool_calls_executed, tool_calls_failed, artifact_count,
            artifact_files, artifact_cards,
            safety_findings, safety_blocked, gov_allow, gov_warn, gov_deny,
            prompt_count, hook_event_count, hook_status, skill_invocations, skills,
            first_at, last_at, duration_seconds, refreshed_at)
        SELECT a.context_id, a.user_id, a.session_id, a.client_session_id, rs.group_id, rs.project_id,
               COALESCE(a.client_kind, 'unknown'), COALESCE(a.client_attestation, 'unknown'),
               COALESCE(a.wire_protocol, 'unknown'), a.model, a.provider,
               COALESCE(u.models, '{}'), COALESCE(u.providers, '{}'),
               a.request_count, a.turn_count, a.side_call_count, a.side_call_cost,
               a.error_count, a.rejected_count, a.streaming_count,
               a.input_tokens, a.output_tokens, a.cache_read_tokens, a.cache_creation_tokens, a.reasoning_tokens,
               a.cost_microdollars, a.p50::int, a.p95::int, a.max_latency_ms, a.active_ms,
               COALESCE(i.tool_calls_intended, 0), COALESCE(x.executed, 0), COALESCE(x.failed, 0), COALESCE(x.artifacts, 0),
               COALESCE(x.artifact_files, 0), COALESCE(x.artifact_cards, 0),
               COALESCE(s.findings, 0), COALESCE(s.blocked, 0),
               COALESCE(g.allow, 0), COALESCE(g.warn, 0), COALESCE(g.deny, 0),
               COALESCE(h.prompts, 0), COALESCE(h.hook_events, 0), ps.status,
               COALESCE(k.invocations, 0), COALESCE(k.skills, '{}'),
               a.first_at, GREATEST(a.last_at, c.updated_at),
               GREATEST(EXTRACT(EPOCH FROM GREATEST(a.last_at, c.updated_at) - a.first_at)::bigint, 0),
               clock_timestamp()
        FROM agg a
        LEFT JOIN used u ON u.context_id = a.context_id
        LEFT JOIN intents i ON i.context_id = a.context_id
        LEFT JOIN safety s ON s.context_id = a.context_id
        LEFT JOIN executions x ON x.context_id = a.context_id
        LEFT JOIN governance g ON g.context_id = a.context_id
        LEFT JOIN hooks h ON h.context_id = a.context_id
        LEFT JOIN skills k ON k.context_id = a.context_id
        LEFT JOIN ai_request_scopes rs ON rs.request_id = a.last_request_id
        LEFT JOIN user_contexts c ON c.context_id = a.context_id
        LEFT JOIN plugin_session_summaries ps ON ps.session_id = a.client_session_id
        ON CONFLICT (context_id) DO UPDATE SET
            user_id = EXCLUDED.user_id, session_id = EXCLUDED.session_id,
            client_session_id = EXCLUDED.client_session_id,
            group_id = EXCLUDED.group_id, project_id = EXCLUDED.project_id,
            client_kind = EXCLUDED.client_kind, client_attestation = EXCLUDED.client_attestation,
            wire_protocol = EXCLUDED.wire_protocol, model = EXCLUDED.model, provider = EXCLUDED.provider,
            models = EXCLUDED.models, providers = EXCLUDED.providers,
            request_count = EXCLUDED.request_count, turn_count = EXCLUDED.turn_count,
            side_call_count = EXCLUDED.side_call_count,
            side_call_cost_microdollars = EXCLUDED.side_call_cost_microdollars,
            error_count = EXCLUDED.error_count, rejected_count = EXCLUDED.rejected_count,
            streaming_count = EXCLUDED.streaming_count,
            input_tokens = EXCLUDED.input_tokens, output_tokens = EXCLUDED.output_tokens,
            cache_read_tokens = EXCLUDED.cache_read_tokens,
            cache_creation_tokens = EXCLUDED.cache_creation_tokens,
            reasoning_tokens = EXCLUDED.reasoning_tokens, cost_microdollars = EXCLUDED.cost_microdollars,
            p50_latency_ms = EXCLUDED.p50_latency_ms, p95_latency_ms = EXCLUDED.p95_latency_ms,
            max_latency_ms = EXCLUDED.max_latency_ms, active_ms = EXCLUDED.active_ms,
            tool_calls_intended = EXCLUDED.tool_calls_intended,
            tool_calls_executed = EXCLUDED.tool_calls_executed,
            tool_calls_failed = EXCLUDED.tool_calls_failed, artifact_count = EXCLUDED.artifact_count,
            artifact_files = EXCLUDED.artifact_files, artifact_cards = EXCLUDED.artifact_cards,
            safety_findings = EXCLUDED.safety_findings, safety_blocked = EXCLUDED.safety_blocked,
            gov_allow = EXCLUDED.gov_allow, gov_warn = EXCLUDED.gov_warn, gov_deny = EXCLUDED.gov_deny,
            prompt_count = EXCLUDED.prompt_count, hook_event_count = EXCLUDED.hook_event_count,
            hook_status = EXCLUDED.hook_status, skill_invocations = EXCLUDED.skill_invocations,
            skills = EXCLUDED.skills, first_at = EXCLUDED.first_at, last_at = EXCLUDED.last_at,
            duration_seconds = EXCLUDED.duration_seconds, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1)
    SELECT COUNT(*) INTO written FROM upsert;
    -- Why: a skill row this run did not rewrite belongs to a skill the
    -- conversation no longer shows. Only conversations rebuilt here are
    -- judged, so a conversation whose raw events expired keeps its rows.
    DELETE FROM conversation_skill_facts s
    USING conversation_facts f
    WHERE f.context_id = s.context_id AND f.refreshed_at >= stamp AND s.refreshed_at < stamp;
    IF context_ids IS NOT NULL THEN
        DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids)
            AND NOT EXISTS (SELECT 1 FROM conversation_requests r WHERE r.context_id = f.context_id);
    END IF;
    RETURN COALESCE(written, 0);
END $$;


--
-- Name: refresh_conversation_facts_between(timestamp with time zone, timestamp with time zone); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_conversation_facts_between(from_at timestamp with time zone, to_at timestamp with time zone) RETURNS bigint
    LANGUAGE sql
    AS $$
SELECT refresh_conversation_facts(ARRAY(
    SELECT DISTINCT context_id FROM (
        SELECT r.context_id FROM ai_requests r
        WHERE (r.updated_at >= from_at AND r.updated_at < to_at)
           OR (r.created_at >= from_at AND r.created_at < to_at)
        UNION ALL
        SELECT r.context_id FROM ai_requests r
        WHERE r.client_session_id IN (
            SELECT e.session_id FROM plugin_usage_events e
            WHERE e.created_at >= from_at AND e.created_at < to_at
            UNION ALL
            SELECT d.session_id FROM governance_decisions d
            WHERE d.created_at >= from_at AND d.created_at < to_at
            UNION ALL
            SELECT x.trace_id FROM mcp_tool_executions x
            WHERE x.created_at >= from_at AND x.created_at < to_at AND x.trace_id IS NOT NULL)
        UNION ALL
        SELECT x.context_id FROM mcp_tool_executions x
        WHERE x.created_at >= from_at AND x.created_at < to_at AND x.context_id IS NOT NULL
        UNION ALL
        SELECT m.context_id FROM mcp_artifacts m
        WHERE m.created_at >= from_at AND m.created_at < to_at AND m.context_id IS NOT NULL
        UNION ALL
        SELECT d.context_id FROM governance_decisions d
        WHERE d.created_at >= from_at AND d.created_at < to_at
    ) touched
    WHERE context_id IS NOT NULL AND context_id <> '00000000-0000-0000-0000-4c4547414359'))
$$;


--
-- Name: refresh_conversation_facts_pending(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.refresh_conversation_facts_pending() RETURNS bigint
    LANGUAGE plpgsql
    AS $$
DECLARE from_at TIMESTAMPTZ; to_at TIMESTAMPTZ; written BIGINT;
BEGIN
    INSERT INTO conversation_rollup_state(id, watermark)
    VALUES (TRUE, now() - interval '3 minutes')
    ON CONFLICT (id) DO NOTHING;
    SELECT s.watermark INTO from_at FROM conversation_rollup_state s WHERE s.id FOR UPDATE;
    SELECT LEAST(now(), GREATEST(COALESCE(MIN(a.xact_start), now()), now() - interval '10 minutes'))
           - interval '30 seconds' INTO to_at
    FROM pg_stat_activity a
    WHERE a.datname = current_database() AND a.pid <> pg_backend_pid() AND a.xact_start IS NOT NULL;
    IF to_at <= from_at THEN
        RETURN 0;
    END IF;
    written := refresh_conversation_facts_between(from_at, to_at);
    UPDATE conversation_rollup_state SET watermark = to_at, updated_at = clock_timestamp() WHERE id;
    RETURN written;
END $$;


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
-- Name: request_scope_stamp_ai_requests(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.request_scope_stamp_ai_requests() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    BEGIN
        -- TG_LEVEL: the installer swaps function bodies before migrations run,
        -- so a database mid-upgrade still fires the row-form trigger into this
        -- body until the declarative phase replaces it below.
        IF TG_LEVEL = 'ROW' THEN
            INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
            SELECT NEW.id, NEW.user_id, d.primary_group_id, d.primary_project_id, 'primary'
              FROM (SELECT 1) one
              LEFT JOIN user_scope_defaults d
                ON NEW.actor_kind = 'user' AND d.user_id = NEW.user_id
            ON CONFLICT (request_id) DO NOTHING;
            RETURN NEW;
        END IF;
        INSERT INTO ai_request_scopes (request_id, user_id, group_id, project_id, source)
        SELECT r.id, r.user_id, d.primary_group_id, d.primary_project_id, 'primary'
          FROM new_rows r
          LEFT JOIN user_scope_defaults d
            ON r.actor_kind = 'user' AND d.user_id = r.user_id
        ON CONFLICT (request_id) DO NOTHING;
    EXCEPTION WHEN OTHERS THEN
        RAISE WARNING 'request_scope_stamp_ai_requests failed: %', SQLERRM;
    END;
    RETURN NULL;
END;
$$;


--
-- Name: sp_ai_request_message_count(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.sp_ai_request_message_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE ai_requests r SET message_count = r.message_count + c.n
        FROM (SELECT request_id, count(*)::integer AS n FROM new_rows GROUP BY request_id) c
        WHERE r.id = c.request_id;
    ELSE
        UPDATE ai_requests r SET message_count = GREATEST(r.message_count - c.n, 0)
        FROM (SELECT request_id, count(*)::integer AS n FROM old_rows GROUP BY request_id) c
        WHERE r.id = c.request_id;
    END IF;
    RETURN NULL;
END;
$$;


--
-- Name: tool_input_summary(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.tool_input_summary(input text) RETURNS text
    LANGUAGE plpgsql IMMUTABLE
    AS $$
DECLARE body jsonb;
BEGIN
    IF input IS NULL OR input = '' THEN RETURN NULL; END IF;
    IF left(ltrim(input), 1) <> '{' THEN RETURN left(input, 160); END IF;
    body := input::jsonb;
    RETURN COALESCE(
        NULLIF(body->>'file_path', ''), NULLIF(body->>'command', ''),
        NULLIF(body->>'pattern', ''), NULLIF(body->>'query', ''),
        NULLIF(body->>'path', ''), NULLIF(body->>'url', ''),
        NULLIF(body->>'skill', ''), NULLIF(body->>'name', ''),
        (SELECT k FROM jsonb_object_keys(body) k LIMIT 1));
EXCEPTION WHEN others THEN
    RETURN left(input, 160);
END $$;


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
-- Name: access_control_rule_validity; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.access_control_rule_validity (
    rule_id text NOT NULL,
    valid_until timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
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
-- Name: ai_request_client_evidence; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_request_client_evidence (
    ai_request_id character varying(255) NOT NULL,
    kind_source text NOT NULL,
    attested_host text,
    declared_client text,
    native_marker text,
    ua_product text,
    ua_version text,
    sdk_lang text,
    sdk_package_version text,
    sdk_runtime text,
    sdk_runtime_version text,
    sdk_os text,
    sdk_arch text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT ai_request_client_evidence_attested_host_check CHECK (((attested_host IS NULL) OR (attested_host = ANY (ARRAY['claude-code'::text, 'claude-desktop'::text, 'codex'::text, 'opencode'::text, 'hermes'::text, 'pi'::text, 'other'::text, 'internal'::text, 'unknown'::text])))),
    CONSTRAINT ai_request_client_evidence_declared_client_check CHECK ((length(declared_client) <= 64)),
    CONSTRAINT ai_request_client_evidence_kind_source_check CHECK ((kind_source = ANY (ARRAY['host-token'::text, 'bridge-secret'::text, 'declared'::text, 'native-marker'::text, 'user-agent'::text, 'none'::text, 'internal'::text, 'unknown'::text]))),
    CONSTRAINT ai_request_client_evidence_native_marker_check CHECK (((native_marker IS NULL) OR (native_marker = ANY (ARRAY['claude-desktop-entrypoint'::text, 'claude-cli-entrypoint'::text, 'claude-metadata-user-id'::text, 'claude-metadata-json'::text, 'codex-turn-metadata'::text])))),
    CONSTRAINT ai_request_client_evidence_sdk_arch_check CHECK ((length(sdk_arch) <= 64)),
    CONSTRAINT ai_request_client_evidence_sdk_lang_check CHECK ((length(sdk_lang) <= 64)),
    CONSTRAINT ai_request_client_evidence_sdk_os_check CHECK ((length(sdk_os) <= 64)),
    CONSTRAINT ai_request_client_evidence_sdk_package_version_check CHECK ((length(sdk_package_version) <= 64)),
    CONSTRAINT ai_request_client_evidence_sdk_runtime_check CHECK ((length(sdk_runtime) <= 64)),
    CONSTRAINT ai_request_client_evidence_sdk_runtime_version_check CHECK ((length(sdk_runtime_version) <= 64)),
    CONSTRAINT ai_request_client_evidence_ua_product_check CHECK ((length(ua_product) <= 64)),
    CONSTRAINT ai_request_client_evidence_ua_version_check CHECK ((length(ua_version) <= 64))
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
    offered_tools_sha256 text,
    prepared_tools_sha256 text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: ai_request_scopes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_request_scopes (
    request_id text NOT NULL,
    user_id text NOT NULL,
    group_id text,
    project_id text,
    source text DEFAULT 'primary'::text NOT NULL,
    resolved_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT ai_request_scopes_source_check CHECK ((source = ANY (ARRAY['primary'::text, 'header'::text])))
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
    served_provider text,
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
    finish_reason text,
    cache_hit boolean DEFAULT false NOT NULL,
    cache_read_tokens integer,
    cache_creation_tokens integer,
    reasoning_tokens integer,
    is_streaming boolean DEFAULT false NOT NULL,
    status character varying(255) DEFAULT 'pending'::character varying NOT NULL,
    error_message text,
    accounting_failed_at timestamp with time zone,
    accounting_error text,
    actor_kind text NOT NULL,
    actor_id text NOT NULL,
    synthetic boolean DEFAULT false NOT NULL,
    request_kind text DEFAULT 'turn'::text NOT NULL,
    client_kind text DEFAULT 'unknown'::text NOT NULL,
    wire_protocol text DEFAULT 'unknown'::text NOT NULL,
    client_attestation text DEFAULT 'unknown'::text NOT NULL,
    message_count integer DEFAULT 0 NOT NULL,
    instance_id character varying(255),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT ai_requests_actor_id_check CHECK ((length(actor_id) > 0)),
    CONSTRAINT ai_requests_actor_kind_check CHECK ((actor_kind = ANY (ARRAY['user'::text, 'job'::text, 'mcp'::text]))),
    CONSTRAINT ai_requests_client_attestation_check CHECK ((client_attestation = ANY (ARRAY['host-token'::text, 'bridge-secret'::text, 'declared'::text, 'native-marker'::text, 'user-agent'::text, 'none'::text, 'internal'::text, 'unknown'::text]))),
    CONSTRAINT ai_requests_client_kind_check CHECK ((client_kind = ANY (ARRAY['claude-code'::text, 'claude-desktop'::text, 'codex'::text, 'opencode'::text, 'hermes'::text, 'pi'::text, 'other'::text, 'internal'::text, 'unknown'::text]))),
    CONSTRAINT ai_requests_request_kind_check CHECK ((request_kind = ANY (ARRAY['turn'::text, 'probe'::text, 'utility'::text]))),
    CONSTRAINT ai_requests_routed_has_provider CHECK ((((status)::text = 'rejected'::text) OR ((provider IS NOT NULL) AND (model IS NOT NULL)))),
    CONSTRAINT ai_requests_wire_protocol_check CHECK ((wire_protocol = ANY (ARRAY['anthropic.messages'::text, 'openai.chat'::text, 'openai.responses'::text, 'internal'::text, 'unknown'::text])))
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
-- Name: ai_tool_catalogs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_tool_catalogs (
    sha256 text NOT NULL,
    tools jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT ai_tool_catalogs_sha256_check CHECK ((sha256 ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: analysis_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.analysis_reports (
    id text NOT NULL,
    scope_kind text NOT NULL,
    scope_id text,
    scope_label text,
    window_start timestamp with time zone NOT NULL,
    window_end timestamp with time zone NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    requested_by text NOT NULL,
    provider text,
    model text,
    ai_request_id text,
    input_tokens integer,
    output_tokens integer,
    cost_microdollars bigint,
    inputs jsonb DEFAULT '{}'::jsonb NOT NULL,
    findings jsonb,
    attempts integer DEFAULT 0 NOT NULL,
    lease_token text,
    lease_until timestamp with time zone,
    next_attempt timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    last_error text,
    generated_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    CONSTRAINT analysis_reports_scope_kind_check CHECK ((scope_kind = ANY (ARRAY['global'::text, 'marketplace'::text, 'skill'::text, 'filter'::text]))),
    CONSTRAINT analysis_reports_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'generated'::text, 'failed'::text])))
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
    CONSTRAINT managed_invocation_attributions_traffic_class_check CHECK ((traffic_class = ANY (ARRAY['production'::text, 'fixture'::text])))
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
-- Name: artifact_payloads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.artifact_payloads (
    sha256 character(64) NOT NULL,
    byte_len integer NOT NULL,
    body jsonb NOT NULL,
    ref_count integer DEFAULT 0 NOT NULL,
    first_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


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
-- Name: conversation_analyses; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.conversation_analyses (
    context_id text NOT NULL,
    user_id text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    category text,
    summary text,
    tags text[] DEFAULT '{}'::text[] NOT NULL,
    skills_used text[] DEFAULT '{}'::text[] NOT NULL,
    outcome text,
    confidence real,
    provider text,
    model text,
    ai_request_id text,
    source_request_count bigint DEFAULT 0 NOT NULL,
    source_last_at timestamp with time zone,
    classified_at timestamp with time zone,
    attempts integer DEFAULT 0 NOT NULL,
    next_attempt timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    lease_token text,
    lease_until timestamp with time zone,
    last_error text,
    title text,
    completion smallint,
    completion_rationale text,
    input_tokens integer,
    output_tokens integer,
    cost_microdollars bigint,
    trigger text DEFAULT 'automatic'::text NOT NULL,
    requested_by text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    CONSTRAINT conversation_analyses_category_check CHECK ((category = ANY (ARRAY['development'::text, 'business-analysis'::text, 'operations'::text, 'admin-config'::text, 'writing-comms'::text, 'research-learning'::text, 'other'::text]))),
    CONSTRAINT conversation_analyses_completion_check CHECK (((completion >= 0) AND (completion <= 100))),
    CONSTRAINT conversation_analyses_confidence_check CHECK (((confidence >= (0)::double precision) AND (confidence <= (1)::double precision))),
    CONSTRAINT conversation_analyses_outcome_check CHECK ((outcome = ANY (ARRAY['achieved'::text, 'partial'::text, 'abandoned'::text, 'unclear'::text]))),
    CONSTRAINT conversation_analyses_status_check CHECK ((status = ANY (ARRAY['pending'::text, 'classified'::text, 'failed'::text]))),
    CONSTRAINT conversation_analyses_trigger_check CHECK ((trigger = ANY (ARRAY['automatic'::text, 'manual'::text])))
);


--
-- Name: conversation_facts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.conversation_facts (
    context_id text NOT NULL,
    user_id text NOT NULL,
    session_id text,
    client_session_id text,
    group_id text,
    project_id text,
    client_kind text DEFAULT 'unknown'::text NOT NULL,
    client_attestation text DEFAULT 'unknown'::text NOT NULL,
    wire_protocol text DEFAULT 'unknown'::text NOT NULL,
    model text,
    provider text,
    models text[] DEFAULT '{}'::text[] NOT NULL,
    providers text[] DEFAULT '{}'::text[] NOT NULL,
    request_count bigint DEFAULT 0 NOT NULL,
    turn_count bigint DEFAULT 0 NOT NULL,
    side_call_count bigint DEFAULT 0 NOT NULL,
    side_call_cost_microdollars bigint DEFAULT 0 NOT NULL,
    error_count bigint DEFAULT 0 NOT NULL,
    rejected_count bigint DEFAULT 0 NOT NULL,
    streaming_count bigint DEFAULT 0 NOT NULL,
    input_tokens bigint DEFAULT 0 NOT NULL,
    output_tokens bigint DEFAULT 0 NOT NULL,
    cache_read_tokens bigint DEFAULT 0 NOT NULL,
    cache_creation_tokens bigint DEFAULT 0 NOT NULL,
    reasoning_tokens bigint DEFAULT 0 NOT NULL,
    cost_microdollars bigint DEFAULT 0 NOT NULL,
    p50_latency_ms integer,
    p95_latency_ms integer,
    max_latency_ms integer,
    active_ms bigint DEFAULT 0 NOT NULL,
    tool_calls_intended bigint DEFAULT 0 NOT NULL,
    tool_calls_executed bigint DEFAULT 0 NOT NULL,
    tool_calls_failed bigint DEFAULT 0 NOT NULL,
    artifact_count bigint DEFAULT 0 NOT NULL,
    artifact_files bigint DEFAULT 0 NOT NULL,
    artifact_cards bigint DEFAULT 0 NOT NULL,
    safety_findings bigint DEFAULT 0 NOT NULL,
    safety_blocked bigint DEFAULT 0 NOT NULL,
    gov_allow bigint DEFAULT 0 NOT NULL,
    gov_warn bigint DEFAULT 0 NOT NULL,
    gov_deny bigint DEFAULT 0 NOT NULL,
    prompt_count bigint DEFAULT 0 NOT NULL,
    hook_event_count bigint DEFAULT 0 NOT NULL,
    hook_status text,
    skill_invocations bigint DEFAULT 0 NOT NULL,
    skills text[] DEFAULT '{}'::text[] NOT NULL,
    first_at timestamp with time zone NOT NULL,
    last_at timestamp with time zone NOT NULL,
    duration_seconds bigint DEFAULT 0 NOT NULL,
    refreshed_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);


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
    public.conversation_request_kind(r.request_kind, (p.offered_tools_sha256 IS NOT NULL), t.thread_requests) AS effective_kind
   FROM ((public.ai_requests r
     LEFT JOIN public.ai_request_payloads p ON ((p.ai_request_id = r.id)))
     JOIN threads t ON ((((t.context_id)::text = (r.context_id)::text) AND (NOT ((t.gateway_conversation_id)::text IS DISTINCT FROM (r.gateway_conversation_id)::text)))))
  WHERE ((r.context_id)::text <> '00000000-0000-0000-0000-4c4547414359'::text);


--
-- Name: conversation_rollup_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.conversation_rollup_state (
    id boolean DEFAULT true NOT NULL,
    watermark timestamp with time zone NOT NULL,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    CONSTRAINT conversation_rollup_state_id_check CHECK (id)
);


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
-- Name: conversation_skill_facts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.conversation_skill_facts (
    context_id text NOT NULL,
    plugin_id text NOT NULL,
    skill text NOT NULL,
    user_id text NOT NULL,
    marketplace_id text,
    marketplace_hash text,
    invocations bigint DEFAULT 0 NOT NULL,
    failures bigint DEFAULT 0 NOT NULL,
    first_invoked_at timestamp with time zone NOT NULL,
    last_invoked_at timestamp with time zone NOT NULL,
    refreshed_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);


--
-- Name: conversation_skill_uses; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.conversation_skill_uses AS
 SELECT session_id AS client_session_id,
    skill,
    count(*) AS invocations,
    min(invoked_at) AS first_at
   FROM public.analysis_skill_events e
  WHERE (skill IS NOT NULL)
  GROUP BY session_id, skill;


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
-- Name: departments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.departments (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    org_id text,
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
    consumer text,
    fact jsonb,
    processed_at timestamp with time zone,
    deliver_to_origin boolean DEFAULT false NOT NULL,
    CONSTRAINT event_outbox_actor_id_nonempty CHECK ((length(actor_id) > 0)),
    CONSTRAINT event_outbox_actor_kind_check CHECK ((actor_kind = ANY (ARRAY['user'::text, 'job'::text, 'mcp'::text]))),
    CONSTRAINT event_outbox_fact_pair CHECK ((((consumer IS NULL) AND (fact IS NULL) AND (processed_at IS NULL)) OR ((consumer IS NOT NULL) AND (length(consumer) > 0) AND (fact IS NOT NULL))))
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
-- Name: gateway_routes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.gateway_routes (
    id text NOT NULL,
    "position" integer NOT NULL,
    name text,
    description text,
    model_pattern text NOT NULL,
    provider text NOT NULL,
    upstream_model text,
    extra_headers jsonb DEFAULT '{}'::jsonb NOT NULL,
    pricing jsonb,
    when_match jsonb,
    requires jsonb,
    fallback_provider text,
    fallback_upstream_model text,
    explicit_id boolean DEFAULT false NOT NULL,
    source text DEFAULT 'code'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT gateway_routes_source_check CHECK ((source = ANY (ARRAY['code'::text, 'dashboard'::text])))
);


--
-- Name: governance_chain; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.governance_chain (
    policy_id text NOT NULL,
    "position" integer NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    mode text NOT NULL,
    entry jsonb NOT NULL,
    source text DEFAULT 'code'::text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT governance_chain_mode_check CHECK ((mode = ANY (ARRAY['enforce'::text, 'warn'::text]))),
    CONSTRAINT governance_chain_source_check CHECK ((source = ANY (ARRAY['code'::text, 'dashboard'::text])))
);


--
-- Name: governance_chain_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.governance_chain_settings (
    singleton boolean DEFAULT true NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    mode text NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT governance_chain_settings_mode_check CHECK ((mode = ANY (ARRAY['enforce'::text, 'warn'::text]))),
    CONSTRAINT governance_chain_settings_singleton_check CHECK (singleton)
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
    tool_use_id text,
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
    valid_from timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    valid_until timestamp with time zone,
    revoked_at timestamp with time zone,
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
    CONSTRAINT log_level_check CHECK (((level)::text = ANY ((ARRAY['ERROR'::character varying, 'WARN'::character varying, 'INFO'::character varying, 'DEBUG'::character varying, 'TRACE'::character varying])::text[])))
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
-- Name: managed_consumer_credentials; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_consumer_credentials (
    device_id text NOT NULL,
    issuance_operation text,
    credential_digest text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    revoked_at timestamp with time zone,
    CONSTRAINT managed_consumer_credentials_credential_digest_check CHECK ((credential_digest ~ '^[0-9a-f]{64}$'::text))
);


--
-- Name: managed_consumer_grants; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_consumer_grants (
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    consumer_id text NOT NULL,
    granted_at timestamp with time zone DEFAULT now() NOT NULL,
    revoked_at timestamp with time zone
);


--
-- Name: managed_consumer_session_bindings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_consumer_session_bindings (
    id text NOT NULL,
    receipt_id text NOT NULL,
    consumer_id text NOT NULL,
    device_id text NOT NULL,
    host text NOT NULL,
    native_session_id text NOT NULL,
    bound_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT managed_consumer_session_bindings_native_session_id_check CHECK (((length(native_session_id) >= 1) AND (length(native_session_id) <= 512)))
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
-- Name: managed_installation_coverage; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_installation_coverage (
    owner_id text NOT NULL,
    resource_id text NOT NULL,
    body jsonb NOT NULL,
    generation bigint NOT NULL
);


--
-- Name: managed_installation_coverage_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_installation_coverage_state (
    owner_id text NOT NULL,
    generation bigint DEFAULT 0 NOT NULL,
    observed_at timestamp with time zone
);


--
-- Name: managed_installation_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_installation_receipts (
    id text NOT NULL,
    consumer_id text,
    device_id text,
    host text,
    consumer_evidence jsonb,
    fully_verified boolean,
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
-- Name: managed_inventory_authoring_heads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_inventory_authoring_heads (
    owner_id text NOT NULL,
    entry_id text NOT NULL,
    revision_id text NOT NULL
);


--
-- Name: managed_inventory_bindings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_inventory_bindings (
    owner_id text NOT NULL,
    entry_id text NOT NULL,
    resource_id text NOT NULL,
    bound_by text NOT NULL,
    bound_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: managed_inventory_entries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_inventory_entries (
    owner_id text NOT NULL,
    entry_id text NOT NULL,
    kind text NOT NULL,
    resource_key text NOT NULL,
    origin text NOT NULL,
    configured_key text,
    resource_id text,
    source_id text,
    availability text NOT NULL,
    latest_revision_id text,
    published_revision_id text,
    diagnostic text,
    first_observed_at timestamp with time zone NOT NULL,
    last_observed_at timestamp with time zone NOT NULL,
    generation bigint NOT NULL
);


--
-- Name: managed_inventory_membership; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_inventory_membership (
    owner_id text NOT NULL,
    entry_id text NOT NULL,
    effective_from timestamp with time zone NOT NULL,
    effective_until timestamp with time zone,
    record jsonb NOT NULL,
    CONSTRAINT managed_inventory_membership_check CHECK (((effective_until IS NULL) OR (effective_until >= effective_from)))
);


--
-- Name: managed_inventory_observations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_inventory_observations (
    owner_id text NOT NULL,
    generation bigint NOT NULL,
    observed_at timestamp with time zone NOT NULL,
    entries bigint NOT NULL,
    sources jsonb DEFAULT '{}'::jsonb NOT NULL
);


--
-- Name: managed_inventory_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.managed_inventory_state (
    owner_id text NOT NULL,
    generation bigint DEFAULT 0 NOT NULL,
    observed_at timestamp with time zone,
    entries bigint DEFAULT 0 NOT NULL,
    last_error text,
    sources jsonb DEFAULT '{}'::jsonb NOT NULL
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
-- Name: marketplace_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.marketplace_versions (
    marketplace_id text NOT NULL,
    content_hash text NOT NULL,
    source text NOT NULL,
    source_hash text,
    manifest jsonb,
    plugin_count integer DEFAULT 0 NOT NULL,
    skill_count integer DEFAULT 0 NOT NULL,
    first_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_seen_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    effective_until timestamp with time zone,
    origin text DEFAULT 'manifest'::text NOT NULL,
    CONSTRAINT marketplace_versions_origin_check CHECK ((origin = ANY (ARRAY['manifest'::text, 'legacy_source_hash'::text])))
);


--
-- Name: mcp_artifact_findings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.mcp_artifact_findings (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    artifact_id character varying(255) NOT NULL,
    phase character varying(32) NOT NULL,
    severity character varying(16) NOT NULL,
    category character varying(64) NOT NULL,
    scanner character varying(64) NOT NULL,
    path text,
    excerpt text,
    redacted boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
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
    session_id character varying(255),
    trace_id character varying(255),
    ai_tool_call_id character varying(255),
    server_name character varying(255) NOT NULL,
    tool_name character varying(255),
    artifact_type character varying(100) NOT NULL,
    title character varying(500),
    source character varying(32) DEFAULT 'in_process'::character varying NOT NULL,
    last_seen_source character varying(32),
    data jsonb NOT NULL,
    metadata jsonb,
    payload_sha256 character(64),
    payload_bytes integer,
    is_structured boolean DEFAULT false NOT NULL,
    has_ui_resource boolean DEFAULT false NOT NULL,
    is_error boolean DEFAULT false NOT NULL,
    secret_redactions integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expires_at timestamp with time zone,
    CONSTRAINT mcp_artifacts_source_check CHECK (((source)::text = ANY ((ARRAY['in_process'::character varying, 'proxy'::character varying, 'gateway'::character varying, 'hook_claude_code'::character varying, 'hook_opencode'::character varying])::text[])))
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
    roles jsonb DEFAULT '[]'::jsonb NOT NULL,
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
    source character varying(32) DEFAULT 'in_process'::character varying NOT NULL,
    correlation character varying(16) DEFAULT 'exact'::character varying NOT NULL,
    payload_sha256 character(64),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT mcp_tool_executions_correlation_check CHECK (((correlation)::text = ANY ((ARRAY['exact'::character varying, 'inferred'::character varying])::text[]))),
    CONSTRAINT mcp_tool_executions_source_check CHECK (((source)::text = ANY ((ARRAY['in_process'::character varying, 'proxy'::character varying, 'gateway'::character varying, 'hook_claude_code'::character varying, 'hook_opencode'::character varying])::text[]))),
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
    owner_user_id text NOT NULL,
    registration_token_hash text
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
-- Name: organization_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.organization_members (
    user_id text NOT NULL,
    org_id text NOT NULL,
    org_role text DEFAULT 'member'::text NOT NULL,
    joined_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT organization_members_org_role_check CHECK ((org_role = ANY (ARRAY['owner'::text, 'admin'::text, 'member'::text])))
);


--
-- Name: organizations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.organizations (
    id text DEFAULT (gen_random_uuid())::text NOT NULL,
    slug text NOT NULL,
    name text NOT NULL,
    plan_id text,
    seat_limit_override integer,
    status text DEFAULT 'active'::text NOT NULL,
    is_platform boolean DEFAULT false NOT NULL,
    email_domains text[] DEFAULT ARRAY[]::text[] NOT NULL,
    contract_start date,
    contract_end date,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT organizations_status_check CHECK ((status = ANY (ARRAY['active'::text, 'suspended'::text, 'cancelled'::text])))
);


--
-- Name: otlp_export_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.otlp_export_state (
    signal text NOT NULL,
    watermark timestamp with time zone DEFAULT now() NOT NULL,
    watermark_id text DEFAULT ''::text NOT NULL,
    last_attempt_at timestamp with time zone,
    last_success_at timestamp with time zone,
    last_error text,
    last_error_at timestamp with time zone,
    batches_total bigint DEFAULT 0 NOT NULL,
    failures_total bigint DEFAULT 0 NOT NULL,
    rows_total bigint DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT otlp_export_state_signal_check CHECK ((signal = ANY (ARRAY['traces'::text, 'logs'::text])))
);


--
-- Name: plans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.plans (
    id text NOT NULL,
    name text NOT NULL,
    description text DEFAULT ''::text NOT NULL,
    seat_limit integer,
    monthly_cost_cap_microdollars bigint,
    monthly_price_microdollars bigint DEFAULT 0 NOT NULL,
    grants jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
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
    valid_from timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    valid_until timestamp with time zone,
    revoked_at timestamp with time zone,
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
-- Name: report_agent_tasks; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_agent_tasks AS
 SELECT task_id,
    context_id,
    status,
    status_timestamp,
    user_id,
    session_id,
    trace_id,
    agent_name,
    started_at,
    completed_at,
    execution_time_ms,
    error_message,
    metadata,
    version,
    created_at,
    updated_at
   FROM public.agent_tasks src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = src.user_id) AND (d.status = 'deleted'::text)))));


--
-- Name: report_ai_requests; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_ai_requests AS
 SELECT id,
    request_id,
    user_id,
    session_id,
    task_id,
    context_id,
    gateway_conversation_id,
    client_session_id,
    provider_request_id,
    trace_id,
    mcp_execution_id,
    provider,
    served_provider,
    model,
    requested_model,
    system_prompt_override,
    route_match,
    temperature,
    top_p,
    max_tokens,
    stop_sequences,
    tokens_used,
    input_tokens,
    output_tokens,
    cost_microdollars,
    latency_ms,
    upstream_latency_ms,
    finish_reason,
    cache_hit,
    cache_read_tokens,
    cache_creation_tokens,
    reasoning_tokens,
    is_streaming,
    status,
    error_message,
    accounting_failed_at,
    accounting_error,
    actor_kind,
    actor_id,
    synthetic,
    request_kind,
    client_kind,
    wire_protocol,
    client_attestation,
    message_count,
    instance_id,
    created_at,
    updated_at,
    completed_at
   FROM public.ai_requests src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


--
-- Name: report_analytics_events; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_analytics_events AS
 SELECT id,
    user_id,
    session_id,
    context_id,
    gateway_conversation_id,
    provider_request_id,
    event_type,
    event_category,
    severity,
    endpoint,
    error_code,
    response_time_ms,
    agent_id,
    task_id,
    message,
    metadata,
    event_data,
    "timestamp"
   FROM public.analytics_events src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


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
-- Name: report_bot_sessions; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_bot_sessions AS
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
    bot_type
   FROM public.v_bot_sessions src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


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
-- Name: report_clean_traffic; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_clean_traffic AS
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
   FROM public.v_clean_traffic src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


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
-- Name: report_engaged_traffic; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_engaged_traffic AS
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
   FROM public.v_engaged_traffic src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


--
-- Name: report_markdown_content; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_markdown_content AS
 SELECT id,
    slug,
    locale,
    title,
    description,
    body,
    author,
    published_at,
    keywords,
    kind,
    image,
    category_id,
    source_id,
    version_hash,
    public,
    links,
    updated_at
   FROM public.markdown_content src;


--
-- Name: report_mcp_tool_executions; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_mcp_tool_executions AS
 SELECT mcp_execution_id,
    tool_name,
    server_name,
    started_at,
    completed_at,
    execution_time_ms,
    input,
    output,
    output_schema,
    status,
    error_message,
    user_id,
    session_id,
    context_id,
    task_id,
    trace_id,
    request_method,
    request_source,
    actor_kind,
    actor_id,
    ai_tool_call_id,
    source,
    correlation,
    payload_sha256,
    created_at
   FROM public.mcp_tool_executions src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


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
-- Name: report_task_messages; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_task_messages AS
 SELECT id,
    task_id,
    message_id,
    client_message_id,
    role,
    context_id,
    user_id,
    session_id,
    trace_id,
    sequence_number,
    created_at,
    updated_at,
    metadata,
    reference_task_ids
   FROM public.task_messages src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = src.user_id) AND (d.status = 'deleted'::text)))));


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
-- Name: report_user_contexts; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_user_contexts AS
 SELECT context_id,
    user_id,
    session_id,
    name,
    kind,
    created_at,
    updated_at
   FROM public.user_contexts src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = src.user_id) AND (d.status = 'deleted'::text)))));


--
-- Name: report_user_sessions; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_user_sessions AS
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
   FROM public.user_sessions src
  WHERE (NOT (EXISTS ( SELECT 1
           FROM public.users d
          WHERE ((d.id = (src.user_id)::text) AND (d.status = 'deleted'::text)))));


--
-- Name: report_users; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.report_users AS
 SELECT id,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    is_bot,
    is_scanner,
    avatar_url,
    created_at,
    updated_at
   FROM public.users src
  WHERE (status <> 'deleted'::text);


--
-- Name: retention_archives; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.retention_archives (
    id bigint NOT NULL,
    tier text NOT NULL,
    period text NOT NULL,
    table_name text NOT NULL,
    relative_path text NOT NULL,
    row_count bigint NOT NULL,
    byte_count bigint NOT NULL,
    sha256 text NOT NULL,
    window_from timestamp with time zone NOT NULL,
    window_to timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT retention_archives_tier_check CHECK ((tier = ANY (ARRAY['weekly'::text, 'monthly'::text])))
);


--
-- Name: retention_archives_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.retention_archives_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: retention_archives_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.retention_archives_id_seq OWNED BY public.retention_archives.id;


--
-- Name: retention_health_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.retention_health_reports (
    id bigint NOT NULL,
    run_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    report jsonb NOT NULL,
    findings_p1 integer DEFAULT 0 NOT NULL,
    findings_p2 integer DEFAULT 0 NOT NULL,
    findings_p3 integer DEFAULT 0 NOT NULL
);


--
-- Name: retention_health_reports_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.retention_health_reports_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: retention_health_reports_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.retention_health_reports_id_seq OWNED BY public.retention_health_reports.id;


--
-- Name: retention_runs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.retention_runs (
    id bigint NOT NULL,
    run_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    tier text NOT NULL,
    table_name text NOT NULL,
    live_rows bigint NOT NULL,
    dead_rows bigint NOT NULL,
    total_bytes bigint NOT NULL,
    index_bytes bigint NOT NULL,
    oldest_row timestamp with time zone,
    window_days integer,
    CONSTRAINT retention_runs_tier_check CHECK ((tier = ANY (ARRAY['daily'::text, 'weekly'::text, 'monthly'::text])))
);


--
-- Name: retention_runs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.retention_runs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: retention_runs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.retention_runs_id_seq OWNED BY public.retention_runs.id;


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
    last_message text,
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
-- Name: service_owned_ids; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.service_owned_ids (
    kind text NOT NULL,
    id text NOT NULL,
    source text NOT NULL,
    marketplace_id text,
    recorded_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT service_owned_ids_kind_check CHECK ((kind = ANY (ARRAY['marketplace'::text, 'plugin'::text, 'skill'::text])))
);


--
-- Name: service_sources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.service_sources (
    name text NOT NULL,
    kind text NOT NULL,
    content_hash text,
    digest text,
    version text,
    provenance text NOT NULL,
    recorded_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT service_sources_kind_check CHECK ((kind = ANY (ARRAY['base'::text, 'bundle'::text])))
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
-- Name: sync_state; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sync_state (
    plane text NOT NULL,
    declared_hash text NOT NULL,
    applied_hash text,
    applied_mode text,
    applied_at timestamp with time zone,
    applied_by text,
    base_tree_hash text,
    composed_hash text,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT sync_state_applied_mode_check CHECK ((applied_mode = ANY (ARRAY['seed'::text, 'insert_only'::text, 'overwrite'::text])))
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
-- Name: tool_call_ledger; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.tool_call_ledger AS
 SELECT COALESCE(i.ai_tool_call_id, e.ai_tool_call_id) AS ai_tool_call_id,
    i.id AS intent_id,
    i.request_id,
    e.mcp_execution_id,
    a.artifact_id,
    COALESCE(r.user_id, e.user_id, a.user_id) AS user_id,
    COALESCE(r.session_id, e.session_id, a.session_id) AS session_id,
    COALESCE(r.context_id, e.context_id, a.context_id) AS context_id,
    COALESCE(r.trace_id, e.trace_id, a.trace_id) AS trace_id,
    r.client_kind,
    COALESCE(i.tool_name, e.tool_name, a.tool_name) AS tool_name,
    COALESCE(e.server_name, a.server_name) AS server_name,
    i.created_at AS intended_at,
    e.started_at AS executed_at,
    e.completed_at,
    e.execution_time_ms,
    e.status AS execution_status,
    e.error_message,
    e.source,
    e.correlation,
    a.artifact_type,
    a.title AS artifact_title,
    COALESCE(a.is_structured, false) AS is_structured,
    COALESCE(a.has_ui_resource, false) AS has_ui_resource,
    COALESCE(a.is_error, false) AS is_error,
    a.payload_bytes,
    a.secret_redactions,
        CASE
            WHEN (i.id IS NULL) THEN 'unattested'::text
            WHEN (e.mcp_execution_id IS NULL) THEN 'intended'::text
            ELSE 'executed'::text
        END AS state,
    COALESCE(e.started_at, i.created_at) AS occurred_at,
    ((e.mcp_execution_id IS NOT NULL) AND ((e.server_name)::text = (e.source)::text)) AS is_builtin
   FROM (((public.ai_request_tool_calls i
     FULL JOIN public.mcp_tool_executions e ON (((e.ai_tool_call_id)::text = (i.ai_tool_call_id)::text)))
     LEFT JOIN public.mcp_artifacts a ON (((a.mcp_execution_id)::text = e.mcp_execution_id)))
     LEFT JOIN public.ai_requests r ON ((r.id = (i.request_id)::text)));


--
-- Name: tool_activity; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.tool_activity AS
 SELECT l.ai_tool_call_id,
    l.intent_id,
    l.request_id,
    l.mcp_execution_id,
    l.artifact_id,
    l.user_id,
    l.session_id,
    l.context_id,
    l.trace_id,
    l.client_kind,
    l.tool_name,
    l.server_name,
    l.intended_at,
    l.executed_at,
    l.completed_at,
    l.execution_time_ms,
    l.execution_status,
    l.error_message,
    l.source,
    l.correlation,
    l.artifact_type,
    l.artifact_title,
    l.is_structured,
    l.has_ui_resource,
    l.is_error,
    l.payload_bytes,
    l.secret_redactions,
    l.state,
    l.occurred_at,
    x.context_id AS execution_context_id,
    x.trace_id AS execution_trace_id,
    COALESCE(a.payload_sha256, x.payload_sha256) AS payload_sha256,
    public.artifact_kind((l.tool_name)::text, (l.artifact_type)::text, l.has_ui_resource, l.is_structured, (COALESCE(a.payload_sha256, x.payload_sha256))::text, x.input, b.is_builtin) AS artifact_kind,
    public.tool_input_summary(x.input) AS input_summary,
    b.is_builtin
   FROM (((public.tool_call_ledger l
     LEFT JOIN public.mcp_tool_executions x ON ((x.mcp_execution_id = l.mcp_execution_id)))
     LEFT JOIN public.mcp_artifacts a ON (((a.artifact_id)::text = (l.artifact_id)::text)))
     CROSS JOIN LATERAL ( SELECT ((l.server_name IS NOT NULL) AND ((l.server_name)::text = (l.source)::text) AND ((COALESCE(l.tool_name, ''::character varying))::text !~~ 'mcp\_\_%'::text)) AS is_builtin) b);


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
-- Name: user_device_cert_validity; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_device_cert_validity (
    device_id text NOT NULL,
    valid_until timestamp with time zone NOT NULL,
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
-- Name: user_groups; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.user_groups AS
 SELECT DISTINCT gm.user_id,
    gm.group_id
   FROM public.group_members gm
  WHERE ((gm.revoked_at IS NULL) AND (gm.valid_from <= CURRENT_TIMESTAMP) AND ((gm.valid_until IS NULL) OR (gm.valid_until > CURRENT_TIMESTAMP)))
UNION ALL
 SELECT u.id AS user_id,
    'unassigned'::text AS group_id
   FROM public.users u
  WHERE ((NOT ('anonymous'::text = ANY (u.roles))) AND (NOT (EXISTS ( SELECT 1
           FROM public.group_members gm
          WHERE ((gm.user_id = u.id) AND (gm.revoked_at IS NULL) AND (gm.valid_from <= CURRENT_TIMESTAMP) AND ((gm.valid_until IS NULL) OR (gm.valid_until > CURRENT_TIMESTAMP)))))));


--
-- Name: user_last_seen; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.user_last_seen AS
 WITH signals AS (
         SELECT ai_requests.user_id,
            max(ai_requests.created_at) AS seen_at,
            'request'::text AS source
           FROM public.ai_requests
          GROUP BY ai_requests.user_id
        UNION ALL
         SELECT user_sessions.user_id,
            max(user_sessions.last_activity_at) AS max,
            'session'::text
           FROM public.user_sessions
          WHERE (user_sessions.user_id IS NOT NULL)
          GROUP BY user_sessions.user_id
        UNION ALL
         SELECT user_activity.user_id,
            max(user_activity.created_at) AS max,
            'activity'::text
           FROM public.user_activity
          GROUP BY user_activity.user_id
        UNION ALL
         SELECT plugin_usage_events.user_id,
            max(plugin_usage_events.created_at) AS max,
            'hook'::text
           FROM public.plugin_usage_events
          GROUP BY plugin_usage_events.user_id
        UNION ALL
         SELECT mcp_tool_executions.user_id,
            max(mcp_tool_executions.started_at) AS max,
            'tool'::text
           FROM public.mcp_tool_executions
          GROUP BY mcp_tool_executions.user_id
        UNION ALL
         SELECT bridge_sessions.user_id,
            max(bridge_sessions.last_heartbeat_at) AS max,
            'bridge'::text
           FROM public.bridge_sessions
          GROUP BY bridge_sessions.user_id
        )
 SELECT DISTINCT ON (user_id) user_id,
    seen_at AS last_seen_at,
    source AS last_seen_source
   FROM signals
  WHERE (seen_at IS NOT NULL)
  ORDER BY user_id, seen_at DESC;


--
-- Name: user_manual_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_manual_roles (
    user_id text NOT NULL,
    role text NOT NULL,
    granted_by text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    valid_from timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    valid_until timestamp with time zone,
    CONSTRAINT user_manual_roles_role_check CHECK ((role = ANY (ARRAY['platform_admin'::text, 'admin'::text, 'developer'::text, 'user'::text, 'project_manager'::text, 'knowledge_worker'::text, 'super_admin'::text])))
);


--
-- Name: user_profile_ext; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_profile_ext (
    user_id text NOT NULL,
    department text DEFAULT 'Default'::text NOT NULL,
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
-- Name: user_projects; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.user_projects AS
 SELECT DISTINCT user_id,
    project_id
   FROM public.project_members pm
  WHERE ((revoked_at IS NULL) AND (valid_from <= CURRENT_TIMESTAMP) AND ((valid_until IS NULL) OR (valid_until > CURRENT_TIMESTAMP)));


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
-- Name: retention_archives id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_archives ALTER COLUMN id SET DEFAULT nextval('public.retention_archives_id_seq'::regclass);


--
-- Name: retention_health_reports id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_health_reports ALTER COLUMN id SET DEFAULT nextval('public.retention_health_reports_id_seq'::regclass);


--
-- Name: retention_runs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_runs ALTER COLUMN id SET DEFAULT nextval('public.retention_runs_id_seq'::regclass);


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
-- Name: access_control_rule_validity access_control_rule_validity_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_control_rule_validity
    ADD CONSTRAINT access_control_rule_validity_pkey PRIMARY KEY (rule_id);


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
-- Name: ai_request_client_evidence ai_request_client_evidence_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_client_evidence
    ADD CONSTRAINT ai_request_client_evidence_pkey PRIMARY KEY (ai_request_id);


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
-- Name: ai_request_scopes ai_request_scopes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_scopes
    ADD CONSTRAINT ai_request_scopes_pkey PRIMARY KEY (request_id);


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
-- Name: ai_tool_catalogs ai_tool_catalogs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_tool_catalogs
    ADD CONSTRAINT ai_tool_catalogs_pkey PRIMARY KEY (sha256);


--
-- Name: analysis_reports analysis_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.analysis_reports
    ADD CONSTRAINT analysis_reports_pkey PRIMARY KEY (id);


--
-- Name: analytics_events analytics_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.analytics_events
    ADD CONSTRAINT analytics_events_pkey PRIMARY KEY (id);


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
-- Name: artifact_payloads artifact_payloads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.artifact_payloads
    ADD CONSTRAINT artifact_payloads_pkey PRIMARY KEY (sha256);


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
-- Name: conversation_analyses conversation_analyses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.conversation_analyses
    ADD CONSTRAINT conversation_analyses_pkey PRIMARY KEY (context_id);


--
-- Name: conversation_facts conversation_facts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.conversation_facts
    ADD CONSTRAINT conversation_facts_pkey PRIMARY KEY (context_id);


--
-- Name: conversation_rollup_state conversation_rollup_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.conversation_rollup_state
    ADD CONSTRAINT conversation_rollup_state_pkey PRIMARY KEY (id);


--
-- Name: conversation_skill_facts conversation_skill_facts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.conversation_skill_facts
    ADD CONSTRAINT conversation_skill_facts_pkey PRIMARY KEY (context_id, plugin_id, skill);


--
-- Name: daily_summaries daily_summaries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.daily_summaries
    ADD CONSTRAINT daily_summaries_pkey PRIMARY KEY (user_id, summary_date);


--
-- Name: departments departments_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_name_key UNIQUE (name);


--
-- Name: departments departments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_pkey PRIMARY KEY (id);


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
-- Name: gateway_routes gateway_routes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gateway_routes
    ADD CONSTRAINT gateway_routes_pkey PRIMARY KEY (id);


--
-- Name: governance_chain governance_chain_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.governance_chain
    ADD CONSTRAINT governance_chain_pkey PRIMARY KEY (policy_id);


--
-- Name: governance_chain_settings governance_chain_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.governance_chain_settings
    ADD CONSTRAINT governance_chain_settings_pkey PRIMARY KEY (singleton);


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
-- Name: managed_consumer_credentials managed_consumer_credentials_credential_digest_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_credentials
    ADD CONSTRAINT managed_consumer_credentials_credential_digest_key UNIQUE (credential_digest);


--
-- Name: managed_consumer_credentials managed_consumer_credentials_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_credentials
    ADD CONSTRAINT managed_consumer_credentials_pkey PRIMARY KEY (device_id);


--
-- Name: managed_consumer_grants managed_consumer_grants_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_grants
    ADD CONSTRAINT managed_consumer_grants_pkey PRIMARY KEY (owner_id, resource_id, consumer_id);


--
-- Name: managed_consumer_session_bindings managed_consumer_session_bind_receipt_id_consumer_id_device_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_session_bindings
    ADD CONSTRAINT managed_consumer_session_bind_receipt_id_consumer_id_device_key UNIQUE (receipt_id, consumer_id, device_id, host, native_session_id);


--
-- Name: managed_consumer_session_bindings managed_consumer_session_bindings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_session_bindings
    ADD CONSTRAINT managed_consumer_session_bindings_pkey PRIMARY KEY (id);


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
-- Name: managed_installation_coverage managed_installation_coverage_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_coverage
    ADD CONSTRAINT managed_installation_coverage_pkey PRIMARY KEY (owner_id, resource_id);


--
-- Name: managed_installation_coverage_state managed_installation_coverage_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_coverage_state
    ADD CONSTRAINT managed_installation_coverage_state_pkey PRIMARY KEY (owner_id);


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
-- Name: managed_inventory_authoring_heads managed_inventory_authoring_heads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_authoring_heads
    ADD CONSTRAINT managed_inventory_authoring_heads_pkey PRIMARY KEY (owner_id, entry_id);


--
-- Name: managed_inventory_bindings managed_inventory_bindings_owner_id_resource_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_bindings
    ADD CONSTRAINT managed_inventory_bindings_owner_id_resource_id_key UNIQUE (owner_id, resource_id);


--
-- Name: managed_inventory_bindings managed_inventory_bindings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_bindings
    ADD CONSTRAINT managed_inventory_bindings_pkey PRIMARY KEY (owner_id, entry_id);


--
-- Name: managed_inventory_entries managed_inventory_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_entries
    ADD CONSTRAINT managed_inventory_entries_pkey PRIMARY KEY (owner_id, entry_id);


--
-- Name: managed_inventory_membership managed_inventory_membership_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_membership
    ADD CONSTRAINT managed_inventory_membership_pkey PRIMARY KEY (owner_id, entry_id, effective_from);


--
-- Name: managed_inventory_observations managed_inventory_observations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_observations
    ADD CONSTRAINT managed_inventory_observations_pkey PRIMARY KEY (owner_id, generation);


--
-- Name: managed_inventory_state managed_inventory_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_state
    ADD CONSTRAINT managed_inventory_state_pkey PRIMARY KEY (owner_id);


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
-- Name: marketplace_versions marketplace_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.marketplace_versions
    ADD CONSTRAINT marketplace_versions_pkey PRIMARY KEY (marketplace_id, content_hash);


--
-- Name: mcp_artifact_findings mcp_artifact_findings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_artifact_findings
    ADD CONSTRAINT mcp_artifact_findings_pkey PRIMARY KEY (id);


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
-- Name: organization_members organization_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.organization_members
    ADD CONSTRAINT organization_members_pkey PRIMARY KEY (user_id);


--
-- Name: organizations organizations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.organizations
    ADD CONSTRAINT organizations_pkey PRIMARY KEY (id);


--
-- Name: organizations organizations_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.organizations
    ADD CONSTRAINT organizations_slug_key UNIQUE (slug);


--
-- Name: otlp_export_state otlp_export_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.otlp_export_state
    ADD CONSTRAINT otlp_export_state_pkey PRIMARY KEY (signal);


--
-- Name: plans plans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.plans
    ADD CONSTRAINT plans_pkey PRIMARY KEY (id);


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
-- Name: retention_archives retention_archives_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_archives
    ADD CONSTRAINT retention_archives_pkey PRIMARY KEY (id);


--
-- Name: retention_archives retention_archives_tier_period_table_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_archives
    ADD CONSTRAINT retention_archives_tier_period_table_name_key UNIQUE (tier, period, table_name);


--
-- Name: retention_health_reports retention_health_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_health_reports
    ADD CONSTRAINT retention_health_reports_pkey PRIMARY KEY (id);


--
-- Name: retention_runs retention_runs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.retention_runs
    ADD CONSTRAINT retention_runs_pkey PRIMARY KEY (id);


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
-- Name: service_owned_ids service_owned_ids_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_owned_ids
    ADD CONSTRAINT service_owned_ids_pkey PRIMARY KEY (kind, id);


--
-- Name: service_sources service_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_sources
    ADD CONSTRAINT service_sources_pkey PRIMARY KEY (name);


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
-- Name: sync_state sync_state_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sync_state
    ADD CONSTRAINT sync_state_pkey PRIMARY KEY (plane);


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
-- Name: user_device_cert_validity user_device_cert_validity_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_device_cert_validity
    ADD CONSTRAINT user_device_cert_validity_pkey PRIMARY KEY (device_id);


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
-- Name: idx_access_control_entities_default; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_control_entities_default ON public.access_control_entities USING btree (default_included) WHERE (default_included = true);


--
-- Name: idx_access_control_rule_validity_until; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_control_rule_validity_until ON public.access_control_rule_validity USING btree (valid_until);


--
-- Name: idx_access_control_rules_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_access_control_rules_source ON public.access_control_rules USING btree (source);


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
-- Name: idx_ai_gateway_policies_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_gateway_policies_enabled ON public.ai_gateway_policies USING btree (enabled);


--
-- Name: idx_ai_gateway_thought_signatures_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_gateway_thought_signatures_expires_at ON public.ai_gateway_thought_signatures USING btree (expires_at);


--
-- Name: idx_ai_quota_buckets_window; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_quota_buckets_window ON public.ai_quota_buckets USING btree (window_start);


--
-- Name: idx_ai_request_messages_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_messages_role ON public.ai_request_messages USING btree (role);


--
-- Name: idx_ai_request_payloads_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_payloads_created_at ON public.ai_request_payloads USING btree (created_at);


--
-- Name: idx_ai_request_scopes_group; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_scopes_group ON public.ai_request_scopes USING btree (group_id);


--
-- Name: idx_ai_request_scopes_project; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_scopes_project ON public.ai_request_scopes USING btree (project_id);


--
-- Name: idx_ai_request_scopes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_scopes_user ON public.ai_request_scopes USING btree (user_id);


--
-- Name: idx_ai_request_tool_calls_ai_tool_call_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_tool_calls_ai_tool_call_id ON public.ai_request_tool_calls USING btree (ai_tool_call_id) WHERE (ai_tool_call_id IS NOT NULL);


--
-- Name: idx_ai_request_tool_calls_mcp_execution_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_request_tool_calls_mcp_execution_id ON public.ai_request_tool_calls USING btree (mcp_execution_id);


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
-- Name: idx_ai_requests_provider_request_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_provider_request_id ON public.ai_requests USING btree (provider_request_id);


--
-- Name: idx_ai_requests_provider_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_provider_status ON public.ai_requests USING btree (provider, status);


--
-- Name: idx_ai_requests_session_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_session_created ON public.ai_requests USING btree (session_id, created_at);


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
-- Name: idx_ai_requests_updated_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_updated_at ON public.ai_requests USING btree (updated_at);


--
-- Name: idx_ai_requests_user_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_requests_user_created ON public.ai_requests USING btree (user_id, created_at);


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
-- Name: idx_analysis_reports_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analysis_reports_created ON public.analysis_reports USING btree (created_at DESC);


--
-- Name: idx_analysis_reports_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analysis_reports_pending ON public.analysis_reports USING btree (next_attempt) WHERE (status = 'pending'::text);


--
-- Name: idx_analysis_reports_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_analysis_reports_scope ON public.analysis_reports USING btree (scope_kind, scope_id, created_at DESC);


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
-- Name: idx_artifact_parts_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_parts_context_id ON public.artifact_parts USING btree (context_id);


--
-- Name: idx_artifact_parts_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_parts_kind ON public.artifact_parts USING btree (part_kind);


--
-- Name: idx_artifact_payloads_last_seen; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_artifact_payloads_last_seen ON public.artifact_payloads USING btree (last_seen_at DESC);


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
-- Name: idx_campaign_links_source_content; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_source_content ON public.campaign_links USING btree (source_content_id);


--
-- Name: idx_campaign_links_target_url; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_campaign_links_target_url ON public.campaign_links USING btree (target_url);


--
-- Name: idx_content_files_file_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_files_file_id ON public.content_files USING btree (file_id);


--
-- Name: idx_content_files_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_content_files_role ON public.content_files USING btree (role);


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
-- Name: idx_conversation_analyses_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_analyses_category ON public.conversation_analyses USING btree (category, classified_at DESC);


--
-- Name: idx_conversation_analyses_completion; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_analyses_completion ON public.conversation_analyses USING btree (completion) WHERE (completion IS NOT NULL);


--
-- Name: idx_conversation_analyses_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_analyses_pending ON public.conversation_analyses USING btree (next_attempt) WHERE (status <> 'classified'::text);


--
-- Name: idx_conversation_analyses_skills; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_analyses_skills ON public.conversation_analyses USING gin (skills_used);


--
-- Name: idx_conversation_analyses_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_analyses_user ON public.conversation_analyses USING btree (user_id, classified_at DESC);


--
-- Name: idx_conversation_facts_client; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_client ON public.conversation_facts USING btree (client_kind, last_at DESC);


--
-- Name: idx_conversation_facts_group; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_group ON public.conversation_facts USING btree (group_id, last_at DESC);


--
-- Name: idx_conversation_facts_last_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_last_at ON public.conversation_facts USING btree (last_at DESC);


--
-- Name: idx_conversation_facts_model; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_model ON public.conversation_facts USING btree (model, last_at DESC);


--
-- Name: idx_conversation_facts_project; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_project ON public.conversation_facts USING btree (project_id, last_at DESC);


--
-- Name: idx_conversation_facts_session; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_session ON public.conversation_facts USING btree (client_session_id) WHERE (client_session_id IS NOT NULL);


--
-- Name: idx_conversation_facts_skills; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_skills ON public.conversation_facts USING gin (skills);


--
-- Name: idx_conversation_facts_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_facts_user ON public.conversation_facts USING btree (user_id, last_at DESC);


--
-- Name: idx_conversation_skill_facts_first; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_skill_facts_first ON public.conversation_skill_facts USING btree (first_invoked_at);


--
-- Name: idx_conversation_skill_facts_version; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_conversation_skill_facts_version ON public.conversation_skill_facts USING btree (marketplace_id, marketplace_hash, first_invoked_at);


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
-- Name: idx_event_outbox_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_actor ON public.event_outbox USING btree (actor_kind, actor_id);


--
-- Name: idx_event_outbox_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_created_at ON public.event_outbox USING btree (created_at);


--
-- Name: idx_event_outbox_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_pending ON public.event_outbox USING btree (consumer, created_at, id) WHERE ((consumer IS NOT NULL) AND (processed_at IS NULL));


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
-- Name: idx_gateway_routes_position; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_gateway_routes_position ON public.gateway_routes USING btree ("position");


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
-- Name: idx_governance_decisions_tool_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_tool_name ON public.governance_decisions USING btree (tool_name);


--
-- Name: idx_governance_decisions_tool_use_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_governance_decisions_tool_use_id ON public.governance_decisions USING btree (tool_use_id) WHERE (tool_use_id IS NOT NULL);


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
-- Name: idx_group_members_valid_until; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_group_members_valid_until ON public.group_members USING btree (valid_until) WHERE ((valid_until IS NOT NULL) AND (revoked_at IS NULL));


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
-- Name: idx_marketplace_versions_current; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_marketplace_versions_current ON public.marketplace_versions USING btree (marketplace_id) WHERE (effective_until IS NULL);


--
-- Name: idx_marketplace_versions_interval; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_marketplace_versions_interval ON public.marketplace_versions USING btree (marketplace_id, first_seen_at, effective_until);


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
-- Name: idx_mcp_artifact_findings_artifact; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifact_findings_artifact ON public.mcp_artifact_findings USING btree (artifact_id);


--
-- Name: idx_mcp_artifact_findings_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifact_findings_category ON public.mcp_artifact_findings USING btree (category, created_at DESC);


--
-- Name: idx_mcp_artifacts_ai_tool_call; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_ai_tool_call ON public.mcp_artifacts USING btree (ai_tool_call_id) WHERE (ai_tool_call_id IS NOT NULL);


--
-- Name: idx_mcp_artifacts_context_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_context_id ON public.mcp_artifacts USING btree (context_id) WHERE (context_id IS NOT NULL);


--
-- Name: idx_mcp_artifacts_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_created_at ON public.mcp_artifacts USING btree (created_at DESC);


--
-- Name: idx_mcp_artifacts_execution; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_mcp_artifacts_execution ON public.mcp_artifacts USING btree (mcp_execution_id);


--
-- Name: idx_mcp_artifacts_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_expires_at ON public.mcp_artifacts USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_mcp_artifacts_payload; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_payload ON public.mcp_artifacts USING btree (payload_sha256) WHERE (payload_sha256 IS NOT NULL);


--
-- Name: idx_mcp_artifacts_server_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_server_created ON public.mcp_artifacts USING btree (server_name, created_at DESC);


--
-- Name: idx_mcp_artifacts_session_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_session_created ON public.mcp_artifacts USING btree (session_id, created_at DESC) WHERE (session_id IS NOT NULL);


--
-- Name: idx_mcp_artifacts_structured; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_structured ON public.mcp_artifacts USING btree (created_at DESC) WHERE is_structured;


--
-- Name: idx_mcp_artifacts_trace; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_artifacts_trace ON public.mcp_artifacts USING btree (trace_id) WHERE (trace_id IS NOT NULL);


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
-- Name: idx_mcp_tool_executions_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_created_at ON public.mcp_tool_executions USING btree (created_at DESC);


--
-- Name: idx_mcp_tool_executions_execution_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_execution_time ON public.mcp_tool_executions USING btree (execution_time_ms DESC);


--
-- Name: idx_mcp_tool_executions_server_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_server_status ON public.mcp_tool_executions USING btree (server_name, status);


--
-- Name: idx_mcp_tool_executions_server_tool; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_server_tool ON public.mcp_tool_executions USING btree (server_name, tool_name);


--
-- Name: idx_mcp_tool_executions_session_started; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_session_started ON public.mcp_tool_executions USING btree (session_id, started_at DESC) WHERE (session_id IS NOT NULL);


--
-- Name: idx_mcp_tool_executions_session_tool; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_session_tool ON public.mcp_tool_executions USING btree (session_id, tool_name);


--
-- Name: idx_mcp_tool_executions_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mcp_tool_executions_source ON public.mcp_tool_executions USING btree (source, started_at DESC);


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
-- Name: idx_message_parts_file_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_file_id ON public.message_parts USING btree (file_id) WHERE (file_id IS NOT NULL);


--
-- Name: idx_message_parts_kind; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_parts_kind ON public.message_parts USING btree (part_kind);


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
-- Name: idx_oauth_client_grant_types_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_oauth_client_grant_types_type ON public.oauth_client_grant_types USING btree (grant_type);


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
-- Name: idx_organization_members_org; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organization_members_org ON public.organization_members USING btree (org_id);


--
-- Name: idx_organization_members_org_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organization_members_org_role ON public.organization_members USING btree (org_id, org_role);


--
-- Name: idx_organizations_email_domains; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organizations_email_domains ON public.organizations USING gin (email_domains);


--
-- Name: idx_organizations_plan; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organizations_plan ON public.organizations USING btree (plan_id);


--
-- Name: idx_organizations_platform; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_organizations_platform ON public.organizations USING btree (is_platform) WHERE is_platform;


--
-- Name: idx_organizations_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_organizations_status ON public.organizations USING btree (status);


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
-- Name: idx_project_members_valid_until; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_project_members_valid_until ON public.project_members USING btree (valid_until) WHERE ((valid_until IS NOT NULL) AND (revoked_at IS NULL));


--
-- Name: idx_refresh_tokens_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_tokens_lookup ON public.oauth_refresh_tokens USING btree (token_id, expires_at);


--
-- Name: idx_retention_runs_table_run; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_retention_runs_table_run ON public.retention_runs USING btree (table_name, run_at DESC);


--
-- Name: idx_scheduled_jobs_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scheduled_jobs_enabled ON public.scheduled_jobs USING btree (enabled);


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
-- Name: idx_service_owned_ids_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_service_owned_ids_source ON public.service_owned_ids USING btree (source);


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
-- Name: idx_task_messages_session_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_task_messages_session_id ON public.task_messages USING btree (session_id);


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
-- Name: idx_user_contexts_user_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_contexts_user_updated ON public.user_contexts USING btree (user_id, updated_at DESC);


--
-- Name: idx_user_device_cert_validity_until; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_device_cert_validity_until ON public.user_device_cert_validity USING btree (valid_until);


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
-- Name: idx_user_manual_roles_valid_until; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_manual_roles_valid_until ON public.user_manual_roles USING btree (valid_until) WHERE (valid_until IS NOT NULL);


--
-- Name: idx_user_profile_ext_department; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_profile_ext_department ON public.user_profile_ext USING btree (department);


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
-- Name: idx_user_sessions_is_scanner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_is_scanner ON public.user_sessions USING btree (is_scanner);


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
-- Name: idx_webauthn_setup_tokens_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_webauthn_setup_tokens_user_id ON public.webauthn_setup_tokens USING btree (user_id);


--
-- Name: ingestion_outbox_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX ingestion_outbox_pending ON public.ingestion_outbox USING btree (created_at) WHERE (processed_at IS NULL);


--
-- Name: managed_consumer_receipt_identity; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX managed_consumer_receipt_identity ON public.managed_installation_receipts USING btree (consumer_id, device_id, host, installation_id, publication_id) WHERE (consumer_id IS NOT NULL);


--
-- Name: managed_inventory_observations_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX managed_inventory_observations_time ON public.managed_inventory_observations USING btree (owner_id, observed_at);


--
-- Name: managed_inventory_open_membership; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX managed_inventory_open_membership ON public.managed_inventory_membership USING btree (owner_id, entry_id) WHERE (effective_until IS NULL);


--
-- Name: managed_revisions_owner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX managed_revisions_owner_id ON public.managed_revisions USING btree (owner_id, id);


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
-- Name: managed_consumer_session_bindings managed_consumer_bindings_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_consumer_bindings_immutable BEFORE UPDATE ON public.managed_consumer_session_bindings FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_revision_dependencies managed_dependencies_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_dependencies_immutable BEFORE UPDATE ON public.managed_revision_dependencies FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_installation_receipts managed_installation_receipts_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_installation_receipts_immutable BEFORE UPDATE ON public.managed_installation_receipts FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


--
-- Name: managed_inventory_bindings managed_inventory_bindings_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER managed_inventory_bindings_immutable BEFORE UPDATE ON public.managed_inventory_bindings FOR EACH ROW EXECUTE FUNCTION public.reject_managed_content_update();


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
-- Name: ai_request_messages message_count_delete; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER message_count_delete AFTER DELETE ON public.ai_request_messages REFERENCING OLD TABLE AS old_rows FOR EACH STATEMENT EXECUTE FUNCTION public.sp_ai_request_message_count();


--
-- Name: ai_request_messages message_count_insert; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER message_count_insert AFTER INSERT ON public.ai_request_messages REFERENCING NEW TABLE AS new_rows FOR EACH STATEMENT EXECUTE FUNCTION public.sp_ai_request_message_count();


--
-- Name: ai_requests request_scope_stamp_ai_requests_stmt; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER request_scope_stamp_ai_requests_stmt AFTER INSERT ON public.ai_requests REFERENCING NEW TABLE AS new_rows FOR EACH STATEMENT EXECUTE FUNCTION public.request_scope_stamp_ai_requests();


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
-- Name: access_control_rule_validity access_control_rule_validity_rule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_control_rule_validity
    ADD CONSTRAINT access_control_rule_validity_rule_id_fkey FOREIGN KEY (rule_id) REFERENCES public.access_control_rules(id) ON DELETE CASCADE;


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
-- Name: ai_request_client_evidence ai_request_client_evidence_ai_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_client_evidence
    ADD CONSTRAINT ai_request_client_evidence_ai_request_id_fkey FOREIGN KEY (ai_request_id) REFERENCES public.ai_requests(id) ON DELETE CASCADE;


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
-- Name: ai_request_payloads ai_request_payloads_offered_tools_sha256_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_payloads
    ADD CONSTRAINT ai_request_payloads_offered_tools_sha256_fkey FOREIGN KEY (offered_tools_sha256) REFERENCES public.ai_tool_catalogs(sha256);


--
-- Name: ai_request_payloads ai_request_payloads_prepared_tools_sha256_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_payloads
    ADD CONSTRAINT ai_request_payloads_prepared_tools_sha256_fkey FOREIGN KEY (prepared_tools_sha256) REFERENCES public.ai_tool_catalogs(sha256);


--
-- Name: ai_request_scopes ai_request_scopes_group_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_scopes
    ADD CONSTRAINT ai_request_scopes_group_id_fkey FOREIGN KEY (group_id) REFERENCES public.groups(id) ON DELETE SET NULL;


--
-- Name: ai_request_scopes ai_request_scopes_project_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_scopes
    ADD CONSTRAINT ai_request_scopes_project_id_fkey FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE SET NULL;


--
-- Name: ai_request_scopes ai_request_scopes_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_request_scopes
    ADD CONSTRAINT ai_request_scopes_request_id_fkey FOREIGN KEY (request_id) REFERENCES public.ai_requests(id) ON DELETE CASCADE;


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
-- Name: conversation_skill_facts conversation_skill_facts_context_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.conversation_skill_facts
    ADD CONSTRAINT conversation_skill_facts_context_id_fkey FOREIGN KEY (context_id) REFERENCES public.conversation_facts(context_id) ON DELETE CASCADE;


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
-- Name: managed_consumer_credentials managed_consumer_credentials_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_credentials
    ADD CONSTRAINT managed_consumer_credentials_device_id_fkey FOREIGN KEY (device_id) REFERENCES public.user_device_certs(id) ON DELETE CASCADE;


--
-- Name: managed_consumer_grants managed_consumer_grants_consumer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_grants
    ADD CONSTRAINT managed_consumer_grants_consumer_id_fkey FOREIGN KEY (consumer_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: managed_consumer_grants managed_consumer_grants_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_grants
    ADD CONSTRAINT managed_consumer_grants_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_consumer_session_bindings managed_consumer_session_bindings_consumer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_session_bindings
    ADD CONSTRAINT managed_consumer_session_bindings_consumer_id_fkey FOREIGN KEY (consumer_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: managed_consumer_session_bindings managed_consumer_session_bindings_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_session_bindings
    ADD CONSTRAINT managed_consumer_session_bindings_device_id_fkey FOREIGN KEY (device_id) REFERENCES public.user_device_certs(id) ON DELETE CASCADE;


--
-- Name: managed_consumer_session_bindings managed_consumer_session_bindings_receipt_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_consumer_session_bindings
    ADD CONSTRAINT managed_consumer_session_bindings_receipt_id_fkey FOREIGN KEY (receipt_id) REFERENCES public.managed_installation_receipts(id) ON DELETE CASCADE;


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
-- Name: managed_installation_coverage managed_installation_coverage_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_coverage
    ADD CONSTRAINT managed_installation_coverage_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_installation_coverage_state managed_installation_coverage_state_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_coverage_state
    ADD CONSTRAINT managed_installation_coverage_state_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: managed_installation_receipts managed_installation_receipts_consumer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_consumer_id_fkey FOREIGN KEY (consumer_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: managed_installation_receipts managed_installation_receipts_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_device_id_fkey FOREIGN KEY (device_id) REFERENCES public.user_device_certs(id) ON DELETE CASCADE;


--
-- Name: managed_installation_receipts managed_installation_receipts_owner_id_resource_id_publication_; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_installation_receipts
    ADD CONSTRAINT managed_installation_receipts_owner_id_resource_id_publication_ FOREIGN KEY (owner_id, resource_id, publication_id) REFERENCES public.managed_publications(owner_id, resource_id, id);


--
-- Name: managed_inventory_authoring_heads managed_inventory_authoring_heads_owner_id_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_authoring_heads
    ADD CONSTRAINT managed_inventory_authoring_heads_owner_id_entry_id_fkey FOREIGN KEY (owner_id, entry_id) REFERENCES public.managed_inventory_entries(owner_id, entry_id);


--
-- Name: managed_inventory_authoring_heads managed_inventory_authoring_heads_owner_id_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_authoring_heads
    ADD CONSTRAINT managed_inventory_authoring_heads_owner_id_revision_id_fkey FOREIGN KEY (owner_id, revision_id) REFERENCES public.managed_revisions(owner_id, id);


--
-- Name: managed_inventory_bindings managed_inventory_bindings_bound_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_bindings
    ADD CONSTRAINT managed_inventory_bindings_bound_by_fkey FOREIGN KEY (bound_by) REFERENCES public.users(id);


--
-- Name: managed_inventory_bindings managed_inventory_bindings_owner_id_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_bindings
    ADD CONSTRAINT managed_inventory_bindings_owner_id_entry_id_fkey FOREIGN KEY (owner_id, entry_id) REFERENCES public.managed_inventory_entries(owner_id, entry_id);


--
-- Name: managed_inventory_bindings managed_inventory_bindings_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_bindings
    ADD CONSTRAINT managed_inventory_bindings_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_inventory_entries managed_inventory_entries_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_entries
    ADD CONSTRAINT managed_inventory_entries_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: managed_inventory_entries managed_inventory_entries_owner_id_resource_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_entries
    ADD CONSTRAINT managed_inventory_entries_owner_id_resource_id_fkey FOREIGN KEY (owner_id, resource_id) REFERENCES public.managed_resources(owner_id, id);


--
-- Name: managed_inventory_entries managed_inventory_entries_owner_id_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_entries
    ADD CONSTRAINT managed_inventory_entries_owner_id_source_id_fkey FOREIGN KEY (owner_id, source_id) REFERENCES public.managed_sources(owner_id, id);


--
-- Name: managed_inventory_membership managed_inventory_membership_owner_id_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_membership
    ADD CONSTRAINT managed_inventory_membership_owner_id_entry_id_fkey FOREIGN KEY (owner_id, entry_id) REFERENCES public.managed_inventory_entries(owner_id, entry_id);


--
-- Name: managed_inventory_observations managed_inventory_observations_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_observations
    ADD CONSTRAINT managed_inventory_observations_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: managed_inventory_state managed_inventory_state_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_inventory_state
    ADD CONSTRAINT managed_inventory_state_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


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
-- Name: managed_publication_reviews managed_publication_reviews_owner_id_resource_id_revision_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_owner_id_resource_id_revision_id_fk FOREIGN KEY (owner_id, resource_id, revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_publication_reviews managed_publication_reviews_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_reviews
    ADD CONSTRAINT managed_publication_reviews_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: managed_publication_selections managed_publication_selections_owner_id_resource_id_publication; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_selections
    ADD CONSTRAINT managed_publication_selections_owner_id_resource_id_publication FOREIGN KEY (owner_id, resource_id, publication_id) REFERENCES public.managed_publications(owner_id, resource_id, id);


--
-- Name: managed_publication_selections managed_publication_selections_owner_id_resource_id_revision_id; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_publication_selections
    ADD CONSTRAINT managed_publication_selections_owner_id_resource_id_revision_id FOREIGN KEY (owner_id, resource_id, revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


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
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_incoming_revision_; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_incoming_revision_ FOREIGN KEY (owner_id, resource_id, incoming_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_managed_candidate_; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_managed_candidate_ FOREIGN KEY (owner_id, resource_id, managed_candidate_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_resolved_revision_; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_resolved_revision_ FOREIGN KEY (owner_id, resource_id, resolved_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


--
-- Name: managed_reconciliations managed_reconciliations_owner_id_resource_id_upstream_base_revi; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.managed_reconciliations
    ADD CONSTRAINT managed_reconciliations_owner_id_resource_id_upstream_base_revi FOREIGN KEY (owner_id, resource_id, upstream_base_revision_id) REFERENCES public.managed_revisions(owner_id, resource_id, id);


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
-- Name: mcp_artifact_findings mcp_artifact_findings_artifact_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_artifact_findings
    ADD CONSTRAINT mcp_artifact_findings_artifact_id_fkey FOREIGN KEY (artifact_id) REFERENCES public.mcp_artifacts(artifact_id) ON DELETE CASCADE;


--
-- Name: mcp_artifacts mcp_artifacts_mcp_execution_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_artifacts
    ADD CONSTRAINT mcp_artifacts_mcp_execution_id_fkey FOREIGN KEY (mcp_execution_id) REFERENCES public.mcp_tool_executions(mcp_execution_id) ON DELETE CASCADE;


--
-- Name: mcp_artifacts mcp_artifacts_payload_sha256_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.mcp_artifacts
    ADD CONSTRAINT mcp_artifacts_payload_sha256_fkey FOREIGN KEY (payload_sha256) REFERENCES public.artifact_payloads(sha256) ON DELETE SET NULL;


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
    ADD CONSTRAINT mcp_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


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
-- Name: organization_members organization_members_org_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.organization_members
    ADD CONSTRAINT organization_members_org_id_fkey FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE CASCADE;


--
-- Name: organization_members organization_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.organization_members
    ADD CONSTRAINT organization_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: organizations organizations_plan_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.organizations
    ADD CONSTRAINT organizations_plan_id_fkey FOREIGN KEY (plan_id) REFERENCES public.plans(id) ON DELETE SET NULL;


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
-- Name: reviewed_production_failures reviewed_failure_case_owner; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_failure_case_owner FOREIGN KEY (owner_id, development_case_revision_id) REFERENCES public.managed_revisions(owner_id, id);


--
-- Name: reviewed_production_failures reviewed_failure_invocation_owner; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_failure_invocation_owner FOREIGN KEY (owner_id, invocation_id) REFERENCES public.plugin_usage_events(user_id, id);


--
-- Name: reviewed_production_failures reviewed_production_failures_development_case_revision_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reviewed_production_failures
    ADD CONSTRAINT reviewed_production_failures_development_case_revision_id_fkey FOREIGN KEY (development_case_revision_id) REFERENCES public.managed_revisions(id);


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
-- Name: service_owned_ids service_owned_ids_source_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.service_owned_ids
    ADD CONSTRAINT service_owned_ids_source_fkey FOREIGN KEY (source) REFERENCES public.service_sources(name) ON DELETE CASCADE;


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
-- Name: user_device_cert_validity user_device_cert_validity_device_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_device_cert_validity
    ADD CONSTRAINT user_device_cert_validity_device_id_fkey FOREIGN KEY (device_id) REFERENCES public.user_device_certs(id) ON DELETE CASCADE;


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
    ADD CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


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


-- Dumped from database version 18.3
-- Dumped by pg_dump version 18.3

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: extension_migrations; Type: TABLE DATA; Schema: public; Owner: -
--

INSERT INTO public.extension_migrations VALUES ('events_001', 'events', 1, 'actor_attribution', '5e14861e8d6b9497', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_002', 'events', 2, 'actor_attribution_lock', 'ae405cd923565550', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_003', 'events', 3, 'outbox_origin_instance', '268b8ff895ad72f9', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_004', 'events', 4, 'durable_consumption', '820ef124b2170e06', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_005', 'events', 5, 'reporting_privacy', '3a3bdbce6d55a827', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_006', 'events', 6, 'user_privacy_delivery', '69e5571b4674e381', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_007', 'events', 7, 'drop_duplicate_actor_id_check', 'be593158d2a67308', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_008', 'events', 8, 'restore_actor_id_nonempty', '62c7dc72691e642c', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('events_009', 'events', 9, 'retire_reporting_capture', 'e07e831649884c2f', '2026-09-28 14:35:43.979678+00');
INSERT INTO public.extension_migrations VALUES ('users_001', 'users', 1, 'add_user_sessions_utm_content_term', '4c584594fd20d672', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_002', 'users', 2, 'add_user_sessions_is_ai_crawler', '2fd7329ef96d7544', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_003', 'users', 3, 'rebuild_clean_traffic_index', '31b675f52bf6642c', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_004', 'users', 4, 'user_sessions_revoked_at', 'd8450941ffa1159c', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_005', 'users', 5, 'federated_identities', '68ff85fac4197d59', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_006', 'users', 6, 'user_sessions_source_bridge_mcp', '61c647f86c67b2c7', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_007', 'users', 7, 'drop_session_throttle', '12033d6a8facbe08', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_008', 'users', 8, 'canonical_traffic_views', 'cc8abe54b9257f80', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_009', 'users', 9, 'normalise_user_emails', 'c15d7e14f647c941', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_010', 'users', 10, 'drop_users_name_unique', '619cec5c7f0b6648', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_011', 'users', 11, 'user_rate_limit_buckets', '504384d07e8cf48d', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_012', 'users', 12, 'device_eligibility_interface', '52272c6000339053', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_013', 'users', 13, 'user_retention_barrier', '76e6d55c2749d617', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_014', 'users', 14, 'reporting_privacy', '8adff81817225598', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_015', 'users', 15, 'user_privacy_delivery', '24488db167f9f4b5', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_016', 'users', 16, 'user_sessions_cascade', '001f70a7b635854c', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_018', 'users', 18, 'prune_prefix_duplicate_indexes', '1360951d05a02204', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_019', 'users', 19, 'backfill_ghost_session_flags', '6f121ade48f26ae1', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('users_020', 'users', 20, 'retire_reporting_privacy', '1de595d4ff0a3732', '2026-09-28 14:35:44.008249+00');
INSERT INTO public.extension_migrations VALUES ('mcp_001', 'mcp', 1, 'session_initialize_params', 'c95c06aab309a732', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_002', 'mcp', 2, 'artifact_server_name_repair', '6085aba80a5e0cf7', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_003', 'mcp', 3, 'tool_execution_actor', 'd99c19dfe5556653', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_004', 'mcp', 4, 'mcp_proxy_identities', 'fe504c84ba6e4289', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_005', 'mcp', 5, 'mcp_external_sessions', 'cac12aa04e6e168e', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_006', 'mcp', 6, 'reporting_privacy', 'a062992389a880c5', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_007', 'mcp', 7, 'mcp_proxy_identity_roles', 'ad374034ba30fa76', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_008', 'mcp', 8, 'artifact_narrow_waist', '42c3cdc0a887ec22', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_009', 'mcp', 9, 'mcp_sessions_cascade', '5bf8e6be9a75feb1', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_011', 'mcp', 11, 'artifact_constraints', '182833e1ae999fb3', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_012', 'mcp', 12, 'artifact_source_repair', '5bacfd58a552e856', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_013', 'mcp', 13, 'reporting_fact_repair', '8623a2d4d76d32be', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_014', 'mcp', 14, 'pair_hook_attestations', '4039b1ffe08f6d9b', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_015', 'mcp', 15, 'prune_prefix_duplicate_indexes', '5c42d99635e69a6b', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_016', 'mcp', 16, 'seal_proxy_identity_tokens', '4980043241766621', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('mcp_017', 'mcp', 17, 'retire_reporting_capture', '09123780bf3f3d51', '2026-09-28 14:35:44.125731+00');
INSERT INTO public.extension_migrations VALUES ('ai_001', 'ai', 1, 'gateway_governance', '5e9fa59d853246fc', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_002', 'ai', 2, 'split_context_id', 'be2b1b232789908c', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_003', 'ai', 3, 'drop_runtime_tenancy', '78d22d6fbf237b08', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_004', 'ai', 4, 'actor_attribution', '7ed4218b3778b19b', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_005', 'ai', 5, 'actor_attribution_lock', '8539122f633d828a', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_006', 'ai', 6, 'requested_model', '42c8ccec74f973b1', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_007', 'ai', 7, 'system_prompt_override', '648a0ae118302719', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_008', 'ai', 8, 'route_match', '2f959e35df8af04f', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_009', 'ai', 9, 'ai_requests_session_fk', '1cdbb014e4ef592d', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_010', 'ai', 10, 'nullable_rejection_routing', 'f6c07dd155d0d20e', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_011', 'ai', 11, 'subject_quota_buckets', '9607680136af51f6', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_012', 'ai', 12, 'payload_digests', '7a30f9d904f464ea', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_013', 'ai', 13, 'offered_tools', 'a2163d4d538ab632', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_014', 'ai', 14, 'ai_requests_context_not_null', '7eb1fb85f247f1c9', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_015', 'ai', 15, 'ai_requests_synthetic', 'd48d51cf1a2bbabc', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_016', 'ai', 16, 'gateway_policy_priority', 'f60d4b2cddfe7916', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_017', 'ai', 17, 'gateway_thought_signatures', '1889d2fc1ff08604', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_018', 'ai', 18, 'ai_requests_instance_id', '589699534dfad8b8', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_019', 'ai', 19, 'ai_safety_findings_blocked', 'a2f3985a0215e8ff', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_020', 'ai', 20, 'ai_requests_reasoning_tokens', '5d81e67e024dcc33', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_021', 'ai', 21, 'ai_requests_client_session_kind', '072a3fc6d0e1be38', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_022', 'ai', 22, 'ai_requests_upstream_latency', '895995d35b0b1851', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_023', 'ai', 23, 'thought_signature_owner', '01624b951824e1ab', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_024', 'ai', 24, 'ai_request_accounting_failure', 'efc5bfd58d565960', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_025', 'ai', 25, 'reporting_privacy', '3d0d3a8577b0c9f4', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_026', 'ai', 26, 'ai_requests_client_origin', '75c76a8cf077ff7b', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_027', 'ai', 27, 'ai_request_client_attestation', '2598ddd2895b2723', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_028', 'ai', 28, 'ai_requests_finish_reason', '2aba8e8504e8ade7', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_029', 'ai', 29, 'ai_requests_served_provider', '8951702c85e02f20', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_030', 'ai', 30, 'prepared_tools', '482cc4d7fa4c07bc', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_031', 'ai', 31, 'tool_call_ledger_builtin', 'aeb69fcd231e97b7', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_032', 'ai', 32, 'ai_requests_message_count', '4569ec7acab1d46a', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_033', 'ai', 33, 'tool_catalog_dedup', '2c8afa4349bc7df6', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_034', 'ai', 34, 'claude_metadata_json_marker', 'e1bc3120940ee0ec', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_035', 'ai', 35, 'backfill_session_ai_counters', 'cf30817dacd18908', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_036', 'ai', 36, 'prune_prefix_duplicate_indexes', '16f8daa0c7516ccd', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('ai_037', 'ai', 37, 'retire_reporting_capture', '0f8bdbeb2bf8ab8c', '2026-09-28 14:35:44.220819+00');
INSERT INTO public.extension_migrations VALUES ('oauth_001', 'oauth', 1, 'add_rfc8707_resource_column', '494da2b59158d9f7', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_002', 'oauth', 2, 'rename_cowork_to_bridge', '251290add3ada5fc', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_003', 'oauth', 3, 'drop_bridge_session_tenant', '43ca55d5add0b178', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_004', 'oauth', 4, 'oauth_client_owner', 'be4ae04b220a213e', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_005', 'oauth', 5, 'auth_code_family', '9710a939665978b4', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_006', 'oauth', 6, 'at_rest_pepper_hash', 'bf9839b89d43c11b', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_007', 'oauth', 7, 'oauth_state_bindings', 'ea71cf93a0490c6d', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_008', 'oauth', 8, 'oauth_jti_revocations', '6a8865943936cb66', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_009', 'oauth', 9, 'refresh_token_consumed_at', 'b0af15521a52b286', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_010', 'oauth', 10, 'backfill_oauth_client_owner_fk', '5577d06bb8692d22', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_011', 'oauth', 11, 'add_application_type', '2efc21180c0aac45', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_012', 'oauth', 12, 'bridge_host_model_prefs', '42199296a92ae69b', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_013', 'oauth', 13, 'id_jag_replay', '73e439a5dd598d0f', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_014', 'oauth', 14, 'webauthn_challenges_state_store', '4ed05b736e1c9236', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_015', 'oauth', 15, 'webauthn_challenges_user_fk', 'bacb4641e0b50924', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_016', 'oauth', 16, 'oauth_client_registration_token', '9fe7272a6aeba3a3', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('oauth_017', 'oauth', 17, 'prune_prefix_duplicate_indexes', '6c1a2867d3c328c2', '2026-09-28 14:35:44.377244+00');
INSERT INTO public.extension_migrations VALUES ('analytics_001', 'analytics', 1, 'add_engagement_event_type', 'd322ff86aa134d2a', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_002', 'analytics', 2, 'add_engagement_event_data', '2156106c6c94d2ff', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_003', 'analytics', 3, 'seed_anomaly_thresholds', '46d8e70b7ce49ea9', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_004', 'analytics', 4, 'drop_high_risk_fingerprints_view', '78a931cf97094d4e', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_005', 'analytics', 5, 'feedback_facts', '799cd1cfa68fb59c', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_006', 'analytics', 6, 'ingestion_producers', 'd154a6fd89acada7', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_007', 'analytics', 7, 'feedback_snapshots', 'b83c88688892dfdc', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_014', 'analytics', 14, 'prune_prefix_duplicate_indexes', '9eae745584427a19', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_015', 'analytics', 15, 'retire_feedback_and_reporting_projection', 'bc3b903339e227ff', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_016', 'analytics', 16, 'drop_retired_report_tables', '2d0ab8fdf772fd5a', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('analytics_017', 'analytics', 17, 'report_views_replace_in_place', 'ab54fd41e4ad9fd2', '2026-09-28 14:35:44.656057+00');
INSERT INTO public.extension_migrations VALUES ('content_001', 'content', 1, 'markdown_content_locale_unique', '1d5d5978cbf29a46', '2026-09-28 14:35:44.769199+00');
INSERT INTO public.extension_migrations VALUES ('content_002', 'content', 2, 'drop_link_analytics_views', '1fc92a6fbbe2b211', '2026-09-28 14:35:44.769199+00');
INSERT INTO public.extension_migrations VALUES ('content_003', 'content', 3, 'reporting_privacy', '1cf422e471e2d37d', '2026-09-28 14:35:44.769199+00');
INSERT INTO public.extension_migrations VALUES ('content_004', 'content', 4, 'prune_prefix_duplicate_indexes', '7008af52ba09665d', '2026-09-28 14:35:44.769199+00');
INSERT INTO public.extension_migrations VALUES ('content_005', 'content', 5, 'retire_reporting_capture', 'd1922b0cb75688a8', '2026-09-28 14:35:44.769199+00');
INSERT INTO public.extension_migrations VALUES ('logging_001', 'logging', 1, 'split_context_id', '13c45db59ea66544', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_002', 'logging', 2, 'analytics_event_data', 'a36c0cad23329eea', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_003', 'logging', 3, 'prune_redundant_log_indexes', '80ad435eacd2fd1e', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_004', 'logging', 4, 'drop_client_log_views', '76ae397f539d7fc6', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_005', 'logging', 5, 'logs_instance_id', '830d265f0c451b1c', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_006', 'logging', 6, 'reporting_privacy', 'd413012b2405adcf', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_007', 'logging', 7, 'drop_logs_projection', '884d7f65aaf5eb7e', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_008', 'logging', 8, 'drop_duplicate_level_check', '3b853ef799e173b6', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('logging_009', 'logging', 9, 'retire_reporting_capture', 'fa648c59b2a107f0', '2026-09-28 14:35:44.853807+00');
INSERT INTO public.extension_migrations VALUES ('scheduler_001', 'scheduler', 1, 'scheduled_jobs_last_instance', '049f54408f4f5ccc', '2026-09-28 14:35:45.123917+00');
INSERT INTO public.extension_migrations VALUES ('scheduler_002', 'scheduler', 2, 'scheduled_jobs_last_message', '3cb7c5c4216e85aa', '2026-09-28 14:35:45.123917+00');
INSERT INTO public.extension_migrations VALUES ('scheduler_003', 'scheduler', 3, 'prune_prefix_duplicate_indexes', 'aaeeba25e0358f5a', '2026-09-28 14:35:45.123917+00');
INSERT INTO public.extension_migrations VALUES ('agent_001', 'agent', 1, 'drop_playbooks', '2fe3674396079f31', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_002', 'agent', 2, 'add_server_type', 'a32dc3019b6818b0', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_003', 'agent', 3, 'a2a_v1_task_states', '2f3387804ae0319b', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_004', 'agent', 4, 'ai_requests_task_fk', '923ec035712b93e3', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_005', 'agent', 5, 'add_task_version', '60590892c4ccd2e6', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_006', 'agent', 6, 'drop_agent_skills', '19b3ac09309e7d35', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_007', 'agent', 7, 'drop_agents', '6f0f35bb3892ebf9', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_008', 'agent', 8, 'add_user_contexts_kind', '9b62a2fc269c2fba', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_009', 'agent', 9, 'drop_session_analytics_views', '754d9b78e2758951', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_010', 'agent', 10, 'services_instance_scope', 'f0f9e3195f30dd6f', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_011', 'agent', 11, 'drop_task_push_notification_configs', 'aaeea339959a412f', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_012', 'agent', 12, 'reporting_privacy', '0b46a511dd2d182f', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_013', 'agent', 13, 'prune_prefix_duplicate_indexes', '8be1987cdbe240ce', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('agent_014', 'agent', 14, 'retire_reporting_capture', '5e07dec1ca8ba59f', '2026-09-28 14:35:44.552603+00');
INSERT INTO public.extension_migrations VALUES ('authz_001', 'authz', 1, 'access_control_rules_evolution', '46f992c79ec0745c', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_002', 'authz', 2, 'actor_attribution', '4bb62e10ee4e261e', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_003', 'authz', 3, 'actor_attribution_lock', 'e94806d1d72ddf08', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_004', 'authz', 4, 'act_chain', 'db5ec60fe7bd133d', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_005', 'authz', 5, 'actor_kind_extend', 'e19ceb4e8e0bdbd6', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_007', 'authz', 7, 'split_acl_entities', '95f9291c73f52e90', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_008', 'authz', 8, 'drop_department_acl', 'e87c60bd3f0e2e6f', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_009', 'authz', 9, 'messaging_acl_entity_types', 'f939735dcd876537', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_010', 'authz', 10, 'governance_context_task', '58f8cb475372e638', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_011', 'authz', 11, 'open_rule_type_vocabulary', '3ad469ab6af692ca', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_012', 'authz', 12, 'governance_decisions_context_not_null', 'd206fdaf609a1eda', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_013', 'authz', 13, 'governance_decisions_trace_id', 'b7823ebf44dfe8b7', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_014', 'authz', 14, 'tool_approval_requests', '985c9656d1111428', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_015', 'authz', 15, 'governance_decisions_client_id', '6fe3fa7445d5354b', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_016', 'authz', 16, 'governance_decisions_warn', '34656346b10dff69', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_017', 'authz', 17, 'access_control_rules_source', 'd9185a002adf5048', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_018', 'authz', 18, 'governance_decisions_append_only', 'a1d728415f309370', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_019', 'authz', 19, 'governance_decisions_tool_use_id', '1808427c71778556', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_020', 'authz', 20, 'backfill_governance_decision_context', '04645cdf47a04041', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('authz_021', 'authz', 21, 'prune_prefix_duplicate_indexes', '2260f19f49a85619', '2026-09-28 14:35:44.696493+00');
INSERT INTO public.extension_migrations VALUES ('files_001', 'files', 1, 'drop_ai_image_stats_view', '2559297865e35d23', '2026-09-28 14:35:44.827927+00');
INSERT INTO public.extension_migrations VALUES ('files_002', 'files', 2, 'prune_prefix_duplicate_indexes', 'fb1f92bd1d147073', '2026-09-28 14:35:44.827927+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_001', 'managed_resources', 1, 'revision_listing_indexes', 'd00dbf6413b9ef80', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_002', 'managed_resources', 2, 'managed_resolution', 'a3c3c4b330d81b0f', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_003', 'managed_resources', 3, 'evaluation_attestations', 'dd9fe5d9bfd846af', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_004', 'managed_resources', 4, 'consumer_evidence', '8be01a0228e9f90b', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_005', 'managed_resources', 5, 'dependency_verification', '7a3a55d7ee0b176c', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_006', 'managed_resources', 6, 'inventory', '4c8deef71e356bf3', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_007', 'managed_resources', 7, 'installation_coverage', '6b4c3e847d4f04d7', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_008', 'managed_resources', 8, 'api_operations', '1ee81b04b36d62af', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_009', 'managed_resources', 9, 'publication_review_experiment', 'b93a1ad63dc56c94', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_010', 'managed_resources', 10, 'inventory_sources', '605ff52b7fcc0abe', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_011', 'managed_resources', 11, 'drop_evaluation', '15bd5db5940e2130', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_013', 'managed_resources', 13, 'local_tree_roots_follow_current', '002636c337db8c5f', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_014', 'managed_resources', 14, 'receipts_accept_unavailable_mode_on_plain_files', '95c59f04c026feab', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_015', 'managed_resources', 15, 'prune_prefix_duplicate_indexes', '088038aafc42e356', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('managed_resources_016', 'managed_resources', 16, 'drop_dead_verification_and_capture_tables', '2385071133538324', '2026-09-28 14:35:44.887808+00');
INSERT INTO public.extension_migrations VALUES ('web_059', 'web', 59, 'dashboard_groups_projects', '35d64d79e1eeba2a', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_060', 'web', 60, 'dashboard_scope_defaults', '791b04447ef866a9', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_061', 'web', 61, 'dashboard_connector_credentials', 'bd0f943aead797fe', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_062', 'web', 62, 'dashboard_connector_accounts', '1d1021bdb362515d', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_063', 'web', 63, 'dashboard_conversation_requests', '0f7b91736f2ceb54', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_064', 'web', 64, 'dashboard_skill_invocation_events', 'c519a389bcdcab80', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_065', 'web', 65, 'dashboard_usage_loc_columns', 'b2cf29e47e74f2f9', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_066', 'web', 66, 'dashboard_transcript_fts', '5b8a7724bccab402', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_067', 'web', 67, 'dashboard_salesforce_identity', '105e3f5b687e637b', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_068', 'web', 68, 'dashboard_indexes', '024d96a9276e52f6', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_069', 'web', 69, 'dev_login_codes', '01adfd0a1fc6593a', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_070', 'web', 70, 'ingestion_integrity', '999efd17f8e3dc8e', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_071', 'web', 71, 'recover_native_sessions', 'd5003f7536c0c83b', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_072', 'web', 72, 'skill_version_impact', '7021fa5f70d67e85', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_073', 'web', 73, 'version_impact_owner_constraints', '6c350b38d5eda549', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_074', 'web', 74, 'drop_independently_metered_tool_charges', '5b2b4a2f554fcc9d', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_075', 'web', 75, 'dashboard_usage_metrics', '1bcb45653abc5d54', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_076', 'web', 76, 'drop_legacy_mcp_artifact_index', 'acbea0534eafa9ea', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_082', 'web', 82, 'restore_web_billing_model', '742229f272ff8518', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_083', 'web', 83, 'sync_state', '7002c9c6ebd8fa05', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_084', 'web', 84, 'service_sources', '0c9c645937b72dcc', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_085', 'web', 85, 'marketplace_versions', '05ee9c947b1a72e3', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_086', 'web', 86, 'conversation_analyses', 'b55f76c0744b2cff', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_087', 'web', 87, 'request_scopes', '08165425503e8cf9', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_088', 'web', 88, 'time_bound_access', '252c18ccd1932831', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_089', 'web', 89, 'gateway_routes', '7d09941d1078cd13', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_090', 'web', 90, 'tool_artifacts', '48205ac5acd5d824', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_091', 'web', 91, 'conversation_facts', '594cde98bc55cebc', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_092', 'web', 92, 'user_last_seen', '6957d595736ab2a2', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_093', 'web', 93, 'retention_ledger', '771b34a2ed318723', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('web_094', 'web', 94, 'raw_retention', '163b6d694d2300c6', '2026-09-28 14:35:45.172471+00');
INSERT INTO public.extension_migrations VALUES ('database_001', 'database', 1, 'prune_prefix_duplicate_indexes', 'ea17f5309de0b4a9', '2026-09-28 14:35:46.736159+00');


--
-- PostgreSQL database dump complete
--


