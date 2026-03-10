#!/bin/bash
# Инициализация демо-БД для Semaphore UI

DB_PATH="/tmp/semaphore.db"
PASSWORD_HASH='$2b$12$pDKL.XOgDcQCXBm77saF4eO/84j.Ul1zDhnYPUM61vkqZAUnz9vwS'
NOW=$(date -Iseconds)

echo "📊 Инициализация демо-БД Semaphore UI"
echo "======================================"
echo ""

# Создание пользователей
echo "👥 Создание пользователей..."
sqlite3 "$DB_PATH" << SQL
INSERT INTO user (username, name, email, password, admin, alert, external, pro, created) VALUES
('admin', 'Admin User', 'admin@localhost', '$PASSWORD_HASH', 1, 0, 0, 0, '$NOW'),
('john.doe', 'John Doe', 'john.doe@localhost', '$PASSWORD_HASH', 0, 0, 0, 0, '$NOW'),
('jane.smith', 'Jane Smith', 'jane.smith@localhost', '$PASSWORD_HASH', 0, 0, 0, 0, '$NOW'),
('devops', 'DevOps User', 'devops@localhost', '$PASSWORD_HASH', 0, 0, 0, 0, '$NOW');
SQL
if [ $? -eq 0 ]; then echo "  ✅ 4 пользователя созданы"; else echo "  ❌ Ошибка"; fi

# Создание проектов
echo "📁 Создание проектов..."
sqlite3 "$DB_PATH" << SQL
INSERT INTO project (name, created, alert, max_parallel_tasks, type) VALUES
('Web Application', '$NOW', 0, 0, 'git'),
('Database Migration', '$NOW', 0, 0, 'git'),
('Infrastructure', '$NOW', 0, 0, 'git'),
('CI/CD Pipeline', '$NOW', 0, 0, 'git'),
('Monitoring Setup', '$NOW', 0, 0, 'git');
SQL
if [ $? -eq 0 ]; then echo "  ✅ 5 проектов создано"; else echo "  ❌ Ошибка"; fi

# Связи пользователей с проектами
echo "🔗 Создание связей..."
sqlite3 "$DB_PATH" << SQL
INSERT INTO project__user (project_id, user_id, role) VALUES
(1, 1, 'owner'), (1, 2, 'manager'), (1, 3, 'manager'),
(2, 1, 'owner'), (2, 4, 'manager'),
(3, 1, 'owner'), (3, 2, 'manager'), (3, 4, 'manager'),
(4, 1, 'owner'), (4, 3, 'manager'),
(5, 1, 'owner'), (5, 2, 'manager'), (5, 3, 'manager'), (5, 4, 'manager');
SQL
if [ $? -eq 0 ]; then echo "  ✅ Связи созданы"; else echo "  ❌ Ошибка"; fi

echo ""
echo "📋 Результат:"
echo "------------"
echo "Пользователи:"
sqlite3 -header -column "$DB_PATH" "SELECT id, username, name, admin FROM user;"
echo ""
echo "Проекты:"
sqlite3 -header -column "$DB_PATH" "SELECT id, name FROM project;"
echo ""
echo "Связи:"
sqlite3 -header -column "$DB_PATH" "SELECT project_id, user_id, role FROM project__user LIMIT 5;"
echo ""
echo "✅ Демо-БД готова!"
echo ""
echo "🔐 Учётные данные (пароль: demo123):"
echo "   - admin (Администратор)"
echo "   - john.doe (Менеджер)"
echo "   - jane.smith (Менеджер)"
echo "   - devops (Исполнитель)"
echo ""
echo "📍 Откройте http://localhost:3000"
