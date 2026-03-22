-- Terraform Cost Estimates (Infracost integration)
CREATE TABLE IF NOT EXISTS cost_estimate (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    task_id INTEGER NOT NULL,
    template_id INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    monthly_cost REAL,
    monthly_cost_diff REAL,
    resource_count INTEGER NOT NULL DEFAULT 0,
    resources_added INTEGER NOT NULL DEFAULT 0,
    resources_changed INTEGER NOT NULL DEFAULT 0,
    resources_deleted INTEGER NOT NULL DEFAULT 0,
    breakdown_json TEXT,
    infracost_version TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_cost_estimate_project ON cost_estimate(project_id);
CREATE INDEX IF NOT EXISTS idx_cost_estimate_task ON cost_estimate(project_id, task_id);
