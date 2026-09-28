-- Raw evidence expiry: one function that removes the hook plane, the gateway
-- request log and what hangs off them older than a cutoff (at least 90 days),
-- keeping conversation_facts and the daily rollups. Twin of
-- schema/32_raw_retention.sql.

CREATE OR REPLACE FUNCTION expire_raw_evidence(cutoff TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE deleted BIGINT;
BEGIN
    IF cutoff > clock_timestamp()-interval '90 days' THEN
        RAISE EXCEPTION 'Raw evidence retention cutoff is too recent' USING ERRCODE='23514';
    END IF;
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
    DELETE FROM ingestion_event_receipts WHERE accepted_at<cutoff;
    DELETE FROM plugin_usage_events WHERE created_at<cutoff;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    DELETE FROM ai_requests WHERE created_at<cutoff;
    DELETE FROM conversation_analyses WHERE created_at<cutoff;
    DELETE FROM plugin_session_summaries WHERE COALESCE(ended_at,updated_at,created_at)<cutoff;
    DELETE FROM session_entity_links WHERE last_seen_at<cutoff;
    DELETE FROM ingestion_session_owners o WHERE created_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.session_id=o.session_id)
        AND NOT EXISTS(SELECT 1 FROM plugin_session_summaries s WHERE s.session_id=o.session_id);
    PERFORM public.expire_user_sessions(cutoff);
    RETURN deleted;
END $$;
