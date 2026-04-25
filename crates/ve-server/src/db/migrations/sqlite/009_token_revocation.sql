-- Token revocation infrastructure
-- Add current_jti to client_devices for fast device-level revocation
ALTER TABLE client_devices ADD COLUMN current_jti TEXT;

-- Revoked tokens table for individual token revocation
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti         TEXT PRIMARY KEY,
    device_id   TEXT NOT NULL REFERENCES client_devices(device_id) ON DELETE CASCADE,
    revoked_at  TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_device_id ON revoked_tokens(device_id);
