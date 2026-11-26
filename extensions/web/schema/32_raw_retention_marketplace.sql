-- Raw evidence expiry for the marketplace member's session bindings;
-- declared only when the build links it. A binding is kept while its session
-- still has hook events newer than the cutoff. `plugin_usage_retention` runs
-- it after `expire_raw_evidence` (32_raw_retention.sql) in one transaction.
CREATE OR REPLACE FUNCTION expire_marketplace_raw_evidence(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT;
BEGIN
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Raw evidence retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
    GET DIAGNOSTICS deleted = ROW_COUNT;
    RETURN deleted;
END $$;
