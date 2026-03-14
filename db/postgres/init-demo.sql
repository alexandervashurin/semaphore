-- ============================================================================
-- Инициализация БД PostgreSQL для Semaphore с демонстрационными данными
-- ============================================================================
-- Этот файл автоматически применяется при первом запуске PostgreSQL через
-- docker-entrypoint-initdb.d/
--
-- Использование:
--   docker-compose -f docker-compose.postgres.yml up -d
--
-- Файл содержит:
--   1. Полную схему БД
--   2. Демонстрационные данные (пользователи, проекты, шаблоны, ключи)
-- ============================================================================

-- ============================================================================
-- ЧАСТЬ 1: Схема БД
-- ============================================================================

-- Таблица пользователей
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

-- Таблица проектов
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

-- Связь пользователей и проектов
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
    inventory_type VARCHAR(50) NOT NULL DEFAULT 'static',
    inventory_data TEXT NOT NULL,
    key_id INTEGER,
    secret_storage_id INTEGER,
    ssh_login VARCHAR(255) DEFAULT 'root',
    ssh_port INTEGER DEFAULT 22,
    extra_vars TEXT,
    ssh_key_id INTEGER,
    become_key_id INTEGER,
    vaults TEXT,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица репозиториев
CREATE TABLE IF NOT EXISTS repository (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    git_url VARCHAR(510) NOT NULL,
    git_type VARCHAR(50) NOT NULL DEFAULT 'git',
    git_branch VARCHAR(255),
    key_id INTEGER NOT NULL DEFAULT 0,
    git_path VARCHAR(255),
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица окружений (environment)
CREATE TABLE IF NOT EXISTS environment (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    json TEXT NOT NULL,
    secret_storage_id INTEGER,
    secrets TEXT,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица ключей доступа (access_key)
CREATE TABLE IF NOT EXISTS access_key (
    id SERIAL PRIMARY KEY,
    project_id INTEGER REFERENCES project(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    user_id INTEGER,
    login_password_login VARCHAR(255),
    login_password_password TEXT,
    ssh_key TEXT,
    ssh_passphrase TEXT,
    access_key_access_key TEXT,
    access_key_secret_key TEXT,
    secret_storage_id INTEGER,
    environment_id INTEGER,
    owner VARCHAR(50) DEFAULT 'project',
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

-- Миграции
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
CREATE INDEX IF NOT EXISTS idx_user_email ON "user"(email);
CREATE INDEX IF NOT EXISTS idx_user_username ON "user"(username);
CREATE INDEX IF NOT EXISTS idx_project_name ON project(name);

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

-- ============================================================================
-- ЧАСТЬ 2: Демонстрационные данные
-- ============================================================================

-- Пользователи
-- Пароль для всех пользователей: 'demo123' (хеш bcrypt)
-- Хеш сгенерирован командой: python3 -c "import bcrypt; print(bcrypt.hashpw(b'demo123', bcrypt.gensalt(rounds=10)).decode())"
INSERT INTO "user" (id, username, name, email, password, admin, alert) VALUES
(1, 'admin', 'Administrator', 'admin@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', TRUE, TRUE),
(2, 'john.doe', 'John Doe', 'john.doe@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', FALSE, FALSE),
(3, 'jane.smith', 'Jane Smith', 'jane.smith@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', FALSE, TRUE),
(4, 'devops', 'DevOps Engineer', 'devops@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', FALSE, FALSE);

-- Проекты
INSERT INTO project (id, name, alert, max_parallel_tasks, type) VALUES
(1, 'Demo Infrastructure', TRUE, 5, 'default'),
(2, 'Web Application Deployment', FALSE, 3, 'default'),
(3, 'Database Management', TRUE, 2, 'default'),
(4, 'Security & Compliance', FALSE, 1, 'default');

-- Связи пользователей с проектами
INSERT INTO project_user (project_id, user_id, role, created) VALUES
-- Admin имеет доступ ко всем проектам
(1, 1, 'owner', NOW()),
(2, 1, 'owner', NOW()),
(3, 1, 'owner', NOW()),
(4, 1, 'owner', NOW()),
-- John Doe работает с Web Application
(2, 2, 'manager', NOW()),
-- Jane Smith работает с Database и Security
(3, 3, 'manager', NOW()),
(4, 3, 'task_runner', NOW()),
-- DevOps работает со всеми проектами
(1, 4, 'task_runner', NOW()),
(2, 4, 'task_runner', NOW()),
(3, 4, 'task_runner', NOW()),
(4, 4, 'task_runner', NOW());

-- Ключи доступа (Access Keys)
INSERT INTO access_key (id, project_id, name, type, ssh_key, login_password_login, login_password_password, created) VALUES
(1, 1, 'Demo SSH Key', 'ssh', '-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcn
NhAAAAAwEAAQAAAIEA0Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X
5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X
-----END OPENSSH PRIVATE KEY-----', NULL, NULL, NOW()),
(2, 1, 'Demo Login/Password', 'login_password', NULL, 'ansible', 'demo123', NOW()),
(3, 2, 'Web App SSH Key', 'ssh', '-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcn
NhAAAAAwEAAQAAAIEA1Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X
5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X
-----END OPENSSH PRIVATE KEY-----', NULL, NULL, NOW()),
(4, 3, 'DB Admin Key', 'login_password', NULL, 'dbadmin', 'dbpass123', NOW()),
(5, 4, 'Security Audit Key', 'ssh', '-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcn
NhAAAAAwEAAQAAAIEA2Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X
5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X
-----END OPENSSH PRIVATE KEY-----', NULL, NULL, NOW());

-- Инвентари
INSERT INTO inventory (id, project_id, name, inventory_type, inventory_data, key_id, ssh_key_id, ssh_login, ssh_port, created) VALUES
(1, 1, 'Production Servers', 'static',
'all:
  children:
    webservers:
      hosts:
        web1.example.com:
          ansible_user: ansible
          ansible_port: 22
        web2.example.com:
          ansible_user: ansible
          ansible_port: 22
    databases:
      hosts:
        db1.example.com:
          ansible_user: ansible
          ansible_port: 22
        db2.example.com:
          ansible_user: ansible
          ansible_port: 22
    monitoring:
      hosts:
        monitor1.example.com:
          ansible_user: ansible
          ansible_port: 22',
1, 1, 'root', 22, NOW()),
(2, 1, 'Staging Environment', 'static',
'[staging]
staging-web1 ansible_host=192.168.1.100 ansible_user=ubuntu
staging-app1 ansible_host=192.168.1.101 ansible_user=ubuntu

[staging:vars]
ansible_port=22
ansible_ssh_private_key_file=~/.ssh/staging_key',
1, 1, 'ubuntu', 22, NOW()),
(3, 2, 'Web App Cluster', 'static',
'all:
  children:
    frontend:
      hosts:
        frontend1:
          ansible_host: 10.0.1.10
        frontend2:
          ansible_host: 10.0.1.11
    backend:
      hosts:
        backend1:
          ansible_host: 10.0.2.10
        backend2:
          ansible_host: 10.0.2.11
    loadbalancer:
      hosts:
        lb1:
          ansible_host: 10.0.0.10',
1, 3, 'root', 22, NOW()),
(4, 3, 'Database Cluster', 'static',
'[postgres_primary]
pg-primary ansible_host=192.168.10.10

[postgres_replica]
pg-replica1 ansible_host=192.168.10.11
pg-replica2 ansible_host=192.168.10.12

[mysql_cluster]
mysql1 ansible_host=192.168.10.20
mysql2 ansible_host=192.168.10.21',
1, 4, 'postgres', 22, NOW()),
(5, 4, 'Security Scan Targets', 'static',
'security_targets:
  hosts:
    target1:
      ansible_host: 192.168.100.1
    target2:
      ansible_host: 192.168.100.2
    target3:
      ansible_host: 192.168.100.3',
1, 5, 'root', 22, NOW());

-- Репозитории
INSERT INTO repository (id, project_id, name, git_url, git_type, git_branch, key_id, created) VALUES
(1, 1, 'Infrastructure Playbooks', 'https://github.com/semaphore-demo/infrastructure-playbooks.git', 'git', 'main', 1, NOW()),
(2, 2, 'Web App Deployment', 'https://github.com/semaphore-demo/webapp-deploy.git', 'git', 'master', 3, NOW()),
(3, 3, 'Database Playbooks', 'https://github.com/semaphore-demo/db-playbooks.git', 'git', 'main', 4, NOW()),
(4, 4, 'Security Scripts', 'https://github.com/semaphore-demo/security-scripts.git', 'git', 'master', 5, NOW()),
(5, 1, 'Common Roles', 'https://github.com/semaphore-demo/common-roles.git', 'git', 'develop', 1, NOW());

-- Окружения (Environment)
INSERT INTO environment (id, project_id, name, json, created) VALUES
(1, 1, 'Production Variables', '{
  "env": "production",
  "domain": "example.com",
  "ssl_enabled": true,
  "monitoring_enabled": true,
  "backup_enabled": true,
  "log_level": "warn"
}', NOW()),
(2, 1, 'Staging Variables', '{
  "env": "staging",
  "domain": "staging.example.com",
  "ssl_enabled": true,
  "monitoring_enabled": true,
  "backup_enabled": false,
  "log_level": "debug"
}', NOW()),
(3, 2, 'Web App Config', '{
  "app_name": "MyWebApp",
  "app_port": 8080,
  "workers": 4,
  "cache_enabled": true,
  "session_timeout": 3600
}', NOW()),
(4, 3, 'Database Config', '{
  "postgres_version": "15",
  "mysql_version": "8.0",
  "max_connections": 200,
  "shared_buffers": "256MB",
  "backup_retention_days": 7
}', NOW()),
(5, 4, 'Security Scan Config', '{
  "scan_type": "full",
  "severity_threshold": "medium",
  "report_format": "html",
  "notify_on_failure": true
}', NOW());

-- Шаблоны (Templates)
INSERT INTO template (id, project_id, inventory_id, repository_id, environment_id, name, description, playbook, arguments, allow_override_args_in_task, git_branch, diff, created) VALUES
(1, 1, 1, 1, 1, 'Deploy Infrastructure', 'Развертывание инфраструктуры', 'site.yml', '[]', FALSE, 'main', TRUE, NOW()),
(2, 1, 1, 1, 1, 'Update Servers', 'Обновление серверов', 'update.yml', '["--tags", "update"]', TRUE, 'main', FALSE, NOW()),
(3, 1, 2, 5, 2, 'Staging Deploy', 'Деплой на staging', 'deploy.yml', '[]', FALSE, 'develop', TRUE, NOW()),
(4, 2, 3, 2, 3, 'Deploy Web App', 'Деплой веб-приложения', 'deploy-webapp.yml', '[]', FALSE, 'master', TRUE, NOW()),
(5, 2, 3, 2, 3, 'Rollback Web App', 'Откат веб-приложения', 'rollback.yml', '[]', FALSE, 'master', FALSE, NOW()),
(6, 2, 3, 2, 3, 'Scale Web App', 'Масштабирование веб-приложения', 'scale.yml', '[]', TRUE, 'master', FALSE, NOW()),
(7, 3, 4, 3, 4, 'Backup Databases', 'Резервное копирование БД', 'backup.yml', '[]', FALSE, 'main', FALSE, NOW()),
(8, 3, 4, 3, 4, 'Restore Database', 'Восстановление БД', 'restore.yml', '[]', FALSE, 'main', FALSE, NOW()),
(9, 3, 4, 3, 4, 'DB Health Check', 'Проверка здоровья БД', 'healthcheck.yml', '[]', FALSE, 'main', FALSE, NOW()),
(10, 4, 5, 4, 5, 'Security Scan', 'Сканирование безопасности', 'security-scan.yml', '[]', FALSE, 'master', TRUE, NOW()),
(11, 4, 5, 4, 5, 'Compliance Check', 'Проверка соответствия', 'compliance.yml', '[]', FALSE, 'master', FALSE, NOW()),
(12, 4, 5, 4, 5, 'Patch Security', 'Исправление уязвимостей', 'patch-security.yml', '[]', FALSE, 'master', TRUE, NOW());

-- Расписания (Schedules)
INSERT INTO schedule (project_id, template_id, cron, name, active, created) VALUES
(1, 2, '0 3 * * 0', 'Weekly Server Update', TRUE, NOW()),
(3, 7, '0 2 * * *', 'Daily Database Backup', TRUE, NOW()),
(4, 10, '0 4 * * 1', 'Weekly Security Scan', TRUE, NOW()),
(4, 11, '0 6 * * *', 'Daily Compliance Check', TRUE, NOW());

-- Задачи (Tasks) - демонстрационные
INSERT INTO task (id, template_id, project_id, status, playbook, user_id, created, start_time, end_time, message) VALUES
(1, 1, 1, 'success', 'site.yml', 1, NOW() - INTERVAL '7 days', NOW() - INTERVAL '7 days' + INTERVAL '5 minutes', NOW() - INTERVAL '7 days' + INTERVAL '10 minutes', 'Infrastructure deployed successfully'),
(2, 4, 2, 'success', 'deploy-webapp.yml', 2, NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days' + INTERVAL '3 minutes', NOW() - INTERVAL '5 days' + INTERVAL '8 minutes', 'Web App v1.2.0 deployed'),
(3, 7, 3, 'success', 'backup.yml', 3, NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day' + INTERVAL '15 minutes', NOW() - INTERVAL '1 day' + INTERVAL '45 minutes', 'Database backup completed'),
(4, 10, 4, 'success', 'security-scan.yml', 4, NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days' + INTERVAL '20 minutes', NOW() - INTERVAL '2 days' + INTERVAL '35 minutes', 'Security scan completed, no critical issues'),
(5, 2, 1, 'running', 'update.yml', 1, NOW() - INTERVAL '1 hour', NOW() - INTERVAL '1 hour', NULL, 'Server update in progress'),
(6, 1, 1, 'waiting', 'site.yml', 4, NOW(), NULL, NULL, 'Waiting for execution');

-- Вывод задач (Task Output)
INSERT INTO task_output (task_id, task, time, output) VALUES
(1, 'Gathering Facts', NOW() - INTERVAL '7 days' + INTERVAL '1 minute', 'ok: [web1.example.com]
ok: [web2.example.com]
ok: [db1.example.com]
ok: [db2.example.com]'),
(1, 'Deploy Infrastructure', NOW() - INTERVAL '7 days' + INTERVAL '2 minutes', 'changed: [web1.example.com] => (item=nginx)
changed: [web2.example.com] => (item=nginx)
changed: [db1.example.com] => (item=postgresql)'),
(2, 'Deploy Web App', NOW() - INTERVAL '5 days' + INTERVAL '4 minutes', 'TASK [Download application artifact] **********************
changed: [frontend1]
changed: [frontend2]
changed: [backend1]
changed: [backend2]'),
(3, 'Backup Databases', NOW() - INTERVAL '1 day' + INTERVAL '20 minutes', 'PostgreSQL backup completed: 2.5GB
MySQL backup completed: 1.8GB
Backups uploaded to S3');

-- События (Events)
INSERT INTO event (project_id, user_id, task_id, object_type, description, created) VALUES
(1, 1, 1, 'task', 'Task #1 "Deploy Infrastructure" completed successfully', NOW() - INTERVAL '7 days'),
(2, 2, 2, 'task', 'Task #2 "Deploy Web App" completed successfully', NOW() - INTERVAL '5 days'),
(3, 3, 3, 'task', 'Task #3 "Backup Databases" completed successfully', NOW() - INTERVAL '1 day'),
(4, 4, 4, 'task', 'Task #4 "Security Scan" completed successfully', NOW() - INTERVAL '2 days'),
(1, 1, NULL, 'project', 'Project "Demo Infrastructure" created', NOW() - INTERVAL '30 days'),
(2, 1, NULL, 'project', 'Project "Web Application Deployment" created', NOW() - INTERVAL '25 days');

-- Миграции
INSERT INTO migration (version, name) VALUES
(1, 'initial_schema'),
(2, 'add_alert_columns'),
(3, 'add_schedule_table'),
(4, 'add_api_token_table');

-- Опции
INSERT INTO "option" (key, value) VALUES
('demo_mode', 'true'),
('demo_initialized_at', NOW()),
('telegram_chat_id', ''),
('telegram_token', '');

-- ============================================================================
-- ЧАСТЬ 3: Информация о демонстрационных данных
-- ============================================================================

-- ============================================================================
-- Сброс последовательностей для предотвращения конфликтов ID
-- ============================================================================
-- После вставки демо-данных с явными ID необходимо обновить последовательности
-- чтобы новые записи начинались со следующих доступных ID

SELECT setval('project_id_seq', (SELECT COALESCE(MAX(id), 0) FROM project) + 1, false);
SELECT setval('"user_id_seq"', (SELECT COALESCE(MAX(id), 0) FROM "user") + 1, false);
SELECT setval('template_id_seq', (SELECT COALESCE(MAX(id), 0) FROM template) + 1, false);
SELECT setval('inventory_id_seq', (SELECT COALESCE(MAX(id), 0) FROM inventory) + 1, false);
SELECT setval('repository_id_seq', (SELECT COALESCE(MAX(id), 0) FROM repository) + 1, false);
SELECT setval('environment_id_seq', (SELECT COALESCE(MAX(id), 0) FROM environment) + 1, false);
SELECT setval('access_key_id_seq', (SELECT COALESCE(MAX(id), 0) FROM access_key) + 1, false);
SELECT setval('task_id_seq', (SELECT COALESCE(MAX(id), 0) FROM task) + 1, false);
SELECT setval('schedule_id_seq', (SELECT COALESCE(MAX(id), 0) FROM schedule) + 1, false);

-- ============================================================================
-- ДЕМО ДАННЫЕ УСПЕШНО ЗАГРУЖЕНЫ!
-- ============================================================================
--
-- Пользователи (пароль для всех: demo123):
--   - admin (Administrator) - администратор, доступ ко всем проектам
--   - john.doe (John Doe) - менеджер проекта Web Application
--   - jane.smith (Jane Smith) - менеджер проекта Database Management
--   - devops (DevOps Engineer) - исполнитель задач
--
-- Проекты:
--   1. Demo Infrastructure - основная инфраструктура
--   2. Web Application Deployment - деплой веб-приложений
--   3. Database Management - управление базами данных
--   4. Security & Compliance - безопасность и соответствие
--
-- Шаблоны:
--   - Deploy Infrastructure
--   - Update Servers
--   - Staging Deploy
--   - Deploy Web App
--   - Rollback Web App
--   - Backup Databases
--   - Security Scan
--   - и другие...
--
-- ============================================================================
