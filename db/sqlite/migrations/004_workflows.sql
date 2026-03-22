-- Workflows (DAG automation pipelines) - SQLite
CREATE TABLE IF NOT EXISTS workflow (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    description TEXT,
    created DATETIME NOT NULL DEFAULT (datetime('now')),
    updated DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS workflow_node (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    template_id INTEGER NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    pos_x REAL NOT NULL DEFAULT 0,
    pos_y REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS workflow_edge (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    from_node_id INTEGER NOT NULL REFERENCES workflow_node(id) ON DELETE CASCADE,
    to_node_id INTEGER NOT NULL REFERENCES workflow_node(id) ON DELETE CASCADE,
    condition TEXT NOT NULL DEFAULT 'success'
);

CREATE TABLE IF NOT EXISTS workflow_run (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    message TEXT,
    created DATETIME NOT NULL DEFAULT (datetime('now')),
    started DATETIME,
    finished DATETIME
);
