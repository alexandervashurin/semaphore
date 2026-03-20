#!/bin/bash
# =============================================================================
# Скрипт применения миграции БД для Playbook API
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MIGRATION_FILE="${SCRIPT_DIR}/db/postgres/migrations/003_full_schema_update.sql"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║      Миграция БД для Playbook API                        ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Проверка файла миграции
if [ ! -f "${MIGRATION_FILE}" ]; then
    echo "[ERROR] Файл миграции не найден: ${MIGRATION_FILE}"
    exit 1
fi

echo "[INFO] Файл миграции: ${MIGRATION_FILE}"
echo ""

# Проверка Docker
if ! command -v docker &> /dev/null; then
    echo "[ERROR] Docker не найден"
    exit 1
fi

# Проверка контейнера PostgreSQL
if ! docker ps --format '{{.Names}}' | grep -q "semaphore-db"; then
    echo "[ERROR] Контейнер PostgreSQL не найден. Запустите: ./start-server.sh start"
    exit 1
fi

echo "[INFO] Применение миграции..."
echo ""

# Применение миграции
docker exec -i semaphore-db psql -U semaphore -d semaphore < "${MIGRATION_FILE}"

echo ""
echo "[OK] Миграция применена успешно!"
echo ""
echo "[INFO] Рекомендуется перезапустить сервер:"
echo "       ./start-server.sh restart"
echo ""
