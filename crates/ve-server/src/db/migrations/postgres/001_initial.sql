-- Vibe Everywhere PostgreSQL Migration 001: Initial Schema
-- Creates all core tables for the Vibe Everywhere backend

-- Client devices table
CREATE TABLE IF NOT EXISTS client_devices (
    device_id       VARCHAR(64) PRIMARY KEY NOT NULL,
    device_name     VARCHAR(255) NOT NULL,
    device_type     VARCHAR(20) NOT NULL CHECK (device_type IN ('mobile', 'desktop')),
    authorized_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    legacy_acl      INTEGER NOT NULL DEFAULT 1,
    server_url      VARCHAR(512) NOT NULL,
    last_seen_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Hosts table
CREATE TABLE IF NOT EXISTS hosts (
    host_id         VARCHAR(64) PRIMARY KEY NOT NULL,
    host_name       VARCHAR(255) NOT NULL,
    platform        VARCHAR(20) NOT NULL CHECK (platform IN ('linux', 'macos', 'windows', 'wsl')),
    online_status   VARCHAR(20) NOT NULL DEFAULT 'unknown' CHECK (online_status IN ('online', 'offline', 'unknown')),
    daemon_status   VARCHAR(20) NOT NULL DEFAULT 'disconnected' CHECK (daemon_status IN ('healthy', 'connecting', 'disconnected', 'error')),
    last_active_at  TIMESTAMPTZ,
    pair_status     VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (pair_status IN ('paired', 'pending', 'failed')),
    pair_code       VARCHAR(32),
    qr_payload      TEXT,
    token_hash      VARCHAR(128),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workspaces table
CREATE TABLE IF NOT EXISTS workspaces (
    workspace_id    VARCHAR(64) PRIMARY KEY NOT NULL,
    host_id         VARCHAR(64) NOT NULL REFERENCES hosts(host_id) ON DELETE CASCADE,
    path            VARCHAR(1024) NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    is_favorited    BOOLEAN NOT NULL DEFAULT FALSE,
    last_used_at    TIMESTAMPTZ,
    exists_on_host  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(host_id, path)
);

CREATE INDEX IF NOT EXISTS idx_workspaces_host_id ON workspaces(host_id);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    session_id                VARCHAR(64) PRIMARY KEY NOT NULL,
    title                     VARCHAR(512) NOT NULL,
    host_id                   VARCHAR(64) NOT NULL REFERENCES hosts(host_id),
    workspace_id              VARCHAR(64) NOT NULL REFERENCES workspaces(workspace_id),
    agent_type                VARCHAR(32) NOT NULL DEFAULT 'claude_code',
    status                    VARCHAR(32) NOT NULL DEFAULT 'running'
                              CHECK (status IN ('running', 'pending', 'dispatching', 'waiting_approval', 'paused', 'error', 'closing', 'archived')),
    last_activity_at          TIMESTAMPTZ,
    latest_summary            TEXT,
    unread_event_count        INTEGER NOT NULL DEFAULT 0,
    pending_permission_count  INTEGER NOT NULL DEFAULT 0,
    can_resume_cross_device   BOOLEAN NOT NULL DEFAULT TRUE,
    claude_session_id         VARCHAR(128),
    created_at                TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_host_id ON sessions(host_id);
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

-- Session messages table
CREATE TABLE IF NOT EXISTS session_messages (
    message_id    VARCHAR(64) PRIMARY KEY NOT NULL,
    session_id    VARCHAR(64) NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
    message_type  VARCHAR(32) NOT NULL CHECK (message_type IN ('user', 'assistant', 'system', 'tool', 'error', 'permission')),
    content       TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_session_messages_session_id ON session_messages(session_id);

-- Permission requests table
CREATE TABLE IF NOT EXISTS permission_requests (
    permission_id VARCHAR(64) PRIMARY KEY NOT NULL,
    session_id    VARCHAR(64) NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
    risk_type     VARCHAR(32) NOT NULL CHECK (risk_type IN ('write_fs', 'exec_cmd', 'network')),
    summary       TEXT NOT NULL,
    target        TEXT,
    status        VARCHAR(32) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved_once', 'denied_once', 'approved_session', 'expired')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at  TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_permission_requests_session_id ON permission_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_permission_requests_status ON permission_requests(status);

-- Session archives table
CREATE TABLE IF NOT EXISTS session_archives (
    archive_id    VARCHAR(64) PRIMARY KEY NOT NULL,
    session_id    VARCHAR(64) NOT NULL,
    title         VARCHAR(512) NOT NULL,
    closed_at     TIMESTAMPTZ NOT NULL,
    close_reason  VARCHAR(32) NOT NULL CHECK (close_reason IN ('user_closed', 'completed', 'failed', 'terminated')),
    host_id       VARCHAR(64) NOT NULL,
    workspace_id  VARCHAR(64) NOT NULL,
    metadata_json JSONB,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_session_archives_host_id ON session_archives(host_id);
CREATE INDEX IF NOT EXISTS idx_session_archives_workspace_id ON session_archives(workspace_id);

-- Notification preferences table
CREATE TABLE IF NOT EXISTS notification_preferences (
    device_id                  VARCHAR(64) PRIMARY KEY NOT NULL REFERENCES client_devices(device_id) ON DELETE CASCADE,
    enabled                    BOOLEAN NOT NULL DEFAULT TRUE,
    permission_request_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    task_completed_enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    task_failed_enabled        BOOLEAN NOT NULL DEFAULT TRUE,
    session_error_enabled      BOOLEAN NOT NULL DEFAULT TRUE
);

-- Pairing codes table (for temporary pairing state)
CREATE TABLE IF NOT EXISTS pairing_codes (
    pair_code       VARCHAR(32) PRIMARY KEY NOT NULL,
    host_id         VARCHAR(64) NOT NULL,
    host_name       VARCHAR(255) NOT NULL,
    platform        VARCHAR(20) NOT NULL,
    qr_payload      TEXT,
    pairing_secret  VARCHAR(128),
    expires_at      TIMESTAMPTZ NOT NULL,
    used            BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pairing_codes_expires_at ON pairing_codes(expires_at);

-- Idempotency keys table (for duplicate request protection)
CREATE TABLE IF NOT EXISTS idempotency_keys (
    key          VARCHAR(128) PRIMARY KEY NOT NULL,
    session_id   VARCHAR(64) NOT NULL,
    request_hash VARCHAR(64),
    result_type  VARCHAR(50) NOT NULL DEFAULT 'session',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours')
);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_expires_at ON idempotency_keys(expires_at);
