PRAGMA foreign_keys = OFF;
BEGIN IMMEDIATE;

CREATE TABLE sessions_new (
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
    rerun_from_session_id     TEXT REFERENCES sessions(session_id),
    created_at                TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at                TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO sessions_new (
    session_id, title, host_id, workspace_id, agent_type, status,
    last_activity_at, latest_summary, unread_event_count, pending_permission_count,
    can_resume_cross_device, claude_session_id, rerun_from_session_id, created_at, updated_at
)
SELECT
    session_id, title, host_id, workspace_id, agent_type, status,
    last_activity_at, latest_summary, unread_event_count, pending_permission_count,
    can_resume_cross_device, claude_session_id, rerun_from_session_id, created_at, updated_at
FROM sessions;

DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;

CREATE INDEX IF NOT EXISTS idx_sessions_host_id ON sessions(host_id);
CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_rerun_from_session_id ON sessions(rerun_from_session_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_active_rerun_from_session_id
    ON sessions(rerun_from_session_id)
    WHERE rerun_from_session_id IS NOT NULL AND status NOT IN ('dispatching', 'archived', 'error');
CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_dispatching_rerun_from_session_id
    ON sessions(rerun_from_session_id)
    WHERE rerun_from_session_id IS NOT NULL AND status = 'dispatching';

COMMIT;
PRAGMA foreign_keys = ON;
