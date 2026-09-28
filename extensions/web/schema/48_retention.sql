-- The retention ledger: what the lifecycle jobs measured, archived and found.
--
-- retention_runs is one row per managed table per run, so the console can
-- show a table's size and oldest row as a series and the daily job can say
-- "grew 25 % week on week" from its own history rather than a snapshot.
-- retention_archives lists every weekly/monthly archive written under
-- storage/exports with its digest, which is what the Data lifecycle tab
-- offers for download. retention_health_reports holds the monthly health
-- check as one JSON document per run plus the finding counts the tab sorts by.
CREATE TABLE IF NOT EXISTS retention_runs (
    id BIGSERIAL PRIMARY KEY,
    run_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tier TEXT NOT NULL CHECK (tier IN ('daily', 'weekly', 'monthly')),
    table_name TEXT NOT NULL,
    live_rows BIGINT NOT NULL,
    dead_rows BIGINT NOT NULL,
    total_bytes BIGINT NOT NULL,
    index_bytes BIGINT NOT NULL,
    oldest_row TIMESTAMPTZ,
    window_days INTEGER
);
CREATE INDEX IF NOT EXISTS idx_retention_runs_table_run ON retention_runs (table_name, run_at DESC);

CREATE TABLE IF NOT EXISTS retention_archives (
    id BIGSERIAL PRIMARY KEY,
    tier TEXT NOT NULL CHECK (tier IN ('weekly', 'monthly')),
    period TEXT NOT NULL,
    table_name TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    row_count BIGINT NOT NULL,
    byte_count BIGINT NOT NULL,
    sha256 TEXT NOT NULL,
    window_from TIMESTAMPTZ NOT NULL,
    window_to TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (tier, period, table_name)
);

CREATE TABLE IF NOT EXISTS retention_health_reports (
    id BIGSERIAL PRIMARY KEY,
    run_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    report JSONB NOT NULL,
    findings_p1 INTEGER NOT NULL DEFAULT 0,
    findings_p2 INTEGER NOT NULL DEFAULT 0,
    findings_p3 INTEGER NOT NULL DEFAULT 0
);
