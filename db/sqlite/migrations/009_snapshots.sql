-- Task Snapshots for Rollback
CREATE TABLE IF NOT EXISTS task_snapshot (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
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
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);
