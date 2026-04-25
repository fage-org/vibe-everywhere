-- Migration 010: Host pending daemon token
-- Stores daemon tokens when daemon is offline during pairing completion.
-- Daemon retrieves and clears this on first connection after pairing.

ALTER TABLE hosts ADD COLUMN pending_daemon_token TEXT;
