-- ============================================================================
-- Скрипт наполнения PostgreSQL тестовыми данными для Semaphore UI
-- ============================================================================
-- Использование:
--   docker exec -i semaphore-db psql -U semaphore -d semaphore < fill-demo-data.sql
--
-- Этот скрипт добавляет полные демонстрационные данные для всех сущностей:
--   - Пользователи (4 шт)
--   - Проекты (4 шт)
--   - Ключи доступа (5 шт)
--   - Инвентари (5 шт)
--   - Репозитории (5 шт)
--   - Окружения (5 шт)
--   - Шаблоны (12 шт)
--   - Расписания (4 шт)
--   - Задачи (6 шт)
-- ============================================================================

-- ============================================================================
-- Пользователи (пароль для всех: demo123, хеш bcrypt)
-- ============================================================================
-- Хеш сгенерирован: python3 -c "import bcrypt; print(bcrypt.hashpw(b'demo123', bcrypt.gensalt(rounds=10)).decode())"
INSERT INTO "user" (id, username, name, email, password, admin, external, alert, pro, created) VALUES
(1, 'admin', 'Administrator', 'admin@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', TRUE, FALSE, TRUE, FALSE, NOW()),
(2, 'john.doe', 'John Doe', 'john.doe@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', FALSE, FALSE, FALSE, FALSE, NOW()),
(3, 'jane.smith', 'Jane Smith', 'jane.smith@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', FALSE, FALSE, TRUE, FALSE, NOW()),
(4, 'devops', 'DevOps Engineer', 'devops@semaphore.local', '$2b$10$0anHX0Pp7RcBDzt.3IWPhevop4sw/s5KvuZwygk2F8ULH/zaHlFoi', FALSE, FALSE, FALSE, FALSE, NOW())
ON CONFLICT (username) DO NOTHING;

-- ============================================================================
-- Проекты
-- ============================================================================
INSERT INTO project (id, name, created, alert, max_parallel_tasks, type) VALUES
(1, 'Demo Infrastructure', NOW(), TRUE, 5, 'default'),
(2, 'Web Application Deployment', NOW(), FALSE, 3, 'default'),
(3, 'Database Management', NOW(), TRUE, 2, 'default'),
(4, 'Security & Compliance', NOW(), FALSE, 1, 'default')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Связи пользователей с проектами
-- ============================================================================
-- Admin имеет доступ ко всем проектам как owner
INSERT INTO project_user (project_id, user_id, role, created) VALUES
(1, 1, 'owner', NOW()),
(2, 1, 'owner', NOW()),
(3, 1, 'owner', NOW()),
(4, 1, 'owner', NOW())
ON CONFLICT (project_id, user_id) DO NOTHING;

-- John Doe работает с Web Application
INSERT INTO project_user (project_id, user_id, role, created) VALUES
(2, 2, 'manager', NOW())
ON CONFLICT (project_id, user_id) DO NOTHING;

-- Jane Smith работает с Database и Security
INSERT INTO project_user (project_id, user_id, role, created) VALUES
(3, 3, 'manager', NOW()),
(4, 3, 'task_runner', NOW())
ON CONFLICT (project_id, user_id) DO NOTHING;

-- DevOps работает со всеми проектами как task_runner
INSERT INTO project_user (project_id, user_id, role, created) VALUES
(1, 4, 'task_runner', NOW()),
(2, 4, 'task_runner', NOW()),
(3, 4, 'task_runner', NOW()),
(4, 4, 'task_runner', NOW())
ON CONFLICT (project_id, user_id) DO NOTHING;

