-- Store full conversation transcripts captured from Claude Code Stop events.
-- The transcript_path on the user's machine points to a JSONL file;
-- the hook script reads it and POSTs the content here.
CREATE TABLE IF NOT EXISTS session_transcripts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    plugin_id TEXT,
    transcript JSONB NOT NULL DEFAULT '[]',
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_session_transcripts_user ON session_transcripts(user_id, captured_at DESC);
CREATE INDEX IF NOT EXISTS idx_session_transcripts_session ON session_transcripts(session_id, captured_at DESC);
