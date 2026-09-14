-- Verified, immutable skill-version attribution. Claims are accepted only
-- when authenticated ingestion ownership and a matching installation receipt
-- agree on owner, native session, generation, resource and revision.
CREATE OR REPLACE FUNCTION attribute_ingested_skill_invocation()
RETURNS trigger LANGUAGE plpgsql AS $$
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

DROP TRIGGER IF EXISTS attribute_skill_version ON plugin_usage_events;
CREATE TRIGGER attribute_skill_version AFTER INSERT ON plugin_usage_events
FOR EACH ROW EXECUTE FUNCTION attribute_ingested_skill_invocation();

CREATE OR REPLACE VIEW analysis_skill_version_events AS
SELECT e.id AS invocation_id,e.user_id,e.session_id,e.plugin_id,e.skill,
       e.tool_use_id,e.source,e.invoked_at,a.installation_id,a.resource_id,a.revision_id,
       a.publication_generation,COALESCE(a.traffic_class,'production') AS traffic_class,
       COALESCE(a.status,'revision_unknown') AS attribution_status
FROM analysis_skill_events e
LEFT JOIN managed_invocation_attributions a
  ON a.owner_id=e.user_id AND a.invocation_id=e.id;

CREATE TABLE IF NOT EXISTS reviewed_production_failures (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    invocation_id TEXT NOT NULL,
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    sanitized_evidence JSONB NOT NULL,
    development_case_revision_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id,invocation_id)
);

CREATE TABLE IF NOT EXISTS independently_metered_tool_charges (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    mcp_execution_id TEXT NOT NULL REFERENCES mcp_tool_executions(mcp_execution_id),
    source TEXT NOT NULL CHECK(length(source) BETWEEN 1 AND 100),
    cost_microdollars BIGINT NOT NULL CHECK(cost_microdollars>=0),
    accounting_evidence JSONB NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id,mcp_execution_id,source)
);
