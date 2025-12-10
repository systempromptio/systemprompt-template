-- Migration: Add public column for content visibility control
-- Purpose: Allow marking content as public (visible to all) or private (admin only)

ALTER TABLE markdown_content
ADD COLUMN IF NOT EXISTS public BOOLEAN DEFAULT true;

COMMENT ON COLUMN markdown_content.public IS
'Controls content visibility. true = public (visible to all), false = private (admin only).';
