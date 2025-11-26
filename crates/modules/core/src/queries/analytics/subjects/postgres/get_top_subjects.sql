SELECT
    primary_topic,
    COUNT(*) as topic_count,
    AVG(topic_confidence) as avg_confidence
FROM conversation_subjects
WHERE primary_topic IS NOT NULL
    AND analyzed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
GROUP BY primary_topic
ORDER BY topic_count DESC
LIMIT 20
