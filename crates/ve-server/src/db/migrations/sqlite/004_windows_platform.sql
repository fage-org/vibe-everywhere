-- Vibe Everywhere SQLite Migration 004: allow windows host platform

CREATE TABLE hosts_new (
    host_id         TEXT PRIMARY KEY NOT NULL,
    host_name       TEXT NOT NULL,
    platform        TEXT NOT NULL CHECK (platform IN ('linux', 'macos', 'windows', 'wsl')),
    online_status   TEXT NOT NULL DEFAULT 'unknown' CHECK (online_status IN ('online', 'offline', 'unknown')),
    daemon_status   TEXT NOT NULL DEFAULT 'disconnected' CHECK (daemon_status IN ('healthy', 'connecting', 'disconnected', 'error')),
    last_active_at  TEXT,
    pair_status     TEXT NOT NULL DEFAULT 'pending' CHECK (pair_status IN ('paired', 'pending', 'failed')),
    pair_code       TEXT,
    qr_payload      TEXT,
    token_hash      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO hosts_new (
    host_id,
    host_name,
    platform,
    online_status,
    daemon_status,
    last_active_at,
    pair_status,
    pair_code,
    qr_payload,
    token_hash,
    created_at,
    updated_at
)
SELECT
    host_id,
    host_name,
    platform,
    online_status,
    daemon_status,
    last_active_at,
    pair_status,
    pair_code,
    qr_payload,
    token_hash,
    created_at,
    updated_at
FROM hosts;

DROP TABLE hosts;
ALTER TABLE hosts_new RENAME TO hosts;

CREATE INDEX IF NOT EXISTS idx_hosts_online_status ON hosts(online_status);
CREATE INDEX IF NOT EXISTS idx_hosts_daemon_status ON hosts(daemon_status);
CREATE INDEX IF NOT EXISTS idx_hosts_pair_status ON hosts(pair_status);
