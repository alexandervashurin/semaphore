#!/bin/bash

# Скрипт сборки frontend Velum через Docker
# Не требует установки Node.js/npm на хосте

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🐳 Сборка frontend Velum (Docker)"
echo "=========================================="

# Проверка наличия Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker не установлен. Установите Docker."
    exit 1
fi

# Проверка наличия docker-compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose не установлен. Установите Docker Compose."
    exit 1
fi

cd "$SCRIPT_DIR"

# Определение команды docker-compose
if docker compose version &> /dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

echo "📦 Очистка старых файлов сборки..."
rm -rf "$SCRIPT_DIR/public/app.js" \
       "$SCRIPT_DIR/public/app.css" \
       "$SCRIPT_DIR/public/js" \
       "$SCRIPT_DIR/public/css" 2>/dev/null || true

echo "🔨 Запуск сборки в Docker контейнере..."
$COMPOSE_CMD -f docker-compose.build.yml run --rm frontend-build-clean

echo ""
echo "✓ Сборка завершена!"
echo "📦 Файлы фронтенда: $SCRIPT_DIR/public/"
echo ""
echo "Проверка файлов:"
ls -lh "$SCRIPT_DIR/public/" | grep -E '\.(js|css|html)$'
