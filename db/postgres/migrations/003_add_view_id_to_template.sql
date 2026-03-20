-- ============================================================================
-- Миграция: Добавление таблицы view и колонки view_id
-- Дата: 2026-03-20
-- Описание: 
--   - Создаёт таблицу view для представлений шаблонов
--   - Добавляет колонку view_id в таблицу template
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

-- Добавляем колонку view_id в таблицу template
ALTER TABLE template ADD COLUMN IF NOT EXISTS view_id INTEGER;

-- Индекс для view_id (опционально, для улучшения производительности)
CREATE INDEX IF NOT EXISTS idx_template_view_id ON template(view_id);

-- Примечание:
-- view используется для группировки и фильтрации шаблонов в UI
-- view_id в template позволяет связать шаблон с представлением
