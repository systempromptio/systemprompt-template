-- Raw evidence expiry: the hook plane and what hangs off it, older than the
-- cutoff. Called nightly by `plugin_usage_retention` with a 90-day cutoff,
-- in the transaction that also runs each linked member's expiry
-- (32_raw_retention_ai.sql, 32_raw_retention_marketplace.sql); the durable record of a
-- conversation (`conversation_facts`, `conversation_skill_facts`) and the
-- daily rollups are kept. Ended browser sessions past the cutoff go too,
-- through core's `expire_user_sessions`. Returns the number of hook events
-- removed.
CREATE OR REPLACE FUNCTION expire_raw_evidence(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT;
BEGIN
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Raw evidence retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    DELETE FROM ingestion_event_receipts WHERE accepted_at<cutoff;
    DELETE FROM plugin_usage_events WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    DELETE FROM conversation_analyses WHERE created_at<cutoff;
    DELETE FROM plugin_session_summaries WHERE COALESCE(ended_at,updated_at,created_at)<cutoff;
    DELETE FROM session_entity_links WHERE last_seen_at<cutoff;
    DELETE FROM ingestion_session_owners o WHERE created_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.session_id=o.session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_session_summaries s WHERE s.session_id=o.session_id);
    PERFORM public.expire_user_sessions(cutoff);
    RETURN deleted;
END $$;
