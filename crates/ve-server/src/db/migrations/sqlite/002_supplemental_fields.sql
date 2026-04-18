-- Vibe Everywhere SQLite Migration 002: Supplemental Fields
-- Adds missing fields per ve-supplemental-schema-config.md

-- Add missing columns to idempotency_keys table
-- Note: SQLite doesn't support ADD COLUMN with complex defaults in one statement
-- We add columns individually

ALTER TABLE idempotency_keys ADD COLUMN request_hash TEXT;
ALTER TABLE idempotency_keys ADD COLUMN result_type TEXT NOT NULL DEFAULT 'session';
ALTER TABLE idempotency_keys ADD COLUMN expires_at TEXT;

-- Create index for expires_at to support cleanup queries
DROP INDEX IF EXISTS idx_idempotency_keys_created_at;
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_expires_at ON idempotency_keys(expires_at);

-- Add metadata_json column to session_archives for archive metadata
ALTER TABLE session_archives ADD COLUMN metadata_json TEXT;

-- Note: SQLite doesn't enforce NOT NULL on ALTER TABLE ADD COLUMN
-- The application layer should handle NULL metadata_json gracefully
