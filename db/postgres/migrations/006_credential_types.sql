CREATE TABLE IF NOT EXISTS credential_type (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    input_schema TEXT NOT NULL DEFAULT '[]',
    injectors TEXT NOT NULL DEFAULT '[]',
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS credential_instance (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    credential_type_id INTEGER NOT NULL REFERENCES credential_type(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    "values" TEXT NOT NULL DEFAULT '{}',
    description TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
