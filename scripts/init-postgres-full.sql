-- Полная схема БД для Semaphore (PostgreSQL)
-- Все таблицы необходимые для работы

-- Таблица пользователей (уже есть, создаём если нет)
CREATE TABLE IF NOT EXISTS "user" (
    id SERIAL PRIMARY KEY,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    username VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    admin BOOLEAN NOT NULL DEFAULT FALSE,
    external BOOLEAN NOT NULL DEFAULT FALSE,
    alert BOOLEAN NOT NULL DEFAULT FALSE,
    pro BOOLEAN NOT NULL DEFAULT FALSE,
    totp TEXT,
    email_otp TEXT
);

-- Таблица проектов (уже есть)
CREATE TABLE IF NOT EXISTS project (
    id SERIAL PRIMARY KEY,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    name VARCHAR(255) NOT NULL,
    alert BOOLEAN NOT NULL DEFAULT FALSE,
    alert_chat VARCHAR(255),
    max_parallel_tasks INTEGER NOT NULL DEFAULT 0,
    type VARCHAR(50) NOT NULL DEFAULT 'default',
    default_secret_storage_id INTEGER
);

-- Связь пользователей и проектов (уже есть)
CREATE TABLE IF NOT EXISTS project_user (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_id, user_id)
);

-- Таблица шаблонов (templates)
CREATE TABLE IF NOT EXISTS template (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    inventory_id INTEGER,
    repository_id INTEGER,
    environment_id INTEGER,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    playbook VARCHAR(255) NOT NULL,
    arguments TEXT,
    allow_override_args_in_task BOOLEAN NOT NULL DEFAULT FALSE,
    survey_var TEXT,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    vault_key_id INTEGER,
    type VARCHAR(50) NOT NULL DEFAULT 'ansible',
    app VARCHAR(50) NOT NULL DEFAULT 'ansible',
    git_branch VARCHAR(255),
    git_depth INTEGER DEFAULT 1,
    diff BOOLEAN NOT NULL DEFAULT FALSE,
    operator_id INTEGER,
    last_task_id INTEGER
);

-- Таблица инвентарей
CREATE TABLE IF NOT EXISTS inventory (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    inventory TEXT NOT NULL,
    inventory_format VARCHAR(50) NOT NULL DEFAULT 'yaml',
    ssh_key_id INTEGER,
    become_key_id INTEGER,
    type VARCHAR(50) NOT NULL DEFAULT 'static',
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица репозиториев
CREATE TABLE IF NOT EXISTS repository (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    git_url VARCHAR(510) NOT NULL,
    git_branch VARCHAR(255) NOT NULL DEFAULT 'master',
    ssh_key_id INTEGER,
    access_key_id INTEGER,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица окружений (environment)
CREATE TABLE IF NOT EXISTS environment (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    json TEXT NOT NULL,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица ключей доступа (access_key)
CREATE TABLE IF NOT EXISTS access_key (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    secret TEXT,
    authorization_header TEXT,
    login_password TEXT,
    user_id INTEGER,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица задач (tasks)
CREATE TABLE IF NOT EXISTS task (
    id SERIAL PRIMARY KEY,
    template_id INTEGER NOT NULL REFERENCES template(id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'waiting',
    playbook VARCHAR(255),
    arguments TEXT,
    task_limit VARCHAR(255),
    debug BOOLEAN NOT NULL DEFAULT FALSE,
    dry_run BOOLEAN NOT NULL DEFAULT FALSE,
    diff BOOLEAN NOT NULL DEFAULT FALSE,
    user_id INTEGER,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    start_time TIMESTAMP WITH TIME ZONE,
    end_time TIMESTAMP WITH TIME ZONE,
    message TEXT,
    commit_hash VARCHAR(255),
    commit_message TEXT,
    commit_author VARCHAR(255)
);

-- Вывод задач (task output)
CREATE TABLE IF NOT EXISTS task_output (
    id SERIAL PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES task(id) ON DELETE CASCADE,
    task VARCHAR(50),
    time TIMESTAMP WITH TIME ZONE,
    output TEXT NOT NULL
);

-- Расписания (schedule)
CREATE TABLE IF NOT EXISTS schedule (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    template_id INTEGER NOT NULL REFERENCES template(id) ON DELETE CASCADE,
    cron VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- Playbook таблицы (добавлено в Q1 2026)
-- ============================================================================

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

-- Сессии
CREATE TABLE IF NOT EXISTS session (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ip VARCHAR(50),
    user_agent TEXT,
    expired BOOLEAN NOT NULL DEFAULT FALSE
);

-- API токены
CREATE TABLE IF NOT EXISTS api_token (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    token VARCHAR(255) NOT NULL UNIQUE,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expired BOOLEAN NOT NULL DEFAULT FALSE
);

-- События (events)
CREATE TABLE IF NOT EXISTS event (
    id SERIAL PRIMARY KEY,
    project_id INTEGER REFERENCES project(id) ON DELETE CASCADE,
    user_id INTEGER REFERENCES "user"(id) ON DELETE SET NULL,
    task_id INTEGER REFERENCES task(id) ON DELETE SET NULL,
    object_id INTEGER,
    object_type VARCHAR(50),
    description TEXT NOT NULL,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Опции (options)
CREATE TABLE IF NOT EXISTS "option" (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT
);

-- Миграции (уже есть)
CREATE TABLE IF NOT EXISTS migration (
    version BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

-- Индексы
CREATE INDEX IF NOT EXISTS idx_task_template ON task(template_id);
CREATE INDEX IF NOT EXISTS idx_task_project ON task(project_id);
CREATE INDEX IF NOT EXISTS idx_task_created ON task(created);
CREATE INDEX IF NOT EXISTS idx_template_project ON template(project_id);
CREATE INDEX IF NOT EXISTS idx_inventory_project ON inventory(project_id);
CREATE INDEX IF NOT EXISTS idx_repository_project ON repository(project_id);
CREATE INDEX IF NOT EXISTS idx_environment_project ON environment(project_id);
CREATE INDEX IF NOT EXISTS idx_access_key_project ON access_key(project_id);
CREATE INDEX IF NOT EXISTS idx_schedule_project ON schedule(project_id);
CREATE INDEX IF NOT EXISTS idx_event_project ON event(project_id);
CREATE INDEX IF NOT EXISTS idx_event_created ON event(created);
CREATE INDEX IF NOT EXISTS idx_task_output_task ON task_output(task_id);

-- Индексы для playbook таблиц
CREATE INDEX IF NOT EXISTS idx_playbook_project ON playbook(project_id);
CREATE INDEX IF NOT EXISTS idx_playbook_name ON playbook(name);
CREATE INDEX IF NOT EXISTS idx_playbook_type ON playbook(playbook_type);
CREATE INDEX IF NOT EXISTS idx_playbook_run_project ON playbook_run(project_id);
CREATE INDEX IF NOT EXISTS idx_playbook_run_playbook ON playbook_run(playbook_id);
CREATE INDEX IF NOT EXISTS idx_playbook_run_task ON playbook_run(task_id);
CREATE INDEX IF NOT EXISTS idx_playbook_run_status ON playbook_run(status);
CREATE INDEX IF NOT EXISTS idx_playbook_run_created ON playbook_run(created);
CREATE INDEX IF NOT EXISTS idx_playbook_run_user ON playbook_run(user_id);
