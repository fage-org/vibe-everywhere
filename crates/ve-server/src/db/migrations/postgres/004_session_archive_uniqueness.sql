CREATE UNIQUE INDEX IF NOT EXISTS idx_session_archives_session_id_unique
ON session_archives(session_id);
