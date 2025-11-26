SELECT
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
    expires_at,
    deleted_at
FROM generated_images
WHERE uuid = $1
AND deleted_at IS NULL
