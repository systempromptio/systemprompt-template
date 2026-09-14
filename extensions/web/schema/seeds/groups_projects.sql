-- The system group and the two directory-backed projects.
--
-- Insert-if-absent, so an operator's renames survive every boot. `unassigned`
-- is load-bearing: the `group_memberships` view names it as the home of every
-- user with no `group_members` row, and the groups page treats it as a system
-- row that cannot be deleted.
INSERT INTO groups (id, name, description, is_system, source)
VALUES ('unassigned', 'Unassigned', 'Signed in through the directory but mapped to no group. Membership is derived: any user with no group row is here.', true, 'system')
ON CONFLICT (id) DO NOTHING;

INSERT INTO projects (id, name, source)
VALUES ('commerce', 'Commerce', 'yaml'), ('core', 'Core', 'yaml')
ON CONFLICT (id) DO NOTHING;
