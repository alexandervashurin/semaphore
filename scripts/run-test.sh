#!/bin/bash
# Запуск Velum с тестовой SQLite БД (для разработки)

set -e

echo "🚀 Запуск Velum с тестовой SQLite БД..."

# Переменные окружения
export SEMAPHORE_DB_DIALECT=sqlite
export SEMAPHORE_DB_PATH="/tmp/semaphore_test.db"
export SEMAPHORE_WEB_PATH="${SEMAPHORE_WEB_PATH:-./web/public}"

echo "📊 База данных: $SEMAPHORE_DB_PATH"
echo "🌐 Web путь: $SEMAPHORE_WEB_PATH"
echo "⚠️  Внимание: Данные будут удалены после перезагрузки!"
echo ""

# Запуск сервера
cd "$(dirname "$0")/../rust"
cargo run -- server
