-- Pooled multi-tenancy: customer organizations, seats, and plans.
--
-- An organization is a paying customer. Every user belongs to exactly one
-- (enforced by the PK on organization_members), every department belongs to
-- one, and the plan the org is on decides which marketplaces, plugins, and
-- gateway routes its members can reach.
--
-- Entitlement itself is NOT stored here. A plan is projected into
-- access_control_rules rows at rule_type='organization', rule_value=<org id>,
-- so enforcement is the same resolver that already decides role and department
-- rules — see extensions/web/admin/src/authz/organization.rs.

CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    -- NULL is unlimited. A number is a hard ceiling on active members.
    seat_limit INTEGER,
    -- NULL is uncapped. Enforced by the gateway guard, one request late,
    -- because a request's cost is only known after the response.
    monthly_cost_cap_microdollars BIGINT,
    -- What the customer is billed per month for the licence. The cap above is
    -- what they may spend; this is what they pay. Both are needed to state a
    -- per-organization margin, which is the number the enterprise dashboard
    -- leads with. Zero is a non-billed plan, not an unknown price.
    monthly_price_microdollars BIGINT NOT NULL DEFAULT 0,
    -- The plan's grants, as authored in services/access-control/plans.yaml.
    -- Kept as the projection's source so re-applying a plan to an org can
    -- retract grants the plan no longer carries.
    grants JSONB NOT NULL DEFAULT '[]'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS organizations (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    -- Stable, human-typed, used as the rule_value in access_control_rules.
    -- Renaming an org must not silently orphan its grants, so the slug is
    -- immutable by convention and the display name is what changes.
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    plan_id TEXT REFERENCES plans(id) ON DELETE SET NULL,
    -- Overrides plans.seat_limit when set, for negotiated contracts.
    seat_limit_override INTEGER,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'suspended', 'cancelled')),
    -- The operator's own tenant. Exactly one organization is the platform, and
    -- its admins are the only callers the enterprise console answers to. It is
    -- a column rather than a plan id or a hardcoded slug because it decides an
    -- authorisation boundary, and a boundary must not move when somebody
    -- renames a plan or adds a second organization to the house plan.
    --
    -- Nothing auto-joins it: SSO just-in-time provisioning resolves an
    -- organization by email domain and deliberately skips platform tenants, so
    -- claiming a domain can never mint a platform administrator.
    is_platform BOOLEAN NOT NULL DEFAULT FALSE,
    -- Email domains that map to this organization at SSO just-in-time
    -- provisioning. Without it JIT has no way to decide whose seat a new
    -- federated user consumes, and every enterprise user would arrive
    -- unattached — visible to nobody's billing and entitled to nothing.
    email_domains TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    contract_start DATE,
    contract_end DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_organizations_plan ON organizations(plan_id);
CREATE INDEX IF NOT EXISTS idx_organizations_status ON organizations(status);
-- Two platform tenants would mean two disjoint sets of super-admins, each
-- invisible to the other, so the "exactly one" is a constraint and not a
-- convention.
CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_platform
    ON organizations (is_platform) WHERE is_platform;
CREATE INDEX IF NOT EXISTS idx_organizations_email_domains
    ON organizations USING GIN (email_domains);

-- One row per user. The PK on user_id is the "exactly one org" rule: a user
-- cannot be double-billed or resolve two conflicting organization grants.
CREATE TABLE IF NOT EXISTS organization_members (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    org_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    -- Scope of the customer-side admin surface. 'owner' and 'admin' see their
    -- own org's users, departments, and usage; 'member' sees none of it.
    org_role TEXT NOT NULL DEFAULT 'member'
        CHECK (org_role IN ('owner', 'admin', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_organization_members_org ON organization_members(org_id);
CREATE INDEX IF NOT EXISTS idx_organization_members_org_role
    ON organization_members(org_id, org_role);

-- Departments nest inside organizations. The column itself is declared on
-- `departments` in 12_management.sql, because this file is loaded after it and
-- a declarative schema may not ALTER. The foreign key, the NOT NULL, and the
-- swap of UNIQUE(name) for UNIQUE(org_id, name) all live in
-- migrations/022_organizations_backfill.sql, which is also what backfills
-- existing rows onto the house organization.
