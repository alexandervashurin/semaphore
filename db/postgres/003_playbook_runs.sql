-- Migration: Add playbook_run table for tracking playbook execution history
-- Created: 2026-03-12

-- Таблица для хранения истории запусков playbook
CREATE TABLE IF NOT EXISTS playbook_run (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    playbook_id INTEGER NOT NULL REFERENCES playbook(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES task(id) ON DELETE SET NULL,
    template_id INTEGER REFERENCES template(id) ON DELETE SET NULL,
    
    -- Статус выполнения
    status VARCHAR(50) NOT NULL DEFAULT 'waiting',
    
    -- Параметры запуска
    inventory_id INTEGER REFERENCES inventory(id) ON DELETE SET NULL,
    environment_id INTEGER REFERENCES environment(id) ON DELETE SET NULL,
    extra_vars TEXT,
    limit_hosts VARCHAR(500),
    tags TEXT,
    skip_tags TEXT,
    
    -- Результаты
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    
    -- Статистика
    hosts_total INTEGER DEFAULT 0,
    hosts_changed INTEGER DEFAULT 0,
    hosts_unreachable INTEGER DEFAULT 0,
    hosts_failed INTEGER DEFAULT 0,
    
    -- Вывод
    output TEXT,
    error_message TEXT,
    
    -- Пользователь
    user_id INTEGER REFERENCES "user"(id) ON DELETE SET NULL,
    
    -- Метаданные
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Индексы для производительности
CREATE INDEX IF NOT EXISTS idx_playbook_run_project ON playbook_run(project_id);
CREATE INDEX IF NOT EXISTS idx_playbook_run_playbook ON playbook_run(playbook_id);
CREATE INDEX IF NOT EXISTS idx_playbook_run_task ON playbook_run(task_id);
CREATE INDEX IF NOT EXISTS idx_playbook_run_status ON playbook_run(status);
CREATE INDEX IF NOT EXISTS idx_playbook_run_created ON playbook_run(created);
CREATE INDEX IF NOT EXISTS idx_playbook_run_user ON playbook_run(user_id);

-- Комментарии
COMMENT ON TABLE playbook_run IS 'История запусков playbook';
COMMENT ON COLUMN playbook_run.project_id IS 'ID проекта';
COMMENT ON COLUMN playbook_run.playbook_id IS 'ID playbook';
COMMENT ON COLUMN playbook_run.task_id IS 'ID задачи (task)';
COMMENT ON COLUMN playbook_run.status IS 'Статус: waiting, running, success, failed, cancelled';
COMMENT ON COLUMN playbook_run.inventory_id IS 'ID инвентаря';
COMMENT ON COLUMN playbook_run.environment_id IS 'ID окружения';
COMMENT ON COLUMN playbook_run.extra_vars IS 'Дополнительные переменные (JSON)';
COMMENT ON COLUMN playbook_run.limit_hosts IS 'Ограничение по хостам';
COMMENT ON COLUMN playbook_run.tags IS 'Теги для запуска';
COMMENT ON COLUMN playbook_run.skip_tags IS 'Пропускаемые теги';
COMMENT ON COLUMN playbook_run.start_time IS 'Время начала выполнения';
COMMENT ON COLUMN playbook_run.end_time IS 'Время завершения';
COMMENT ON COLUMN playbook_run.duration_seconds IS 'Длительность в секундах';
COMMENT ON COLUMN playbook_run.hosts_total IS 'Всего хостов';
COMMENT ON COLUMN playbook_run.hosts_changed IS 'Изменено хостов';
COMMENT ON COLUMN playbook_run.hosts_unreachable IS 'Недоступных хостов';
COMMENT ON COLUMN playbook_run.hosts_failed IS 'Хостов с ошибками';
COMMENT ON COLUMN playbook_run.output IS 'Вывод playbook';
COMMENT ON COLUMN playbook_run.error_message IS 'Сообщение об ошибке';
COMMENT ON COLUMN playbook_run.user_id IS 'ID пользователя, запустившего playbook';
