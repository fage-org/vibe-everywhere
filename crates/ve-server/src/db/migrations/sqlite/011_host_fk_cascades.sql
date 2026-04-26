-- Vibe Everywhere SQLite Migration 011: Host FK cascades
--
-- Add ON DELETE CASCADE to sessions.host_id and session_archives.host_id
-- to match PostgreSQL migration 010 behavior.

PRAGMA foreign_keys = OFF;
BEGIN IMMEDIATE;

-- 1. Rebuild sessions with CASCADE on host_id
CREATE TABLE sessions_new (
    session_id                TEXT PRIMARY KEY NOT NULL,
    title                     TEXT NOT NULL,
    host_id                   TEXT NOT NULL REFERENCES hosts(host_id) ON DELETE CASCADE,
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
    rerun_from_session_id     TEXT REFERENCES sessions(session_id),
    created_at                TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at                TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO sessions_new SELECT * FROM sessions;

DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;

CREATE INDEX IF NOT EXISTS idx_sessions_host_id ON sessions(host_id);
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_rerun_from_session_id ON sessions(rerun_from_session_id);

-- 2. Rebuild session_archives with FK + CASCADE on host_id
CREATE TABLE session_archives_new (
    archive_id    TEXT PRIMARY KEY NOT NULL,
    session_id    TEXT NOT NULL,
    title         TEXT NOT NULL,
    closed_at     TEXT NOT NULL,
    close_reason  TEXT NOT NULL CHECK (close_reason IN ('user_closed', 'completed', 'failed', 'terminated')),
    host_id       TEXT NOT NULL REFERENCES hosts(host_id) ON DELETE CASCADE,
    workspace_id  TEXT NOT NULL,
    metadata_json TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(session_id)
);

INSERT INTO session_archives_new SELECT * FROM session_archives;

DROP TABLE session_archives;
ALTER TABLE session_archives_new RENAME TO session_archives;

CREATE INDEX IF NOT EXISTS idx_session_archives_host_id ON session_archives(host_id);
CREATE INDEX IF NOT EXISTS idx_session_archives_workspace_id ON session_archives(workspace_id);

COMMIT;
PRAGMA foreign_keys = ON;
