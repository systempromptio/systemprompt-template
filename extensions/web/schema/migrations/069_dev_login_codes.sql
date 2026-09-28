-- One-shot developer login codes for `/admin/auth/dev/login`.
--
-- Only the SHA-256 of a code is stored, and a row is consumed atomically on
-- first use, so a code copied out of a terminal or a log line is worth
-- nothing once redeemed and nothing after its window. The table is separate
-- from bridge_exchange_codes on purpose: a bridge device-link code must never
-- be redeemable as a browser session, nor the reverse.

CREATE TABLE IF NOT EXISTS dev_login_codes (
    code_hash   TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at  TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_dev_login_codes_user ON dev_login_codes(user_id);
