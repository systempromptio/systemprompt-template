-- Persisted usage anomalies, one row per metric per hourly window. Written by
-- the usage_anomaly job when an hour's requests, cost, or errors spike past a
-- multiple of the trailing-week hourly baseline; read by the spend dashboard.
-- Persisted (unlike core's in-memory anomaly service) so a restart cannot
-- forget an incident and the first-detection alert fires exactly once.
CREATE TABLE IF NOT EXISTS usage_anomalies (
    metric TEXT NOT NULL CHECK (metric IN ('requests', 'cost', 'errors')),
    window_start TIMESTAMPTZ NOT NULL,
    observed BIGINT NOT NULL,
    baseline BIGINT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (metric, window_start)
);

CREATE INDEX IF NOT EXISTS idx_usage_anomalies_detected
    ON usage_anomalies(detected_at DESC);
