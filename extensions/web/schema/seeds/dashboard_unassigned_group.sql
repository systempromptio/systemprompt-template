-- The unassigned bucket is shared across installations, not tenant data.
INSERT INTO groups (id, name, description, is_system, source) VALUES
    ('unassigned', 'Unassigned', 'People without a group membership', true, 'system')
ON CONFLICT (id) DO NOTHING;
