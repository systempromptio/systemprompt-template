-- Core 61 removed its legacy organization model. These tables are owned by
-- this web extension and remain required by the internal billing and access
-- control surfaces.
CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    seat_limit INTEGER,
    monthly_cost_cap_microdollars BIGINT,
    monthly_price_microdollars BIGINT NOT NULL DEFAULT 0,
    grants JSONB NOT NULL DEFAULT '[]'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    plan_id TEXT REFERENCES plans(id) ON DELETE SET NULL,
    seat_limit_override INTEGER,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'cancelled')),
    is_platform BOOLEAN NOT NULL DEFAULT FALSE,
    email_domains TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    contract_start DATE,
    contract_end DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_organizations_plan ON organizations(plan_id);
CREATE INDEX IF NOT EXISTS idx_organizations_status ON organizations(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_platform ON organizations(is_platform) WHERE is_platform;
CREATE INDEX IF NOT EXISTS idx_organizations_email_domains ON organizations USING GIN(email_domains);
CREATE TABLE IF NOT EXISTS organization_members (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    org_role TEXT NOT NULL DEFAULT 'member' CHECK (org_role IN ('owner', 'admin', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_organization_members_org ON organization_members(org_id);
CREATE INDEX IF NOT EXISTS idx_organization_members_org_role ON organization_members(org_id, org_role);
CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    org_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE user_profile_ext ADD COLUMN IF NOT EXISTS department TEXT NOT NULL DEFAULT 'Default';
CREATE INDEX IF NOT EXISTS idx_user_profile_ext_department ON user_profile_ext(department);
