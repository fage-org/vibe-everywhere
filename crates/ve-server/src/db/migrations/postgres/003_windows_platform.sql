-- Vibe Everywhere PostgreSQL Migration 003: allow windows host platform

ALTER TABLE hosts DROP CONSTRAINT IF EXISTS hosts_platform_check;
ALTER TABLE hosts
    ADD CONSTRAINT hosts_platform_check
    CHECK (platform IN ('linux', 'macos', 'windows', 'wsl'));
