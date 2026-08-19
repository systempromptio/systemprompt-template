-- The operator's own tenant, and the catch-all department every user lands in
-- without an explicit assignment.
--
-- Insert-if-absent only: services/access-control/plans.yaml is the ongoing
-- source of truth for plans and organizations (the publish pipeline upserts it
-- on every boot), but the pipeline runs after install, and the Default
-- department needs an organization to belong to now. `is_platform` marks this
-- as the operator's tenant — the enterprise console's authorisation boundary.
INSERT INTO plans (id, name, description, seat_limit, monthly_cost_cap_microdollars)
VALUES ('house', 'House', 'Astound-internal. Unlimited seats, no spend cap.', NULL, NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO organizations (id, slug, name, plan_id, is_platform)
VALUES ('house', 'house', 'Astound Digital', 'house', TRUE)
ON CONFLICT (id) DO NOTHING;

INSERT INTO departments (name, description, org_id)
VALUES ('Default', 'Default department — contains every user without an explicit assignment.', 'house')
ON CONFLICT (org_id, name) DO NOTHING;
