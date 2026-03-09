-- ============================================================================
-- Audit Log Migration
-- ============================================================================
-- Таблица для расширенного логирования действий пользователей
-- Поддерживает поиск, фильтрацию и пагинацию
-- ============================================================================

-- Таблица audit_log
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT,
    user_id BIGINT,
    username VARCHAR(255),
    action VARCHAR(100) NOT NULL,
    object_type VARCHAR(50) NOT NULL,
    object_id BIGINT,
    object_name VARCHAR(255),
    description TEXT NOT NULL,
    level VARCHAR(20) NOT NULL DEFAULT 'info',
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSONB,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Индексы для ускорения поиска
CREATE INDEX IF NOT EXISTS idx_audit_log_project_id ON audit_log(project_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_username ON audit_log(username);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_object_type ON audit_log(object_type);
CREATE INDEX IF NOT EXISTS idx_audit_log_object_id ON audit_log(object_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_level ON audit_log(level);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_desc ON audit_log(created DESC);

-- Составные индексы для частых запросов
CREATE INDEX IF NOT EXISTS idx_audit_log_project_created ON audit_log(project_id, created DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_user_created ON audit_log(user_id, created DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_action_created ON audit_log(action, created DESC);

-- GIN индекс для JSONB поля details
CREATE INDEX IF NOT EXISTS idx_audit_log_details_gin ON audit_log USING GIN (details);

-- Комментарий к таблице
COMMENT ON TABLE audit_log IS 'Расширенное логирование действий пользователей для аудита и безопасности';
COMMENT ON COLUMN audit_log.action IS 'Тип действия: login, logout, task_created, template_run, и т.д.';
COMMENT ON COLUMN audit_log.object_type IS 'Тип объекта: user, project, task, template, inventory, repository, и т.д.';
COMMENT ON COLUMN audit_log.level IS 'Уровень важности: info, warning, error, critical';
COMMENT ON COLUMN audit_log.details IS 'Дополнительные данные в формате JSON (IP, user agent, изменения, и т.д.)';
