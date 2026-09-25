-- Full-text search over session transcripts for the conversation-history
-- surface (REQ-045).
--
-- A generated tsvector column plus a GIN index, so `websearch_to_tsquery`
-- ranked search never scans transcript JSONB. Generated rather than
-- trigger-maintained: it cannot drift from the transcript it indexes. The
-- text fed to to_tsvector is capped at 256KB so a pathological transcript
-- stays under the 1MB tsvector limit instead of rejecting the insert.
--
-- Fresh installs get the column from schema/07_analytics.sql; this converges
-- established databases. Adding a STORED generated column rewrites the
-- table, which is acceptable at current transcript volumes.

ALTER TABLE session_transcripts
    ADD COLUMN IF NOT EXISTS search_tsv tsvector
    GENERATED ALWAYS AS (to_tsvector('english', left(transcript::text, 262144))) STORED;

CREATE INDEX IF NOT EXISTS idx_session_transcripts_fts
    ON session_transcripts USING GIN (search_tsv);
