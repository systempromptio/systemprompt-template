-- Migration: Convert published_at from TEXT to TIMESTAMP
-- This fixes the timestamp casting issue in content_trends and other analytics queries
BEGIN;

-- Add a new TIMESTAMP column
ALTER TABLE markdown_content
ADD COLUMN published_at_timestamp TIMESTAMP;

-- Migrate data from TEXT to TIMESTAMP
UPDATE markdown_content
SET published_at_timestamp = published_at::timestamp
WHERE published_at IS NOT NULL AND published_at != '';

-- Set default for new rows
ALTER TABLE markdown_content
ALTER COLUMN published_at_timestamp SET DEFAULT CURRENT_TIMESTAMP;

-- Drop the old TEXT column and indexes that depend on it
ALTER TABLE markdown_content
DROP COLUMN published_at CASCADE;

-- Rename the new column to the original name
ALTER TABLE markdown_content
RENAME COLUMN published_at_timestamp TO published_at;

-- Recreate the index
CREATE INDEX idx_markdown_content_published ON markdown_content(published_at DESC);

-- Make the column NOT NULL as it was before
ALTER TABLE markdown_content
ALTER COLUMN published_at SET NOT NULL;

COMMIT;
