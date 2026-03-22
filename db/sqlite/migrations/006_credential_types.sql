CREATE TABLE IF NOT EXISTS credential_type (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    input_schema TEXT NOT NULL DEFAULT '[]',
    injectors TEXT NOT NULL DEFAULT '[]',
    created DATETIME NOT NULL DEFAULT (datetime('now')),
    updated DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS credential_instance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    credential_type_id INTEGER NOT NULL REFERENCES credential_type(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    "values" TEXT NOT NULL DEFAULT '{}',
    description TEXT,
    created DATETIME NOT NULL DEFAULT (datetime('now'))
);
