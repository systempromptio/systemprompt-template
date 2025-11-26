INSERT INTO conversation_subjects (task_id, extracted_keywords, primary_topic, topic_confidence, analyzed_at)
VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
ON CONFLICT (task_id) DO UPDATE
SET extracted_keywords = $2, primary_topic = $3, topic_confidence = $4, analyzed_at = CURRENT_TIMESTAMP
