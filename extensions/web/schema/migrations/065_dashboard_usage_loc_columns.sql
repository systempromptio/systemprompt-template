-- AI-authored lines of code, measured at hook ingest from Edit/Write tool
-- inputs before sanitize_metadata truncates them. Declarative files
-- 05_plugin_usage.sql / 07_analytics.sql carry the same columns for fresh
-- installs; this migration converges established databases.
--
-- Numbered 030, not 027: the web migration chain was retired in ab902af2 and
-- files 002-027 deleted, but established databases still carry ledger rows for
-- every number they ever applied. Slot 027 was already spent on
-- demo_user_uuids, so refilling it made the recorded checksum disagree with the
-- file and every established install refused to boot. New migrations take the
-- next number above the highest ever used, never a retired one.

ALTER TABLE plugin_usage_events
    ADD COLUMN IF NOT EXISTS loc_added BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS loc_removed BIGINT NOT NULL DEFAULT 0;

ALTER TABLE plugin_usage_daily
    ADD COLUMN IF NOT EXISTS loc_added BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS loc_removed BIGINT NOT NULL DEFAULT 0;

ALTER TABLE plugin_session_summaries
    ADD COLUMN IF NOT EXISTS loc_added BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS loc_removed BIGINT NOT NULL DEFAULT 0;
