-- LDAP Group → Team Mapping
CREATE TABLE IF NOT EXISTS ldap_group_mapping (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ldap_group_dn TEXT NOT NULL,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'task:runner',
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    UNIQUE(ldap_group_dn, project_id)
);
