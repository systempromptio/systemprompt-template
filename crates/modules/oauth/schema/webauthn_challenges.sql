CREATE TABLE IF NOT EXISTS webauthn_challenges (
    challenge TEXT PRIMARY KEY NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    challenge_type VARCHAR(255) NOT NULL,
    session_state TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_user_id ON webauthn_challenges(user_id);
CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_expires_at ON webauthn_challenges(expires_at);
