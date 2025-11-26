UPDATE user_sessions
SET
    ai_request_count = ai_request_count + 1,
    total_tokens_used = total_tokens_used + $1,
    total_ai_cost_cents = total_ai_cost_cents + $2
WHERE session_id = $3