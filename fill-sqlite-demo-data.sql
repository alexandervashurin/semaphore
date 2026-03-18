-- Демо-данные для Velum (SQLite)
-- Использование: cat fill-sqlite-demo-data.sql | sqlite3 ./data/semaphore.db

-- ============================================================================
-- Пользователи (пароль для всех: admin123)
-- ============================================================================
-- Хеш сгенерирован для admin123
INSERT OR IGNORE INTO user (id, username, name, email, password, admin, external, alert, pro, created) VALUES
(2, 'john.doe', 'John Doe', 'john.doe@localhost', '$2b$12$pDKL.XOgDcQCXBm77saF4eO/84j.Ul1zDhnYPUM61vkqZAUnz9vwS', 0, 0, 0, 0, datetime('now')),
(3, 'jane.smith', 'Jane Smith', 'jane.smith@localhost', '$2b$12$pDKL.XOgDcQCXBm77saF4eO/84j.Ul1zDhnYPUM61vkqZAUnz9vwS', 0, 0, 1, 0, datetime('now')),
(4, 'devops', 'DevOps Engineer', 'devops@localhost', '$2b$12$pDKL.XOgDcQCXBm77saF4eO/84j.Ul1zDhnYPUM61vkqZAUnz9vwS', 0, 0, 0, 0, datetime('now'));

-- ============================================================================
-- Проекты
-- ============================================================================
INSERT OR IGNORE INTO project (id, name, created, alert, max_parallel_tasks, type) VALUES
(1, 'Demo Infrastructure', datetime('now'), 1, 5, 'default'),
(2, 'Web Application Deployment', datetime('now'), 0, 3, 'default'),
(3, 'Database Management', datetime('now'), 1, 2, 'default'),
(4, 'Security & Compliance', datetime('now'), 0, 1, 'default');

-- ============================================================================
-- Связи пользователей с проектами
-- ============================================================================
INSERT OR IGNORE INTO project_user (project_id, user_id, role, created) VALUES
(1, 1, 'owner', datetime('now')),
(2, 1, 'owner', datetime('now')),
(3, 1, 'owner', datetime('now')),
(4, 1, 'owner', datetime('now')),
(2, 2, 'manager', datetime('now')),
(3, 3, 'manager', datetime('now')),
(4, 3, 'task_runner', datetime('now')),
(1, 4, 'task_runner', datetime('now')),
(2, 4, 'task_runner', datetime('now')),
(3, 4, 'task_runner', datetime('now')),
(4, 4, 'task_runner', datetime('now'));

-- ============================================================================
-- Ключи доступа (Access Keys)
-- ============================================================================
INSERT OR IGNORE INTO access_key (id, project_id, name, type, ssh_key, login_password_login, login_password_password, created) VALUES
(1, 1, 'Demo SSH Key', 'ssh', '-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcnNhAAAAAwEAAQAAAIEA0Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X-----END OPENSSH PRIVATE KEY-----', NULL, NULL, datetime('now')),
(2, 1, 'Demo Login/Password', 'login_password', NULL, 'ansible', 'demo123', datetime('now')),
(3, 2, 'Web App SSH Key', 'ssh', '-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcnNhAAAAAwEAAQAAAIEA1Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X-----END OPENSSH PRIVATE KEY-----', NULL, NULL, datetime('now')),
(4, 3, 'DB Admin Key', 'login_password', NULL, 'dbadmin', 'dbpass123', datetime('now')),
(5, 4, 'Security Audit Key', 'ssh', '-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcnNhAAAAAwEAAQAAAIEA2Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X-----END OPENSSH PRIVATE KEY-----', NULL, NULL, datetime('now'));

-- ============================================================================
-- Инвентари
-- ============================================================================
INSERT OR IGNORE INTO inventory (id, project_id, name, inventory_type, inventory_data, ssh_key_id, ssh_login, ssh_port, created) VALUES
(1, 1, 'Production Servers', 'static', 'all:\n  children:\n    webservers:\n      hosts:\n        web1.example.com:\n        web2.example.com:\n    databases:\n      hosts:\n        db1.example.com:\n        db2.example.com:', 1, 'ansible', '22', datetime('now')),
(2, 1, 'Staging Servers', 'static', 'staging:\n  hosts:\n    staging1.example.com:\n    staging2.example.com:', 1, 'ansible', '22', datetime('now')),
(3, 2, 'Web App Inventory', 'static', 'webapp:\n  hosts:\n    app1.example.com:\n    app2.example.com:\n    app3.example.com:', 3, 'ubuntu', '22', datetime('now')),
(4, 3, 'Database Servers', 'static', 'databases:\n  hosts:\n    postgres1.example.com:\n    postgres2.example.com:\n    mysql1.example.com:', 4, 'postgres', '22', datetime('now')),
(5, 4, 'Security Scanners', 'static', 'security:\n  hosts:\n    scanner1.example.com:\n    scanner2.example.com:', 5, 'security', '22', datetime('now'));

