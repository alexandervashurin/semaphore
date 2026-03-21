-- Task Snapshots for Rollback
CREATE TABLE IF NOT EXISTS task_snapshot (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL,
    template_id INTEGER NOT NULL,
    task_id INTEGER NOT NULL,
    git_branch TEXT,
    git_commit TEXT,
    arguments TEXT,
    inventory_id INTEGER,
    environment_id INTEGER,
    message TEXT,
    label TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_snapshot_project ON task_snapshot(project_id);
CREATE INDEX IF NOT EXISTS idx_task_snapshot_template ON task_snapshot(project_id, template_id);
