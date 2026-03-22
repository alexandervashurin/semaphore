-- Workflows (DAG automation pipelines)
CREATE TABLE IF NOT EXISTS workflow (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_node (
    id SERIAL PRIMARY KEY,
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    template_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    pos_x DOUBLE PRECISION NOT NULL DEFAULT 0,
    pos_y DOUBLE PRECISION NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS workflow_edge (
    id SERIAL PRIMARY KEY,
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    from_node_id INTEGER NOT NULL REFERENCES workflow_node(id) ON DELETE CASCADE,
    to_node_id INTEGER NOT NULL REFERENCES workflow_node(id) ON DELETE CASCADE,
    condition TEXT NOT NULL DEFAULT 'success' CHECK (condition IN ('success', 'failure', 'always'))
);

CREATE TABLE IF NOT EXISTS workflow_run (
    id SERIAL PRIMARY KEY,
    workflow_id INTEGER NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'success', 'failed', 'cancelled')),
    message TEXT,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started TIMESTAMPTZ,
    finished TIMESTAMPTZ
);
