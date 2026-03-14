#!/bin/bash
# Запуск Semaphore UI с PostgreSQL

set -e

echo "🚀 Запуск Semaphore UI с PostgreSQL..."

# Переменные окружения
export SEMAPHORE_DB_DIALECT=postgres
export SEMAPHORE_DB_HOST="${SEMAPHORE_DB_HOST:-localhost}"
export SEMAPHORE_DB_PORT="${SEMAPHORE_DB_PORT:-5432}"
export SEMAPHORE_DB_USER="${SEMAPHORE_DB_USER:-semaphore}"
export SEMAPHORE_DB_PASS="${SEMAPHORE_DB_PASS:-semaphore_pass}"
export SEMAPHORE_DB_NAME="${SEMAPHORE_DB_NAME:-semaphore}"
export SEMAPHORE_WEB_PATH="${SEMAPHORE_WEB_PATH:-./web/public}"

# Формирование URL базы данных
export SEMAPHORE_DB_URL="postgres://${SEMAPHORE_DB_USER}:${SEMAPHORE_DB_PASS}@${SEMAPHORE_DB_HOST}:${SEMAPHORE_DB_PORT}/${SEMAPHORE_DB_NAME}"

echo "📊 Хост: $SEMAPHORE_DB_HOST:$SEMAPHORE_DB_PORT"
echo "📊 База данных: $SEMAPHORE_DB_NAME"
echo "👤 Пользователь: $SEMAPHORE_DB_USER"
echo "🌐 Web путь: $SEMAPHORE_WEB_PATH"
echo ""

# Запуск сервера
cd "$(dirname "$0")/../rust"
cargo run -- server
