-- ============================================================================
-- Минимальная инициализация БД PostgreSQL для Semaphore
-- ============================================================================
-- Этот файл автоматически применяется при первом запуске PostgreSQL через
-- docker-entrypoint-initdb.d/
--
-- Использование:
--   docker-compose -f docker-compose.postgres.yml up -d
--
-- Или через скрипт:
--   ./scripts/postgres-quick-start.sh
-- ============================================================================

-- Таблица миграций (создаётся первой)

CREATE TABLE IF NOT EXISTS migration (
    version BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

-- Таблица пользователей
CREATE TABLE IF NOT EXISTS "user" (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица проектов
CREATE TABLE IF NOT EXISTS project (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица связей пользователей с проектами
CREATE TABLE IF NOT EXISTS project_user (
    id SERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_id, user_id)
);

-- Индексы для производительности
CREATE INDEX IF NOT EXISTS idx_user_email ON "user"(email);
CREATE INDEX IF NOT EXISTS idx_user_username ON "user"(username);
CREATE INDEX IF NOT EXISTS idx_project_name ON project(name);
CREATE INDEX IF NOT EXISTS idx_project_user_project ON project_user(project_id);
CREATE INDEX IF NOT EXISTS idx_project_user_user ON project_user(user_id);

-- ============================================================================
-- Webhook таблицы (добавлено в Q1 2027)
-- ============================================================================

-- Таблица webhook
CREATE TABLE IF NOT EXISTS webhook (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    secret VARCHAR(255),
    headers JSONB,
    active BOOLEAN DEFAULT TRUE,
    events JSONB NOT NULL DEFAULT '[]',
    retry_count INTEGER DEFAULT 3,
    timeout_secs BIGINT DEFAULT 30,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица истории webhook
CREATE TABLE IF NOT EXISTS webhook_log (
    id BIGSERIAL PRIMARY KEY,
    webhook_id BIGINT REFERENCES webhook(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    status_code INTEGER,
    success BOOLEAN DEFAULT FALSE,
    error TEXT,
    attempts INTEGER DEFAULT 0,
    payload JSONB,
    response JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Индексы для webhook
CREATE INDEX IF NOT EXISTS idx_webhook_project ON webhook(project_id);
CREATE INDEX IF NOT EXISTS idx_webhook_active ON webhook(active) WHERE active = TRUE;
CREATE INDEX IF NOT EXISTS idx_webhook_type ON webhook(type);

-- Индексы для webhook_log
CREATE INDEX IF NOT EXISTS idx_webhook_log_webhook ON webhook_log(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_log_created ON webhook_log(created_at);
CREATE INDEX IF NOT EXISTS idx_webhook_log_event ON webhook_log(event_type);