-- ============================================================================
-- Репозитории
-- ============================================================================
INSERT OR IGNORE INTO repository (id, project_id, name, git_url, git_branch, ssh_key_id, created) VALUES
(1, 1, 'Infrastructure Code', 'https://github.com/example/infrastructure.git', 'main', 1, datetime('now')),
(2, 2, 'Web Application', 'https://github.com/example/webapp.git', 'main', 3, datetime('now')),
(3, 3, 'Database Scripts', 'https://github.com/example/db-scripts.git', 'main', 4, datetime('now')),
(4, 4, 'Security Policies', 'https://github.com/example/security-policies.git', 'main', 5, datetime('now')),
(5, 1, 'Ansible Playbooks', 'https://github.com/example/ansible-playbooks.git', 'master', 1, datetime('now'));

-- ============================================================================
-- Окружения (Environments)
-- ============================================================================
INSERT OR IGNORE INTO environment (id, project_id, name, json_data, created) VALUES
(1, 1, 'Production', '{"ansible_user": "ansible", "ansible_become": true, "environment": "production", "aws_region": "us-east-1"}', datetime('now')),
(2, 1, 'Staging', '{"ansible_user": "ansible", "ansible_become": true, "environment": "staging", "aws_region": "us-west-2"}', datetime('now')),
(3, 2, 'Web App Production', '{"app_env": "production", "db_host": "db.production.local", "cache_host": "cache.production.local"}', datetime('now')),
(4, 3, 'Database Production', '{"db_type": "postgresql", "db_port": "5432", "backup_enabled": true}', datetime('now')),
(5, 4, 'Security Audit', '{"scan_type": "full", "compliance_level": "strict", "report_format": "json"}', datetime('now'));

-- ============================================================================
-- Шаблоны (Templates)
-- ============================================================================
INSERT OR IGNORE INTO template (id, project_id, name, playbook, inventory_id, repository_id, environment_id, ssh_key_id, created) VALUES
(1, 1, 'Deploy Infrastructure', 'deploy.yml', 1, 1, 1, 1, datetime('now')),
(2, 1, 'Update Servers', 'update.yml', 1, 5, 1, 1, datetime('now')),
(3, 1, 'Backup Database', 'backup.yml', 1, 1, 1, 1, datetime('now')),
(4, 2, 'Deploy Web App', 'deploy-app.yml', 3, 2, 3, 3, datetime('now')),
(5, 2, 'Restart Application', 'restart.yml', 3, 2, 3, 3, datetime('now')),
(6, 2, 'Run Tests', 'test.yml', 3, 2, 3, 3, datetime('now')),
(7, 3, 'Migrate Database', 'migrate.yml', 4, 4, 4, 4, datetime('now')),
(8, 3, 'Backup PostgreSQL', 'pg-backup.yml', 4, 4, 4, 4, datetime('now')),
(9, 3, 'Vacuum Database', 'vacuum.yml', 4, 4, 4, 4, datetime('now')),
(10, 4, 'Security Scan', 'security-scan.yml', 5, 4, 5, 5, datetime('now')),
(11, 4, 'Compliance Check', 'compliance.yml', 5, 4, 5, 5, datetime('now')),
(12, 4, 'Audit Logs', 'audit-logs.yml', 5, 4, 5, 5, datetime('now'));

-- ============================================================================
-- Расписания (Schedules)
-- ============================================================================
INSERT OR IGNORE INTO schedule (id, template_id, cron_format, name, created) VALUES
(1, 3, '0 2 * * *', 'Daily Database Backup', datetime('now')),
(2, 8, '0 3 * * 0', 'Weekly PostgreSQL Backup', datetime('now')),
(3, 10, '0 0 * * 1', 'Weekly Security Scan', datetime('now')),
(4, 2, '0 4 * * 0', 'Weekly Server Update', datetime('now'));

-- ============================================================================
-- Задачи (Tasks) - последние запуски
-- ============================================================================
INSERT OR IGNORE INTO task (id, project_id, template_id, status, created, start, end) VALUES
(1, 1, 1, 'Success', datetime('now', '-1 day'), datetime('now', '-1 day', '+5 minutes'), datetime('now', '-1 day', '+15 minutes')),
(2, 1, 3, 'Success', datetime('now', '-12 hours'), datetime('now', '-12 hours', '+2 minutes'), datetime('now', '-12 hours', '+10 minutes')),
(3, 2, 4, 'Success', datetime('now', '-6 hours'), datetime('now', '-6 hours', '+3 minutes'), datetime('now', '-6 hours', '+8 minutes')),
(4, 3, 7, 'Running', datetime('now', '-1 hour'), datetime('now', '-1 hour'), NULL),
(5, 4, 10, 'Failed', datetime('now', '-2 hours'), datetime('now', '-2 hours', '+1 minute'), datetime('now', '-2 hours', '+5 minutes')),
(6, 1, 2, 'Success', datetime('now', '-30 minutes'), datetime('now', '-30 minutes', '+4 minutes'), datetime('now', '-30 minutes', '+12 minutes'));
