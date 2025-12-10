-- ============================================================================
-- USERS TABLE - Core user identity and profile information (PostgreSQL)
-- ============================================================================
--
-- Role Schema Expectations:
-- -------------------------
-- The 'roles' field stores a TEXT[] array of role strings for PostgreSQL.
-- TEXT[] provides native PostgreSQL array operations without JSON parsing overhead.
--
-- Standard Roles:
--   - "user"      : Default role for all registered users
--   - "admin"     : Full system administration privileges
--   - "moderator" : Content moderation privileges
--   - "anonymous" : Temporary/guest users (auto-cleanup after 30 days)
--
-- Role Characteristics:
--   - Low cardinality (1-3 roles per user typically)
--   - Simple array structure, not hierarchical
--   - Extensible (new roles can be added without schema changes)
--   - Role membership checks use ANY operator (e.g., 'admin' = ANY(roles))
--   - TEXT[] allows efficient GIN indexing if needed: CREATE INDEX idx_users_roles ON users USING GIN(roles);
--
-- Benefits of TEXT[] over JSONB:
--   - Native array operations: 'admin' = ANY(roles)
--   - Better performance (no JSON parsing)
--   - Type safety (can only contain strings)
--   - sqlx automatic handling: Vec<String> works directly
--   - Simpler queries and indexing
--
-- Examples:
--   - Regular user:      ARRAY['user']::TEXT[]
--   - Admin user:        ARRAY['user', 'admin']::TEXT[]
--   - Temporary user:    ARRAY['anonymous']::TEXT[]
--
-- Migration: See migrations/001_roles_jsonb_to_array.sql
-- ============================================================================

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY, -- UUID as primary key for external references
    name VARCHAR(255) NOT NULL UNIQUE, -- Username/login name
    email VARCHAR(255) NOT NULL UNIQUE,
    full_name VARCHAR(255),
    display_name VARCHAR(255),
    -- User status and roles (authorization, not authentication)
    status TEXT CHECK(status IN ('active', 'inactive', 'suspended', 'pending', 'deleted', 'temporary')) DEFAULT 'active',
    email_verified BOOLEAN DEFAULT false,
    roles TEXT[] NOT NULL DEFAULT ARRAY['user']::TEXT[], -- TEXT[] array of roles (see documentation above)
    -- Bot detection (moved from sessions to users for persistence)
    is_bot BOOLEAN NOT NULL DEFAULT false,
    is_scanner BOOLEAN NOT NULL DEFAULT false,
    -- Profile data
    avatar_url TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_name ON users(name);
CREATE INDEX IF NOT EXISTS idx_users_bot_status ON users(is_bot, is_scanner);
