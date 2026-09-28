CREATE TABLE IF NOT EXISTS ingestion_session_owners (
    session_id TEXT PRIMARY KEY CHECK (length(session_id) BETWEEN 1 AND 255),
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION assert_ingestion_owner(session_key text, owner_key text)
RETURNS void LANGUAGE plpgsql AS $$
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

CREATE OR REPLACE FUNCTION enforce_ingestion_owner()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'UPDATE' AND (NEW.user_id <> OLD.user_id OR NEW.session_id <> OLD.session_id) THEN
        RAISE EXCEPTION 'Ingestion ownership is immutable' USING ERRCODE = '23514';
    END IF;
    PERFORM assert_ingestion_owner(NEW.session_id, NEW.user_id);
    RETURN NEW;
END $$;

DROP TRIGGER IF EXISTS ingestion_owner ON plugin_usage_events;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON plugin_usage_events
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS ingestion_owner ON plugin_session_summaries;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON plugin_session_summaries
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS ingestion_owner ON session_cost_snapshots;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON session_cost_snapshots
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS ingestion_owner ON session_transcripts;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON session_transcripts
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();
DROP TRIGGER IF EXISTS ingestion_owner ON session_analyses;
CREATE TRIGGER ingestion_owner BEFORE INSERT OR UPDATE ON session_analyses
FOR EACH ROW EXECUTE FUNCTION enforce_ingestion_owner();

CREATE TABLE IF NOT EXISTS ingestion_repairs (
    request_id TEXT PRIMARY KEY,
    repair_kind TEXT NOT NULL,
    source_digest TEXT,
    recovered_client_session_id TEXT NOT NULL,
    repaired_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION analysis_native_session(metadata_user_id text)
RETURNS text LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE session_key text;
BEGIN
    IF left(btrim(metadata_user_id), 1) = '{' THEN
        session_key := (metadata_user_id::jsonb)->>'session_id';
    ELSE
        session_key := substring(metadata_user_id from '_session_([^[:space:]]+)$');
    END IF;
    RETURN (session_key::uuid)::text;
EXCEPTION WHEN invalid_text_representation THEN RETURN NULL;
END $$;

CREATE OR REPLACE VIEW analysis_skill_events AS
SELECT e.id, e.user_id, e.session_id, e.plugin_id,
       replace(substring(e.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)'), '_', '-') AS skill,
       NULL::text AS tool_use_id, 'slash'::text AS source, e.created_at AS invoked_at
FROM plugin_usage_events e
WHERE e.event_type = 'UserPromptSubmit'
  AND e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'
UNION ALL
SELECT e.id, e.user_id, e.session_id, e.plugin_id,
       replace(e.metadata->'tool_input'->>'skill', '_', '-') AS skill,
       e.metadata->>'tool_use_id' AS tool_use_id, 'tool'::text AS source, e.created_at AS invoked_at
FROM plugin_usage_events e
WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure') AND e.tool_name = 'Skill'
  AND e.metadata->'tool_input'->>'skill' IS NOT NULL;

CREATE TABLE IF NOT EXISTS ingestion_event_receipts (
    dedup_key TEXT PRIMARY KEY,
    payload_digest TEXT NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS ingestion_outbox (
    event_id TEXT PRIMARY KEY REFERENCES plugin_usage_events(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS ingestion_outbox_pending ON ingestion_outbox(created_at) WHERE processed_at IS NULL;

CREATE OR REPLACE FUNCTION accept_ingestion_delivery()
RETURNS trigger LANGUAGE plpgsql AS $$
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
DROP TRIGGER IF EXISTS ingestion_delivery ON plugin_usage_events;
CREATE TRIGGER ingestion_delivery BEFORE INSERT ON plugin_usage_events FOR EACH ROW EXECUTE FUNCTION accept_ingestion_delivery();

CREATE OR REPLACE FUNCTION enqueue_ingestion_event()
RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
    INSERT INTO ingestion_outbox(event_id) VALUES(NEW.id) ON CONFLICT DO NOTHING;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS ingestion_enqueue ON plugin_usage_events;
CREATE TRIGGER ingestion_enqueue AFTER INSERT ON plugin_usage_events FOR EACH ROW EXECUTE FUNCTION enqueue_ingestion_event();

CREATE OR REPLACE FUNCTION drain_ingestion_outbox(batch_size integer)
RETURNS bigint LANGUAGE plpgsql AS $$
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
