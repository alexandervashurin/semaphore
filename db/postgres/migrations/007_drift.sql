-- GitOps Drift Detection tables
CREATE TABLE IF NOT EXISTS drift_config (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    template_id INTEGER NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    schedule TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS drift_result (
    id SERIAL PRIMARY KEY,
    drift_config_id INTEGER NOT NULL REFERENCES drift_config(id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL,
    template_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'clean' CHECK (status IN ('clean', 'drifted', 'error', 'pending')),
    summary TEXT,
    task_id INTEGER,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_drift_result_config ON drift_result(drift_config_id);
CREATE INDEX IF NOT EXISTS idx_drift_result_project ON drift_result(project_id);
