-- LDAP Group → Team Mapping
CREATE TABLE IF NOT EXISTS ldap_group_mapping (
    id SERIAL PRIMARY KEY,
    ldap_group_dn TEXT NOT NULL,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'task:runner',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(ldap_group_dn, project_id)
);

CREATE INDEX IF NOT EXISTS idx_ldap_group_mapping_project ON ldap_group_mapping(project_id);
CREATE INDEX IF NOT EXISTS idx_ldap_group_mapping_dn ON ldap_group_mapping(ldap_group_dn);
