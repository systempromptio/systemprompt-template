-- Migration: Fix derived content constraint for article types
-- Purpose: Allow article_linkedin and article_medium kinds to have parent_content_id
--          The original constraint only allowed social_* kinds to have parents

-- Drop the old constraint (may be named chk_social_has_parent or already updated)
ALTER TABLE markdown_content
DROP CONSTRAINT IF EXISTS chk_social_has_parent;

ALTER TABLE markdown_content
DROP CONSTRAINT IF EXISTS chk_derived_has_parent;

-- Add updated constraint that includes article types
ALTER TABLE markdown_content
ADD CONSTRAINT chk_derived_has_parent CHECK (
    (
        (kind NOT LIKE 'social_%' AND kind != 'article_linkedin' AND kind != 'article_medium')
        AND parent_content_id IS NULL
    )
    OR
    (
        (kind LIKE 'social_%' OR kind = 'article_linkedin' OR kind = 'article_medium')
        AND parent_content_id IS NOT NULL
    )
);

COMMENT ON CONSTRAINT chk_derived_has_parent ON markdown_content IS
'Ensures derived content (social posts and platform articles) has parent_content_id set, while regular content does not.';
