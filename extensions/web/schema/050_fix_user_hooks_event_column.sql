-- Migration 050: Fix user_hooks.event NOT NULL constraint
-- The Rust code uses event_type (added in 048) instead of the original event column.
-- The old event column still has NOT NULL with no default, causing inserts to fail.
-- Make it nullable with a default, same pattern as hook_id fix in 049.

ALTER TABLE user_hooks ALTER COLUMN event DROP NOT NULL;
ALTER TABLE user_hooks ALTER COLUMN event SET DEFAULT '';
