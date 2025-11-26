SELECT
    DATE(analyzed_at) as date,
    primary_topic,
    COUNT(*) as topic_count
FROM conversation_subjects
WHERE primary_topic IS NOT NULL
    AND analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
GROUP BY DATE(analyzed_at), primary_topic
ORDER BY analyzed_at DESC, topic_count DESC
