-- Token revocation infrastructure
-- Add current_jti to client_devices for fast device-level revocation
ALTER TABLE client_devices ADD COLUMN current_jti VARCHAR(64);

-- Revoked tokens table for individual token revocation
CREATE TABLE IF NOT EXISTS revoked_tokens (
    jti         VARCHAR(64) PRIMARY KEY,
    device_id   VARCHAR(64) NOT NULL REFERENCES client_devices(device_id) ON DELETE CASCADE,
    revoked_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_revoked_tokens_expires_at ON revoked_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_revoked_tokens_device_id ON revoked_tokens(device_id);
