-- ============================================================================
-- Скрипт наполнения БД тестовыми данными для Semaphore UI
-- ============================================================================
-- Использование: sqlite3 data/semaphore.db < fill-test-data.sql
-- ============================================================================

-- Дополнительные проекты
INSERT INTO project (name, created, alert, max_parallel_tasks, type) VALUES
    ('Infrastructure', datetime('now'), 0, 0, 'default'),
    ('Web Applications', datetime('now'), 0, 0, 'default'),
    ('Database Cluster', datetime('now'), 0, 0, 'default');

-- Дополнительные пользователи (пароль для всех: demo123)
INSERT INTO user (username, name, email, password, admin, external, alert, pro, created) VALUES
    ('john.doe', 'John Doe', 'john@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', 0, 0, 0, 0, datetime('now')),
    ('jane.smith', 'Jane Smith', 'jane@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', 0, 0, 0, 0, datetime('now')),
    ('devops', 'DevOps User', 'devops@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', 0, 0, 0, 0, datetime('now'));

-- Связи пользователей с проектами (для всех проектов)
INSERT INTO project_user (project_id, user_id, role, created) VALUES
    -- Проект 1: Demo Project
    (1, 1, 'owner', datetime('now')),
    (1, 2, 'manager', datetime('now')),
    (1, 3, 'manager', datetime('now')),
    (1, 4, 'worker', datetime('now')),
    -- Проект 2: Infrastructure
    (2, 1, 'owner', datetime('now')),
    (2, 2, 'manager', datetime('now')),
    (2, 4, 'worker', datetime('now')),
    -- Проект 3: Web Applications
    (3, 1, 'owner', datetime('now')),
    (3, 3, 'manager', datetime('now')),
    (3, 4, 'worker', datetime('now')),
    -- Проект 4: Infrastructure (доп)
    (4, 1, 'owner', datetime('now')),
    (4, 2, 'manager', datetime('now')),
    (4, 3, 'manager', datetime('now')),
    (4, 4, 'worker', datetime('now')),
    -- Проект 5: Web Applications (доп)
    (5, 1, 'owner', datetime('now')),
    (5, 2, 'manager', datetime('now')),
    (5, 4, 'worker', datetime('now')),
    -- Проект 6: Database Cluster
    (6, 1, 'owner', datetime('now')),
    (6, 2, 'manager', datetime('now')),
    (6, 3, 'manager', datetime('now')),
    (6, 4, 'worker', datetime('now'));

-- Тестовые задачи (task_output)
INSERT INTO task_output (task_id, project_id, output, time, stage_id) VALUES
    (1, 1, 'Task executed successfully', datetime('now'), NULL),
    (1, 1, 'Deployment completed', datetime('now'), NULL),
    (1, 2, 'Infrastructure updated', datetime('now'), NULL);

-- Опции
INSERT OR IGNORE INTO option (key, value) VALUES
    ('demo_mode', 'true'),
    ('jwt_secret', 'demo-secret-key-12345'),
    ('session_timeout', '86400');

