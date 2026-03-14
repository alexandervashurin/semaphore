#!/bin/bash
# Запуск Semaphore UI с SQLite

set -e

echo "🚀 Запуск Semaphore UI с SQLite..."

# Переменные окружения
export SEMAPHORE_DB_DIALECT=sqlite
export SEMAPHORE_DB_PATH="${SEMAPHORE_DB_PATH:-/var/lib/semaphore/semaphore.db}"
export SEMAPHORE_WEB_PATH="${SEMAPHORE_WEB_PATH:-./web/public}"

# Создание директории для БД если не существует
DB_DIR=$(dirname "$SEMAPHORE_DB_PATH")
if [ ! -d "$DB_DIR" ]; then
    echo "📁 Создание директории для БД: $DB_DIR"
    mkdir -p "$DB_DIR"
fi

echo "📊 База данных: $SEMAPHORE_DB_PATH"
echo "🌐 Web путь: $SEMAPHORE_WEB_PATH"
echo ""

# Запуск сервера
cd "$(dirname "$0")/../rust"
cargo run -- server
