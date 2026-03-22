CREATE TABLE IF NOT EXISTS notification_policy (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    channel_type TEXT NOT NULL DEFAULT 'slack',
    webhook_url TEXT NOT NULL,
    trigger TEXT NOT NULL DEFAULT 'on_failure' CHECK (trigger IN ('on_failure', 'on_success', 'on_start', 'always')),
    template_id INTEGER,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
