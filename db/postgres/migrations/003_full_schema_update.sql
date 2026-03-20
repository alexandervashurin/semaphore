-- ============================================================================
-- Миграция: Полная схема БД для Playbook API и Task
-- Дата: 2026-03-20
-- Описание: Добавляет все недостающие колонки для поддержки Playbook Run API
-- ============================================================================

-- ============================================================================
-- Таблица view
-- ============================================================================

-- Создаём таблицу view если не существует
CREATE TABLE IF NOT EXISTS "view" (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    position INTEGER DEFAULT 0,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT fk_view_project FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE
);

-- Индекс для таблицы view
CREATE INDEX IF NOT EXISTS idx_view_project ON "view"(project_id);

-- ============================================================================
-- Таблица template
-- ============================================================================

-- Добавляем колонки в таблицу template
ALTER TABLE template ADD COLUMN IF NOT EXISTS view_id INTEGER;
ALTER TABLE template ADD COLUMN IF NOT EXISTS build_template_id INTEGER;
ALTER TABLE template ADD COLUMN IF NOT EXISTS autorun BOOLEAN DEFAULT false;
ALTER TABLE template ADD COLUMN IF NOT EXISTS allow_override_args_vars BOOLEAN DEFAULT false;
ALTER TABLE template ADD COLUMN IF NOT EXISTS allow_override_branch_in_task BOOLEAN DEFAULT false;
ALTER TABLE template ADD COLUMN IF NOT EXISTS allow_inventory_in_task BOOLEAN DEFAULT false;
ALTER TABLE template ADD COLUMN IF NOT EXISTS allow_parallel_tasks BOOLEAN DEFAULT false;
ALTER TABLE template ADD COLUMN IF NOT EXISTS suppress_success_alerts BOOLEAN DEFAULT false;
ALTER TABLE template ADD COLUMN IF NOT EXISTS task_params TEXT;
ALTER TABLE template ADD COLUMN IF NOT EXISTS survey_vars TEXT;
ALTER TABLE template ADD COLUMN IF NOT EXISTS vaults TEXT;

-- Индексы для template
CREATE INDEX IF NOT EXISTS idx_template_view_id ON template(view_id);

-- ============================================================================
-- Таблица task
-- ============================================================================

-- Добавляем колонки в таблицу task
ALTER TABLE task ADD COLUMN IF NOT EXISTS environment TEXT;
ALTER TABLE task ADD COLUMN IF NOT EXISTS limit_hosts VARCHAR(255);
ALTER TABLE task ADD COLUMN IF NOT EXISTS tags TEXT;
ALTER TABLE task ADD COLUMN IF NOT EXISTS skip_tags TEXT;
ALTER TABLE task ADD COLUMN IF NOT EXISTS git_branch VARCHAR(255);
ALTER TABLE task ADD COLUMN IF NOT EXISTS repository_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS inventory_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS environment_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS integration_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS playbook_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS schedule_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS event_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS build_task_id INTEGER;
ALTER TABLE task ADD COLUMN IF NOT EXISTS task_args TEXT;
ALTER TABLE task ADD COLUMN IF NOT EXISTS version VARCHAR(50);
ALTER TABLE task ADD COLUMN IF NOT EXISTS repo_path TEXT;
ALTER TABLE task ADD COLUMN IF NOT EXISTS playbook_content TEXT;

-- Индексы для task
CREATE INDEX IF NOT EXISTS idx_task_playbook ON task(playbook_id);
CREATE INDEX IF NOT EXISTS idx_task_inventory ON task(inventory_id);
CREATE INDEX IF NOT EXISTS idx_task_environment ON task(environment_id);
CREATE INDEX IF NOT EXISTS idx_task_integration ON task(integration_id);
CREATE INDEX IF NOT EXISTS idx_task_schedule ON task(schedule_id);

-- ============================================================================
-- Примечания
-- ============================================================================
-- 
-- view: представления для группировки шаблонов в UI
-- view_id: связь шаблона с представлением
-- build_template_id: связь с шаблоном сборки (для CI/CD)
-- autorun: автоматический запуск при commit
-- allow_*: флаги разрешения переопределения параметров в задаче
-- task_params, survey_vars, vaults: дополнительные параметры задачи
-- 
-- task колонки для поддержки playbook run:
-- playbook_id: связь с playbook
-- environment, limit_hosts, tags, skip_tags: параметры запуска Ansible
-- git_branch, repository_id: параметры Git
-- inventory_id, environment_id: ресурсы
-- integration_id, schedule_id, event_id: триггеры
-- build_task_id: связь с задачей сборки
