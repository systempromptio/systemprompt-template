-- Sync state: one row per plane recording the declared hash last read and the
-- write from code last applied (mode, actor, time, active tree and
-- composition). Twin of schema/33_sync_state.sql.

CREATE TABLE IF NOT EXISTS sync_state (
    plane TEXT PRIMARY KEY,
    declared_hash TEXT NOT NULL,
    applied_hash TEXT,
    applied_mode TEXT CHECK (applied_mode IN ('seed', 'insert_only', 'overwrite')),
    applied_at TIMESTAMPTZ,
    applied_by TEXT,
    base_tree_hash TEXT,
    composed_hash TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
