#!/bin/bash
# ==============================================================================
# Скрипт быстрого запуска Semaphore с демонстрационными данными
# ==============================================================================
# Этот скрипт:
#   1. Запускает PostgreSQL с демонстрационными данными
#   2. Создаёт .env файл для Semaphore
#   3. Экспортирует переменные окружения
#   4. Запускает Semaphore сервер
#
# Использование:
#   ./scripts/postgres-demo-start.sh          # Запуск с demo-данными
#   ./scripts/postgres-demo-start.sh --clean  # Запуск с полной очисткой
#   ./scripts/postgres-demo-start.sh --cleanup # Только очистка (остановка и удаление)
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.postgres.yml"
RUST_DIR="$PROJECT_ROOT/rust"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}==============================================================================${NC}"
echo -e "${BLUE}Semaphore - Запуск демонстрационного окружения${NC}"
echo -e "${BLUE}==============================================================================${NC}"
echo ""

# Проверка наличия Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Ошибка: Docker не найден. Пожалуйста, установите Docker.${NC}"
    exit 1
fi

# Проверка наличия docker-compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}Ошибка: docker-compose не найден. Пожалуйста, установите docker-compose.${NC}"
    exit 1
fi

# Функция для определения команды docker-compose
get_compose_cmd() {
    if command -v docker-compose &> /dev/null; then
        echo "docker-compose"
    else
        echo "docker compose"
    fi
}

COMPOSE_CMD=$(get_compose_cmd)

# Режим только очистки (cleanup)
if [ "$1" == "--cleanup" ]; then
    echo -e "${YELLOW}[1/3] Остановка контейнера semaphore_postgres...${NC}"
    if docker ps -q -f name=semaphore_postgres 2>/dev/null | grep -q .; then
        docker stop semaphore_postgres
        echo -e "${GREEN}✓ Контейнер остановлен${NC}"
    else
        echo -e "${YELLOW}⚠ Контейнер semaphore_postgres не запущен${NC}"
    fi

    echo -e "${YELLOW}[2/3] Удаление контейнера...${NC}"
    if docker ps -aq -f name=semaphore_postgres 2>/dev/null | grep -q .; then
        docker rm semaphore_postgres
        echo -e "${GREEN}✓ Контейнер удален${NC}"
    else
        echo -e "${YELLOW}⚠ Контейнер semaphore_postgres не существует${NC}"
    fi

    echo -e "${YELLOW}[3/3] Удаление volume с данными...${NC}"
    if docker volume ls -q -f name=semaphore_postgres_data 2>/dev/null | grep -q .; then
        docker volume rm semaphore_postgres_data
        echo -e "${GREEN}✓ Volume удален${NC}"
    else
        echo -e "${YELLOW}⚠ Volume semaphore_postgres_data не существует${NC}"
    fi

    echo ""
    echo -e "${GREEN}==============================================================================${NC}"
    echo -e "${GREEN}Очистка завершена!${NC}"
    echo -e "${GREEN}==============================================================================${NC}"
    echo ""
    echo -e "${BLUE}Для запуска нового экземпляра выполните:${NC}"
    echo -e "  ${GREEN}./scripts/postgres-demo-start.sh${NC}"
    echo ""
    exit 0
fi

# Остановка существующих контейнеров
echo -e "${YELLOW}[1/6] Остановка существующих контейнеров...${NC}"
$COMPOSE_CMD -f "$COMPOSE_FILE" down 2>/dev/null || true

# Очистка старых данных (если требуется)
if [ "$1" == "--clean" ] || [ "$1" == "-c" ]; then
    echo -e "${YELLOW}[2/6] Очистка старых данных...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" down -v
fi

# Запуск PostgreSQL
echo -e "${YELLOW}[2/6] Запуск PostgreSQL с демонстрационными данными...${NC}"
$COMPOSE_CMD -f "$COMPOSE_FILE" up -d

# Ожидание готовности PostgreSQL
echo -e "${YELLOW}[3/6] Ожидание готовности PostgreSQL...${NC}"
MAX_ATTEMPTS=30
ATTEMPT=0
while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if docker exec semaphore_postgres pg_isready -U semaphore -d semaphore &> /dev/null; then
        echo -e "${GREEN}✓ PostgreSQL готов!${NC}"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    sleep 1
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo -e "${RED}Ошибка: PostgreSQL не запустился вовремя${NC}"
    docker logs semaphore_postgres
    exit 1
fi

# Проверка наличия демонстрационных данных
echo -e "${YELLOW}[4/6] Проверка демонстрационных данных...${NC}"
USER_COUNT=$(docker exec semaphore_postgres psql -U semaphore -d semaphore -t -c "SELECT COUNT(*) FROM \"user\";" 2>/dev/null | tr -d ' ')
PROJECT_COUNT=$(docker exec semaphore_postgres psql -U semaphore -d semaphore -t -c "SELECT COUNT(*) FROM project;" 2>/dev/null | tr -d ' ')
TEMPLATE_COUNT=$(docker exec semaphore_postgres psql -U semaphore -d semaphore -t -c "SELECT COUNT(*) FROM template;" 2>/dev/null | tr -d ' ')

