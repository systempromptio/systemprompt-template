-- Link-based user invites. An admin mints a token (stored only as a SHA-256
-- hash); the invitee opens /admin/invite/{token}, which provisions the user
-- through the passkey path with the org/department/roles recorded here. The
-- explicit invite is the authorization, so acceptance bypasses the
-- email_allowed domain gate that self-serve passkey registration enforces.
-- No email delivery: the create response carries the URL for copy/paste.

CREATE TABLE IF NOT EXISTS user_invites (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    token_hash TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    org_id TEXT NOT NULL,
    department TEXT NOT NULL DEFAULT 'Default',
    roles TEXT[] NOT NULL DEFAULT ARRAY['user']::TEXT[],
    invited_by TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- One live invite per address: a new invite for the same email requires
-- revoking (or accepting) the outstanding one first.
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_invites_pending_email
    ON user_invites(lower(email))
    WHERE accepted_at IS NULL AND revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_user_invites_org
    ON user_invites(org_id, created_at DESC);
