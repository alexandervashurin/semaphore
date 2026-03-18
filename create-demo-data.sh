#!/bin/bash
# Скрипт создания демо-данных для Velum

DB_PATH="/tmp/semaphore.db"
PASSWORD_HASH='$2b$12$pDKL.XOgDcQCXBm77saF4eO/84j.Ul1zDhnYPUM61vkqZAUnz9vwS'

echo "📊 Создание демо-данных для Velum..."
echo "============================================"

if [ ! -f "$DB_PATH" ]; then
    echo "❌ База данных не найдена: $DB_PATH"
    exit 1
fi

echo "👥 Создание пользователей..."

sqlite3 "$DB_PATH" << SQL
-- Дополнительные пользователи (admin уже есть)
INSERT OR IGNORE INTO user (id, username, name, email, password, admin, created) VALUES
(2, 'john.doe', 'John Doe', 'john.doe@localhost', '$PASSWORD_HASH', 0, datetime('now')),
(3, 'jane.smith', 'Jane Smith', 'jane.smith@localhost', '$PASSWORD_HASH', 0, datetime('now')),
(4, 'devops', 'DevOps User', 'devops@localhost', '$PASSWORD_HASH', 0, datetime('now'));

-- Проекты
INSERT OR IGNORE INTO project (id, name, created, updated) VALUES
(1, 'Web Application', datetime('now'), datetime('now')),
(2, 'Database Migration', datetime('now'), datetime('now')),
(3, 'Infrastructure', datetime('now'), datetime('now')),
(4, 'CI/CD Pipeline', datetime('now'), datetime('now')),
(5, 'Monitoring Setup', datetime('now'), datetime('now'));

-- Связи пользователей с проектами
INSERT OR IGNORE INTO project_user (project_id, user_id, admin) VALUES
(1, 1, 1), (1, 2, 0), (1, 3, 0),
(2, 1, 1), (2, 4, 0),
(3, 1, 1), (3, 2, 0), (3, 4, 0),
(4, 1, 1), (4, 3, 0),
(5, 1, 1), (5, 2, 0), (5, 3, 0), (5, 4, 0);
SQL

echo "✅ Пользователи и проекты созданы"
echo ""
echo "📋 Результат:"
echo "-------------"
echo "Пользователи:"
sqlite3 -header -column "$DB_PATH" "SELECT id, username, name, admin FROM user;"
echo ""
echo "Проекты:"
sqlite3 -header -column "$DB_PATH" "SELECT id, name FROM project;"
echo ""
echo "✅ Демо-данные созданы успешно!"
echo ""
echo "🔐 Учётные данные (пароль: demo123):"
echo "   - admin (Администратор)"
echo "   - john.doe (Менеджер)"
echo "   - jane.smith (Менеджер)"
echo "   - devops (Исполнитель)"
