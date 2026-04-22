ALTER TABLE sessions
ADD COLUMN IF NOT EXISTS rerun_from_session_id VARCHAR(64) REFERENCES sessions(session_id);

CREATE INDEX IF NOT EXISTS idx_sessions_rerun_from_session_id
    ON sessions(rerun_from_session_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_active_rerun_from_session_id
    ON sessions(rerun_from_session_id)
    WHERE rerun_from_session_id IS NOT NULL AND status NOT IN ('dispatching', 'archived', 'error');

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_dispatching_rerun_from_session_id
    ON sessions(rerun_from_session_id)
    WHERE rerun_from_session_id IS NOT NULL AND status = 'dispatching';
