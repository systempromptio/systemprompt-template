INSERT INTO generated_images (
    uuid,
    request_id,
    prompt,
    model,
    provider,
    file_path,
    public_url,
    file_size_bytes,
    mime_type,
    resolution,
    aspect_ratio,
    generation_time_ms,
    cost_estimate,
    user_id,
    session_id,
    trace_id,
    created_at,
    expires_at
) VALUES (
    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
    $11, $12, $13, $14, $15, $16, $17, $18
)
