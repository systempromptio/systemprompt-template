-- Current version-impact schema for clean installations. The trigger and view are
-- also created by migration 072 for upgraded databases.

-- Core 0.61 retires this Marketplace projection only after consumer views
-- release it. Web remains that consumer, so it owns the retained projection
-- on fresh installs as well as on upgrades.
CREATE TABLE IF NOT EXISTS managed_invocation_attributions (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    invocation_id TEXT NOT NULL,
    installation_id TEXT,
    resource_id TEXT,
    revision_id TEXT,
    publication_generation BIGINT,
    traffic_class TEXT NOT NULL CHECK (traffic_class IN ('production','fixture')),
    status TEXT NOT NULL CHECK (status IN ('verified','revision_unknown','unsupported','historical')),
    receipt_id TEXT,
    authenticated_evidence JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY(owner_id,receipt_id) REFERENCES managed_installation_receipts(owner_id,id),
    UNIQUE(owner_id,invocation_id)
);
DROP TRIGGER IF EXISTS managed_invocation_attributions_immutable ON managed_invocation_attributions;
CREATE TRIGGER managed_invocation_attributions_immutable
    BEFORE UPDATE ON managed_invocation_attributions
    FOR EACH ROW EXECUTE FUNCTION reject_managed_content_update();

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
    SELECT 1 FROM attributed LIMIT 1;
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


CREATE UNIQUE INDEX IF NOT EXISTS plugin_usage_events_owner_id
    ON plugin_usage_events(user_id,id);
CREATE UNIQUE INDEX IF NOT EXISTS mcp_tool_executions_owner_id
    ON mcp_tool_executions(user_id,mcp_execution_id);
CREATE UNIQUE INDEX IF NOT EXISTS managed_revisions_owner_id
    ON managed_revisions(owner_id,id);

-- The owner-paired keys need the unique indexes declared above, which on an
-- established database only arrive through migration 057. The installer
-- applies every FOREIGN KEY after migrations and indexes (core's deferred
-- foreign-key phase), and skips a key 057 already created under the same
-- columns — so the same declaration converges fresh and upgraded databases.
-- An inline key that ran in the structural phase took the 0.52 preview down
-- at boot on 2026-09-14; this shape is what the upgrade gate proves.
CREATE TABLE IF NOT EXISTS reviewed_production_failures (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL REFERENCES users(id),
    invocation_id TEXT NOT NULL REFERENCES plugin_usage_events(id),
    reviewer_id TEXT NOT NULL REFERENCES users(id),
    sanitized_evidence JSONB NOT NULL,
    development_case_revision_id TEXT NOT NULL REFERENCES managed_revisions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner_id,invocation_id),
    CONSTRAINT reviewed_failure_invocation_owner
        FOREIGN KEY(owner_id,invocation_id) REFERENCES plugin_usage_events(user_id,id),
    CONSTRAINT reviewed_failure_case_owner
        FOREIGN KEY(owner_id,development_case_revision_id) REFERENCES managed_revisions(owner_id,id)
);

