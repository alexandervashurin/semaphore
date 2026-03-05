#!/bin/bash

# Скрипт остановки Semaphore UI
# Использование: ./stop.sh [--clean]
#
# Опции:
#   --clean, -c    Очистить volumes (удалить данные БД)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

# Определение команды docker-compose
if docker compose version &> /dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

if [ "$1" = "--clean" ] || [ "$1" = "-c" ]; then
    echo "⏹️  Остановка сервисов и очистка volumes..."
    $COMPOSE_CMD -f "$COMPOSE_FILE" down -v
    echo "✓ Сервисы остановлены, volumes очищены"
else
    echo "⏹️  Остановка сервисов..."
    $COMPOSE_CMD -f "$COMPOSE_FILE" down
    echo "✓ Сервисы остановлены"
fi
