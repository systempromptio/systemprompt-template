-- The catch-all department every user lands in without an explicit
-- assignment. Insert-if-absent: an operator's edits survive every boot.
INSERT INTO departments (name, description)
VALUES ('Default', 'Default department — contains every user without an explicit assignment.')
ON CONFLICT (name) DO NOTHING;
