-- Transition: move web-specific columns off the core-owned `markdown_content`
-- and `users` tables into the web-owned side tables. This is the last
-- sanctioned cross-extension ALTER — after it runs, web owns its own columns.
--
-- The column-existence guards keep this idempotent and correct on fresh
-- installs (where the core tables never carried the web columns) as well as
-- legacy installs (where an earlier release ALTERed them in).

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'markdown_content' AND column_name = 'after_reading_this'
    ) THEN
        INSERT INTO markdown_content_enrichment (
            content_id, category, after_reading_this,
            related_playbooks, related_code, related_docs, updated_at
        )
        SELECT
            id,
            category,
            COALESCE(after_reading_this, '[]'::jsonb),
            COALESCE(related_playbooks, '[]'::jsonb),
            COALESCE(related_code, '[]'::jsonb),
            COALESCE(related_docs, '[]'::jsonb),
            COALESCE(updated_at, CURRENT_TIMESTAMP)
        FROM markdown_content
        ON CONFLICT (content_id) DO NOTHING;
    END IF;
END $$;

ALTER TABLE markdown_content DROP COLUMN IF EXISTS category;
ALTER TABLE markdown_content DROP COLUMN IF EXISTS after_reading_this;
ALTER TABLE markdown_content DROP COLUMN IF EXISTS related_playbooks;
ALTER TABLE markdown_content DROP COLUMN IF EXISTS related_code;
ALTER TABLE markdown_content DROP COLUMN IF EXISTS related_docs;
ALTER TABLE markdown_content DROP COLUMN IF EXISTS image_optimization_status;

DROP INDEX IF EXISTS idx_markdown_content_after_reading_this;
DROP INDEX IF EXISTS idx_markdown_content_related_playbooks;
DROP INDEX IF EXISTS idx_markdown_content_related_code;
DROP INDEX IF EXISTS idx_markdown_content_related_docs;
DROP INDEX IF EXISTS idx_markdown_content_category_filter;

DO $$
DECLARE
    has_department BOOLEAN;
    has_share_token_version BOOLEAN;
BEGIN
    SELECT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'department'
    ) INTO has_department;

    SELECT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'users' AND column_name = 'share_token_version'
    ) INTO has_share_token_version;

    IF has_department THEN
        INSERT INTO departments (name)
        SELECT DISTINCT department
        FROM users
        WHERE department IS NOT NULL AND department <> ''
        ON CONFLICT (name) DO NOTHING;

        IF has_share_token_version THEN
            EXECUTE 'INSERT INTO user_profile_ext (user_id, department, share_token_version)
                     SELECT id,
                            COALESCE(NULLIF(department, ''''), ''Default''),
                            COALESCE(share_token_version, 1)
                     FROM users
                     ON CONFLICT (user_id) DO NOTHING';
        ELSE
            INSERT INTO user_profile_ext (user_id, department)
            SELECT
                id,
                COALESCE(NULLIF(department, ''), 'Default')
            FROM users
            ON CONFLICT (user_id) DO NOTHING;
        END IF;
    END IF;
END $$;

ALTER TABLE users DROP COLUMN IF EXISTS department;
ALTER TABLE users DROP COLUMN IF EXISTS share_token_version;

DROP INDEX IF EXISTS idx_users_department;
