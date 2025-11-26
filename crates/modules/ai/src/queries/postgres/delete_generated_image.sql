UPDATE generated_images
SET deleted_at = CURRENT_TIMESTAMP
WHERE uuid = $1
AND deleted_at IS NULL