if [ "$USER_COUNT" -ge 1 ] && [ "$PROJECT_COUNT" -ge 1 ] && [ "$TEMPLATE_COUNT" -ge 1 ]; then
    echo -e "${GREEN}✓ Демонстрационные данные загружены!${NC}"
    echo -e "  - Пользователей: ${GREEN}$USER_COUNT${NC}"
    echo -e "  - Проектов: ${GREEN}$PROJECT_COUNT${NC}"
    echo -e "  - Шаблонов: ${GREEN}$TEMPLATE_COUNT${NC}"
else
    echo -e "${RED}Ошибка: Демонстрационные данные не найдены${NC}"
    exit 1
fi

# Создание .env файла
echo -e "${YELLOW}[5/6] Создание .env файла...${NC}"
cat > "$PROJECT_ROOT/.env" << EOF
# Semaphore Demo Configuration
# Auto-generated by postgres-demo-start.sh

# Database (PostgreSQL)
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_HOST=localhost
SEMAPHORE_DB_PORT=5433
SEMAPHORE_DB_USER=semaphore
SEMAPHORE_DB_PASS=semaphore_pass
SEMAPHORE_DB_NAME=semaphore

# HTTP Server
SEMAPHORE_HTTP_PORT=3000
SEMAPHORE_WEB_HOST=http://localhost:3000

# Admin (уже создан в БД)
# SEMAPHORE_ADMIN=admin
# SEMAPHORE_ADMIN_PASSWORD=changeme

# Logging
RUST_LOG=info

# Max Parallel Tasks
SEMAPHORE_MAX_PARALLEL_TASKS=10
EOF

echo -e "${GREEN}✓ .env файл создан${NC}"

# Экспорт переменных окружения и запуск Semaphore
echo -e "${YELLOW}[6/6] Запуск Semaphore сервера...${NC}"
echo -e "${BLUE}Переменные окружения:${NC}"
echo -e "  SEMAPHORE_DB_DIALECT=postgres"
echo -e "  SEMAPHORE_DB_HOST=localhost"
echo -e "  SEMAPHORE_DB_PORT=5433"
echo -e "  SEMAPHORE_DB_USER=semaphore"
echo -e "  SEMAPHORE_DB_PASS=semaphore_pass"
echo -e "  SEMAPHORE_DB_NAME=semaphore"
echo -e "  SEMAPHORE_HTTP_PORT=3000"
echo -e "  SEMAPHORE_WEB_HOST=http://localhost:3000"
echo -e "  RUST_LOG=info"
echo ""

# Переход в директорию rust и запуск сервера
cd "$RUST_DIR"

# Экспорт переменных для текущей сессии
export SEMAPHORE_DB_DIALECT=postgres
export SEMAPHORE_DB_HOST=localhost
export SEMAPHORE_DB_PORT=5433
export SEMAPHORE_DB_USER=semaphore
export SEMAPHORE_DB_PASS=semaphore_pass
export SEMAPHORE_DB_NAME=semaphore
export SEMAPHORE_HTTP_PORT=3000
export SEMAPHORE_WEB_HOST=http://localhost:3000
export RUST_LOG=info
export SEMAPHORE_MAX_PARALLEL_TASKS=10

echo -e "${GREEN}==============================================================================${NC}"
echo -e "${GREEN}Готово! Запуск Semaphore...${NC}"
echo -e "${GREEN}==============================================================================${NC}"
echo ""
echo -e "${BLUE}Доступ к Semaphore:${NC}"
echo -e "  URL: ${GREEN}http://localhost:3000${NC}"
echo ""
echo -e "${BLUE}Учетные данные (пароль для всех: ${GREEN}demo123${BLUE}):${NC}"
echo -e "  - ${GREEN}admin${NC} (Administrator) - полный доступ"
echo -e "  - ${GREEN}john.doe${NC} (John Doe) - менеджер Web Application"
echo -e "  - ${GREEN}jane.smith${NC} (Jane Smith) - менеджер Database Management"
echo -e "  - ${GREEN}devops${NC} (DevOps Engineer) - исполнитель задач"
echo ""
echo -e "${BLUE}Проекты:${NC}"
echo -e "  1. Demo Infrastructure"
echo -e "  2. Web Application Deployment"
echo -e "  3. Database Management"
echo -e "  4. Security & Compliance"
echo ""
echo -e "${BLUE}Полезные команды (в другом терминале):${NC}"
echo -e "  - Остановить PostgreSQL: ${GREEN}docker-compose -f docker-compose.postgres.yml down${NC}"
echo -e "  - Очистить и перезапустить: ${GREEN}./scripts/postgres-demo-start.sh --clean${NC}"
echo -e "  - Только очистка (остановка и удаление): ${GREEN}./scripts/postgres-demo-start.sh --cleanup${NC}"
echo -e "  - Логи PostgreSQL: ${GREEN}docker logs semaphore_postgres${NC}"
echo -e "  - Подключиться к БД: ${GREEN}docker exec -it semaphore_postgres psql -U semaphore -d semaphore${NC}"
echo -e "  - Отдельный скрипт очистки: ${GREEN}./scripts/postgres-cleanup.sh${NC}"
echo ""
echo -e "${YELLOW}Нажмите Ctrl+C для остановки сервера${NC}"
echo ""

# Запуск сервера Semaphore
exec cargo run -- server
