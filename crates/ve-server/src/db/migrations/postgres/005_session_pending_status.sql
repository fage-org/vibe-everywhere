ALTER TABLE sessions
DROP CONSTRAINT IF EXISTS sessions_status_check;

ALTER TABLE sessions
ADD CONSTRAINT sessions_status_check
CHECK (status IN ('running', 'pending', 'dispatching', 'waiting_approval', 'paused', 'error', 'closing', 'archived'));
