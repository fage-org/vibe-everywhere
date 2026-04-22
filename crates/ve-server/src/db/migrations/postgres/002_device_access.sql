-- Device-scoped access control for hosts and sessions

CREATE TABLE IF NOT EXISTS device_host_access (
    device_id   VARCHAR(64) NOT NULL REFERENCES client_devices(device_id) ON DELETE CASCADE,
    host_id     VARCHAR(64) NOT NULL REFERENCES hosts(host_id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (device_id, host_id)
);

CREATE INDEX IF NOT EXISTS idx_device_host_access_host_id
    ON device_host_access(host_id);

CREATE TABLE IF NOT EXISTS device_session_access (
    device_id   VARCHAR(64) NOT NULL REFERENCES client_devices(device_id) ON DELETE CASCADE,
    session_id  VARCHAR(64) NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (device_id, session_id)
);

CREATE INDEX IF NOT EXISTS idx_device_session_access_session_id
    ON device_session_access(session_id);
