-- Phase 2: Plan Approval Workflow
ALTER TABLE template ADD COLUMN IF NOT EXISTS require_approval BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS terraform_plan (
    id                BIGSERIAL PRIMARY KEY,
    task_id           INTEGER NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    project_id        INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    plan_output       TEXT    NOT NULL DEFAULT '',
    plan_json         TEXT,
    resources_added   INTEGER NOT NULL DEFAULT 0,
    resources_changed INTEGER NOT NULL DEFAULT 0,
    resources_removed INTEGER NOT NULL DEFAULT 0,
    status            TEXT    NOT NULL DEFAULT 'pending',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at       TIMESTAMPTZ,
    reviewed_by       INTEGER REFERENCES "user"(id) ON DELETE SET NULL,
    review_comment    TEXT
);

CREATE INDEX IF NOT EXISTS idx_tf_plan_task_id       ON terraform_plan(task_id);
CREATE INDEX IF NOT EXISTS idx_tf_plan_project_status ON terraform_plan(project_id, status);
