-- Vibe Everywhere PostgreSQL Migration 010: Host FK cascades
--
-- Add ON DELETE CASCADE to sessions.host_id so deleting a host
-- automatically removes its sessions (and transitively session_messages,
-- permission_requests via their own ON DELETE CASCADE).
--
-- Add a foreign key to session_archives.host_id referencing hosts(host_id)
-- so database-level enforcement matches application-level cleanup.

-- 1. Recreate sessions.host_id FK with ON DELETE CASCADE
ALTER TABLE sessions DROP CONSTRAINT IF EXISTS sessions_host_id_fkey;
ALTER TABLE sessions ADD CONSTRAINT sessions_host_id_fkey
    FOREIGN KEY (host_id) REFERENCES hosts(host_id) ON DELETE CASCADE;

-- 2. Add FK to session_archives.host_id
ALTER TABLE session_archives ADD CONSTRAINT session_archives_host_id_fkey
    FOREIGN KEY (host_id) REFERENCES hosts(host_id) ON DELETE CASCADE;
