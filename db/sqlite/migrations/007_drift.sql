-- GitOps Drift Detection tables
CREATE TABLE IF NOT EXISTS drift_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    template_id INTEGER NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    schedule TEXT,
    created TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS drift_result (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    drift_config_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    template_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'clean',
    summary TEXT,
    task_id INTEGER,
    checked_at TEXT NOT NULL DEFAULT (datetime('now'))
);
