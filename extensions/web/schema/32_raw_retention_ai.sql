-- Raw evidence expiry for the gateway request log, which the AI member owns;
-- declared only when the build links it. `plugin_usage_retention` runs it
-- after `expire_raw_evidence` (32_raw_retention.sql) in one transaction.
CREATE OR REPLACE FUNCTION expire_ai_raw_evidence(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT;
BEGIN
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Raw evidence retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    DELETE FROM ai_requests WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    RETURN deleted;
END $$;
