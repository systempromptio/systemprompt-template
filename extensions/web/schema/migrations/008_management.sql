-- Backfill departments from any pre-existing free-text users.department values.
INSERT INTO departments (name)
SELECT DISTINCT department
FROM users
WHERE department IS NOT NULL AND department <> ''
ON CONFLICT (name) DO NOTHING;

-- Widen access_control_rules.entity_type to include 'skill' and 'gateway_route'.
-- The declarative schema already lists the new allow-set; this migration rewrites
-- the constraint on pre-existing installs whose constraint pre-dates the widening.
ALTER TABLE access_control_rules DROP CONSTRAINT IF EXISTS access_control_rules_entity_type_check;
ALTER TABLE access_control_rules ADD CONSTRAINT access_control_rules_entity_type_check
    CHECK (entity_type IN ('plugin', 'agent', 'mcp_server', 'marketplace', 'skill', 'gateway_route'));
