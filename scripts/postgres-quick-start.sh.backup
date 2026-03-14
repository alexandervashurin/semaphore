#!/bin/bash
# Скрипт быстрого запуска PostgreSQL для тестирования Semaphore

set -e

CONTAINER_NAME="semaphore_postgres"
DB_USER="semaphore"
DB_PASS="semaphore_pass"
DB_NAME="semaphore"
DB_PORT="5433"

echo "=== PostgreSQL Quick Start for Semaphore ==="
echo

# Проверка Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker не найден. Установите Docker."
    exit 1
fi

echo "✓ Docker найден"

# Проверка наличия образа
if ! docker image inspect postgres:16-alpine &> /dev/null; then
    echo "⏳ Загрузка образа postgres:16-alpine..."
    docker pull postgres:16-alpine
fi

echo "✓ Образ postgres:16-alpine доступен"

# Проверка запущенного контейнера
if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "✓ Контейнер уже запущен"
else
    echo "🚀 Запуск контейнера..."
    docker run -d \
        --name ${CONTAINER_NAME} \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASS} \
        -e POSTGRES_DB=${DB_NAME} \
        -p ${DB_PORT}:5432 \
        -v $(pwd)/db/postgres/init.sql:/docker-entrypoint-initdb.d/init.sql:ro \
        -v postgres_data:/var/lib/postgresql/data \
        postgres:16-alpine
    
    echo "⏳ Ожидание готовности PostgreSQL..."
    sleep 5
fi

# Проверка подключения
echo
echo "=== Проверка подключения ==="
docker exec ${CONTAINER_NAME} psql -U ${DB_USER} -d ${DB_NAME} -c "SELECT 'PostgreSQL ready!' as status;"

echo
echo "=== Таблицы ==="
docker exec ${CONTAINER_NAME} psql -U ${DB_USER} -d ${DB_NAME} -c "\dt"

echo
echo "=== Готово! ==="
echo
echo "Connection string для Semaphore:"
echo "  postgres://${DB_USER}:${DB_PASS}@localhost:${DB_PORT}/${DB_NAME}"
echo
echo "Для запуска Semaphore выполните:"
echo "  export SEMAPHORE_DB_URL=\"postgres://${DB_USER}:${DB_PASS}@localhost:${DB_PORT}/${DB_NAME}\""
echo "  cargo run --manifest-path=rust/Cargo.toml -- server"
echo
echo "Или в одной команде:"
echo "  SEMAPHORE_DB_URL=\"postgres://${DB_USER}:${DB_PASS}@localhost:${DB_PORT}/${DB_NAME}\" cargo run --manifest-path=rust/Cargo.toml -- server"
echo
echo "Для остановки: docker stop ${CONTAINER_NAME}"
echo "Для удаления: docker rm -f ${CONTAINER_NAME}"
