-- Vibe Everywhere SQLite Migration 001: Initial Schema
-- Creates all core tables for the Vibe Everywhere backend

-- Client devices table
CREATE TABLE IF NOT EXISTS client_devices (
    device_id       TEXT PRIMARY KEY NOT NULL,
    device_name     TEXT NOT NULL,
    device_type     TEXT NOT NULL CHECK (device_type IN ('mobile', 'desktop')),
    authorized_at   TEXT NOT NULL DEFAULT (datetime('now')),
    server_url      TEXT NOT NULL,
    last_seen_at    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Hosts table
CREATE TABLE IF NOT EXISTS hosts (
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

-- Workspaces table
CREATE TABLE IF NOT EXISTS workspaces (
    workspace_id    TEXT PRIMARY KEY NOT NULL,
    host_id         TEXT NOT NULL REFERENCES hosts(host_id) ON DELETE CASCADE,
    path            TEXT NOT NULL,
    display_name    TEXT NOT NULL,
    is_favorited    INTEGER NOT NULL DEFAULT 0,
    last_used_at    TEXT,
    exists_on_host  INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(host_id, path)
);

CREATE INDEX IF NOT EXISTS idx_workspaces_host_id ON workspaces(host_id);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    session_id                TEXT PRIMARY KEY NOT NULL,
    title                     TEXT NOT NULL,
    host_id                   TEXT NOT NULL REFERENCES hosts(host_id),
    workspace_id              TEXT NOT NULL REFERENCES workspaces(workspace_id),
    agent_type                TEXT NOT NULL DEFAULT 'claude_code',
    status                    TEXT NOT NULL DEFAULT 'running'
                              CHECK (status IN ('running', 'pending', 'dispatching', 'waiting_approval', 'paused', 'error', 'closing', 'archived')),
    last_activity_at          TEXT,
    latest_summary            TEXT,
    unread_event_count        INTEGER NOT NULL DEFAULT 0,
    pending_permission_count  INTEGER NOT NULL DEFAULT 0,
    can_resume_cross_device   INTEGER NOT NULL DEFAULT 1,
    claude_session_id         TEXT,
    created_at                TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at                TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_host_id ON sessions(host_id);
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

-- Session messages table
CREATE TABLE IF NOT EXISTS session_messages (
    message_id    TEXT PRIMARY KEY NOT NULL,
    session_id    TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
    message_type  TEXT NOT NULL CHECK (message_type IN ('user', 'assistant', 'system', 'tool', 'error', 'permission')),
    content       TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_session_messages_session_id ON session_messages(session_id);

-- Permission requests table
CREATE TABLE IF NOT EXISTS permission_requests (
    permission_id TEXT PRIMARY KEY NOT NULL,
    session_id    TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
    risk_type     TEXT NOT NULL CHECK (risk_type IN ('write_fs', 'exec_cmd', 'network')),
    summary       TEXT NOT NULL,
    target        TEXT,
    status        TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved_once', 'denied_once', 'approved_session', 'expired')),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    responded_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_permission_requests_session_id ON permission_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_permission_requests_status ON permission_requests(status);

-- Session archives table
CREATE TABLE IF NOT EXISTS session_archives (
    archive_id    TEXT PRIMARY KEY NOT NULL,
    session_id    TEXT NOT NULL,
    title         TEXT NOT NULL,
    closed_at     TEXT NOT NULL,
    close_reason  TEXT NOT NULL CHECK (close_reason IN ('user_closed', 'completed', 'failed', 'terminated')),
    host_id       TEXT NOT NULL,
    workspace_id  TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_session_archives_host_id ON session_archives(host_id);
CREATE INDEX IF NOT EXISTS idx_session_archives_workspace_id ON session_archives(workspace_id);

-- Notification preferences table
CREATE TABLE IF NOT EXISTS notification_preferences (
    device_id                  TEXT PRIMARY KEY NOT NULL REFERENCES client_devices(device_id) ON DELETE CASCADE,
    enabled                    INTEGER NOT NULL DEFAULT 1,
    permission_request_enabled INTEGER NOT NULL DEFAULT 1,
    task_completed_enabled     INTEGER NOT NULL DEFAULT 1,
    task_failed_enabled        INTEGER NOT NULL DEFAULT 1,
    session_error_enabled      INTEGER NOT NULL DEFAULT 1
);

-- Pairing codes table (for temporary pairing state)
CREATE TABLE IF NOT EXISTS pairing_codes (
    pair_code       TEXT PRIMARY KEY NOT NULL,
    host_id         TEXT NOT NULL,
    host_name       TEXT NOT NULL,
    platform        TEXT NOT NULL,
    qr_payload      TEXT,
    pairing_secret  TEXT,
    expires_at      TEXT NOT NULL,
    used            INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_pairing_codes_expires_at ON pairing_codes(expires_at);

-- Idempotency keys table (for duplicate request protection)
CREATE TABLE IF NOT EXISTS idempotency_keys (
    key          TEXT PRIMARY KEY NOT NULL,
    session_id   TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_created_at ON idempotency_keys(created_at);
