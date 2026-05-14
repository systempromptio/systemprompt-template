-- Backfill markdown_content columns on legacy installs where the table was
-- created by core before this extension's declarative schema gained them.
-- Fresh installs already have these columns from 01_content.sql.

ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS image_optimization_status TEXT;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS category TEXT;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS after_reading_this JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS related_playbooks JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS related_code JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE markdown_content ADD COLUMN IF NOT EXISTS related_docs JSONB NOT NULL DEFAULT '[]'::jsonb;
