CREATE TABLE IF NOT EXISTS banned_ips (
    ip_address VARCHAR(45) PRIMARY KEY,
    reason VARCHAR(255) NOT NULL,
    banned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    ban_count INTEGER DEFAULT 1,
    last_offense_path VARCHAR(512),
    last_user_agent TEXT,
    is_permanent BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_banned_ips_expires ON banned_ips(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX idx_banned_ips_banned_at ON banned_ips(banned_at);
