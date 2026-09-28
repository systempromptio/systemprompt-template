-- What each sync plane last applied from code, so the Code <-> Instance page
-- can say "declared ab12.. · last applied ab12.. by <who> on <when> (mode)"
-- rather than only whether the two currently differ.
--
-- Declarative twin of migration 083. One row per plane. `declared_hash` is the
-- hash of the declaration as last read; `applied_*` describe the last write
-- from code (the boot seed, an insert-only, or an overwrite). `base_tree_hash`
-- and `composed_hash` pin which services tree and which bundle composition
-- were active at that write.

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
