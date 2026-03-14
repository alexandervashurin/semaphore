-- Migration: Add playbooks table
-- Created: 2026-03-11

-- Таблица для хранения playbook файлов
CREATE TABLE IF NOT EXISTS playbook (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    description TEXT,
    playbook_type VARCHAR(50) NOT NULL DEFAULT 'ansible',
    repository_id INTEGER REFERENCES repository(id) ON DELETE SET NULL,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Индексы для производительности
CREATE INDEX IF NOT EXISTS idx_playbook_project ON playbook(project_id);
CREATE INDEX IF NOT EXISTS idx_playbook_name ON playbook(name);
CREATE INDEX IF NOT EXISTS idx_playbook_type ON playbook(playbook_type);

-- Комментарии
COMMENT ON TABLE playbook IS 'Хранилище playbook файлов (Ansible, Terraform, Shell)';
COMMENT ON COLUMN playbook.project_id IS 'ID проекта';
COMMENT ON COLUMN playbook.name IS 'Название плейбука';
COMMENT ON COLUMN playbook.content IS 'YAML содержимое плейбука';
COMMENT ON COLUMN playbook.playbook_type IS 'Тип: ansible, terraform, shell';
COMMENT ON COLUMN playbook.repository_id IS 'Связь с git репозиторием (опционально)';
