-- Phase 1: Terraform Remote State Backend
-- HTTP backend compatible with `terraform { backend "http" {} }`

CREATE TABLE IF NOT EXISTS terraform_state (
    id          BIGSERIAL PRIMARY KEY,
    project_id  INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    workspace   TEXT    NOT NULL DEFAULT 'default',
    serial      INTEGER NOT NULL,
    lineage     TEXT    NOT NULL DEFAULT '',
    state_data  BYTEA   NOT NULL,
    encrypted   BOOLEAN NOT NULL DEFAULT FALSE,
    md5         TEXT    NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_id, workspace, serial)
);

CREATE TABLE IF NOT EXISTS terraform_state_lock (
    project_id  INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    workspace   TEXT    NOT NULL DEFAULT 'default',
    lock_id     TEXT    NOT NULL,
    operation   TEXT    NOT NULL DEFAULT '',
    info        TEXT    NOT NULL DEFAULT '',
    who         TEXT    NOT NULL DEFAULT '',
    version     TEXT    NOT NULL DEFAULT '',
    path        TEXT    NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '2 hours',
    PRIMARY KEY (project_id, workspace)
);

CREATE INDEX IF NOT EXISTS idx_tf_state_project_ws
    ON terraform_state(project_id, workspace);
CREATE INDEX IF NOT EXISTS idx_tf_state_lock_expires
    ON terraform_state_lock(expires_at);