-- ============================================================================
-- Ключи доступа (Access Keys)
-- ============================================================================
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
-----END OPENSSH PRIVATE KEY-----', NULL, NULL, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Инвентари
-- ============================================================================
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
1, 5, 'root', 22, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Репозитории
-- ============================================================================
INSERT INTO repository (id, project_id, name, git_url, git_type, git_branch, key_id, created) VALUES
(1, 1, 'Infrastructure Playbooks', 'https://github.com/semaphore-demo/infrastructure-playbooks.git', 'git', 'main', 1, NOW()),
(2, 2, 'Web App Deployment', 'https://github.com/semaphore-demo/webapp-deploy.git', 'git', 'master', 3, NOW()),
(3, 3, 'Database Playbooks', 'https://github.com/semaphore-demo/db-playbooks.git', 'git', 'main', 4, NOW()),
(4, 4, 'Security Scripts', 'https://github.com/semaphore-demo/security-scripts.git', 'git', 'master', 5, NOW()),
(5, 1, 'Common Roles', 'https://github.com/semaphore-demo/common-roles.git', 'git', 'develop', 1, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Окружения (Environment)
-- ============================================================================
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
}', NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Шаблоны (Templates)
-- ============================================================================
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
(12, 4, 5, 4, 5, 'Patch Security', 'Исправление уязвимостей', 'patch-security.yml', '[]', FALSE, 'master', TRUE, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Расписания (Schedules)
-- ============================================================================
INSERT INTO schedule (project_id, template_id, cron, name, active, created) VALUES
(1, 2, '0 3 * * 0', 'Weekly Server Update', TRUE, NOW()),
(3, 7, '0 2 * * *', 'Daily Database Backup', TRUE, NOW()),
(4, 10, '0 4 * * 1', 'Weekly Security Scan', TRUE, NOW()),
(4, 11, '0 6 * * *', 'Daily Compliance Check', TRUE, NOW())
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Задачи (Tasks) - демонстрационные
-- ============================================================================
INSERT INTO task (id, template_id, project_id, status, playbook, user_id, created, start_time, end_time, message) VALUES
(1, 1, 1, 'success', 'site.yml', 1, NOW() - INTERVAL '7 days', NOW() - INTERVAL '7 days' + INTERVAL '5 minutes', NOW() - INTERVAL '7 days' + INTERVAL '10 minutes', 'Infrastructure deployed successfully'),
(2, 4, 2, 'success', 'deploy-webapp.yml', 2, NOW() - INTERVAL '5 days', NOW() - INTERVAL '5 days' + INTERVAL '3 minutes', NOW() - INTERVAL '5 days' + INTERVAL '8 minutes', 'Web App v1.2.0 deployed'),
(3, 7, 3, 'success', 'backup.yml', 3, NOW() - INTERVAL '1 day', NOW() - INTERVAL '1 day' + INTERVAL '15 minutes', NOW() - INTERVAL '1 day' + INTERVAL '45 minutes', 'Database backup completed'),
(4, 10, 4, 'success', 'security-scan.yml', 4, NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days' + INTERVAL '20 minutes', NOW() - INTERVAL '2 days' + INTERVAL '35 minutes', 'Security scan completed, no critical issues'),
(5, 2, 1, 'running', 'update.yml', 1, NOW() - INTERVAL '1 hour', NOW() - INTERVAL '1 hour', NULL, 'Server update in progress'),
(6, 1, 1, 'waiting', 'site.yml', 4, NOW(), NULL, NULL, 'Waiting for execution')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Вывод задач (Task Output)
-- ============================================================================
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
Backups uploaded to S3')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- События (Events)
-- ============================================================================
INSERT INTO event (project_id, user_id, task_id, object_type, description, created) VALUES
(1, 1, 1, 'task', 'Task #1 "Deploy Infrastructure" completed successfully', NOW() - INTERVAL '7 days'),
(2, 2, 2, 'task', 'Task #2 "Deploy Web App" completed successfully', NOW() - INTERVAL '5 days'),
(3, 3, 3, 'task', 'Task #3 "Backup Databases" completed successfully', NOW() - INTERVAL '1 day'),
(4, 4, 4, 'task', 'Task #4 "Security Scan" completed successfully', NOW() - INTERVAL '2 days'),
(1, 1, NULL, 'project', 'Project "Demo Infrastructure" created', NOW() - INTERVAL '30 days'),
(2, 1, NULL, 'project', 'Project "Web Application Deployment" created', NOW() - INTERVAL '25 days')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- Опции
-- ============================================================================
INSERT INTO "option" (key, value) VALUES
    ('demo_mode', 'true'),
    ('demo_initialized_at', NOW()),
    ('jwt_secret', 'demo-secret-key-12345'),
    ('session_timeout', '86400'),
    ('telegram_chat_id', ''),
    ('telegram_token', '')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;

-- ============================================================================
-- Сброс последовательностей для предотвращения конфликтов ID
-- ============================================================================
SELECT setval('project_id_seq', (SELECT COALESCE(MAX(id), 0) FROM project) + 1, false);
SELECT setval('"user_id_seq"', (SELECT COALESCE(MAX(id), 0) FROM "user") + 1, false);
SELECT setval('template_id_seq', (SELECT COALESCE(MAX(id), 0) FROM template) + 1, false);
SELECT setval('inventory_id_seq', (SELECT COALESCE(MAX(id), 0) FROM inventory) + 1, false);
SELECT setval('repository_id_seq', (SELECT COALESCE(MAX(id), 0) FROM repository) + 1, false);
SELECT setval('environment_id_seq', (SELECT COALESCE(MAX(id), 0) FROM environment) + 1, false);
SELECT setval('access_key_id_seq', (SELECT COALESCE(MAX(id), 0) FROM access_key) + 1, false);
SELECT setval('task_id_seq', (SELECT COALESCE(MAX(id), 0) FROM task) + 1, false);
SELECT setval('schedule_id_seq', (SELECT COALESCE(MAX(id), 0) FROM schedule) + 1, false);
SELECT setval('event_id_seq', (SELECT COALESCE(MAX(id), 0) FROM event) + 1, false);

-- ============================================================================
-- Проверка результатов
-- ============================================================================
SELECT '=== DEMO DATA LOADED SUCCESSFULLY! ===' AS status;
SELECT 'Users: ' || COUNT(*) FROM "user";
SELECT 'Projects: ' || COUNT(*) FROM project;
SELECT 'Project-User links: ' || COUNT(*) FROM project_user;
SELECT 'Access Keys: ' || COUNT(*) FROM access_key;
SELECT 'Inventories: ' || COUNT(*) FROM inventory;
SELECT 'Repositories: ' || COUNT(*) FROM repository;
SELECT 'Environments: ' || COUNT(*) FROM environment;
SELECT 'Templates: ' || COUNT(*) FROM template;
SELECT 'Schedules: ' || COUNT(*) FROM schedule;
SELECT 'Tasks: ' || COUNT(*) FROM task;
SELECT 'Events: ' || COUNT(*) FROM event;

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
-- Шаблоны (12 шт):
--   - Deploy Infrastructure, Update Servers, Staging Deploy
--   - Deploy Web App, Rollback Web App, Scale Web App
--   - Backup Databases, Restore Database, DB Health Check
--   - Security Scan, Compliance Check, Patch Security
--
-- ============================================================================
