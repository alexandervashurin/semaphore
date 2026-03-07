#!/bin/bash

# ============================================================================
# Скрипт запуска демонстрационного окружения Semaphore UI
# ============================================================================
# Только frontend (Nginx) + PostgreSQL с демо-данными
# Backend запускается отдельно через cargo
#
# Использование: ./start.sh [OPTIONS]
#
# Опции:
#   --build, -b      Пересобрать образы
#   --clean, -c      Очистить volumes (удалить данные БД)
#   --stop, -s       Остановить сервисы
#   --restart, -r    Перезапустить сервисы
#   --logs, -l       Показать логи
#   --backend        Запустить backend через cargo
#   --help, -h       Показать эту справку
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Флаги
BUILD=false
CLEAN=false
STOP=false
RESTART=false
LOGS=false
BACKEND=false

# Функция для вывода справки
show_help() {
    head -22 "$0" | tail -19
    exit 0
}

# Функция проверки зависимостей
check_dependencies() {
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker не установлен. Установите Docker.${NC}"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        echo -e "${RED}❌ Docker Compose не установлен.${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Зависимости проверены${NC}"
}

# Функция определения команды docker-compose
get_compose_cmd() {
    if docker compose version &> /dev/null 2>&1; then
        echo "docker compose"
    else
        echo "docker-compose"
    fi
}

# Функция проверки статуса сервиса
wait_for_service() {
    local service_name="$1"
    local max_attempts="${2:-30}"
    local attempt=1

    echo -e "${YELLOW}⏳ Ожидание готовности $service_name...${NC}"

    while [ $attempt -le $max_attempts ]; do
        if docker exec semaphore-db pg_isready -U semaphore -d semaphore &> /dev/null 2>&1; then
            echo -e "${GREEN}✓ $service_name готов${NC}"
            return 0
        fi
        sleep 1
        ((attempt++))
    done

    echo -e "${RED}❌ $service_name не запустился за $max_attempts секунд${NC}"
    return 1
}

# Функция проверки наличия файлов frontend
check_frontend() {
    if [ ! -f "$SCRIPT_DIR/web/public/app.js" ] || [ ! -s "$SCRIPT_DIR/web/public/app.js" ]; then
        return 1
    fi
    return 0
}

# Функция сборки frontend
build_frontend() {
    echo -e "${YELLOW}📦 Frontend не собран. Запуск сборки...${NC}"

    if [ -f "$SCRIPT_DIR/web/build.sh" ]; then
        "$SCRIPT_DIR/web/build.sh"
    else
        echo -e "${RED}❌ Скрипт web/build.sh не найден${NC}"
        echo -e "${YELLOW}💡 Соберите frontend: cd web && ./build.sh${NC}"
        exit 1
    fi
}

# Парсинг аргументов
while [[ $# -gt 0 ]]; do
    case $1 in
        --build|-b)
            BUILD=true
            shift
            ;;
        --clean|-c)
            CLEAN=true
            shift
            ;;
        --stop|-s)
            STOP=true
            shift
            ;;
        --restart|-r)
            RESTART=true
            shift
            ;;
        --logs|-l)
            LOGS=true
            shift
            ;;
        --backend)
            BACKEND=true
            shift
            ;;
        --help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}❌ Неизвестный параметр: $1${NC}"
            echo "Используйте --help для справки"
            exit 1
            ;;
    esac
done

# Проверка зависимостей
check_dependencies

# Получение команды docker-compose
COMPOSE_CMD=$(get_compose_cmd)

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Semaphore UI - Demo (Frontend + PostgreSQL)        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Запуск backend через cargo
if [ "$BACKEND" = true ]; then
    echo -e "${YELLOW}🚀 Запуск backend (Rust)...${NC}"
    cd "$SCRIPT_DIR/rust"

    # Установка переменных окружения
    export SEMAPHORE_WEB_PATH="$SCRIPT_DIR/web/public"

    cargo run -- server --host 0.0.0.0 --port 3000 &
    BACKEND_PID=$!
    echo -e "${GREEN}✓ Backend запущен (PID: $BACKEND_PID)${NC}"
    echo ""
    echo -e "${YELLOW}ℹ️  Для остановки backend выполните:${NC}"
    echo -e "   ${CYAN}kill $BACKEND_PID${NC}"
    echo ""
    exit 0
fi

# Обработка команды остановки
if [ "$STOP" = true ]; then
    echo -e "${YELLOW}⏹️  Остановка сервисов...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" down
    echo -e "${GREEN}✓ Сервисы остановлены${NC}"
    exit 0
fi

# Обработка команды перезапуска
if [ "$RESTART" = true ]; then
    echo -e "${YELLOW}🔄 Перезапуск сервисов...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" restart
    echo -e "${GREEN}✓ Сервисы перезапущены${NC}"
    exit 0
fi

# Обработка команды просмотра логов
if [ "$LOGS" = true ]; then
    echo -e "${YELLOW}📋 Просмотр логов (Ctrl+C для выхода)...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" logs -f
    exit 0
fi

# Очистка volumes
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}🧹 Очистка volumes (удаление данных БД)...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" down -v
    echo -e "${GREEN}✓ Volumes очищены${NC}"
    echo ""
fi

# Проверка и сборка frontend
if ! check_frontend; then
    build_frontend
else
    echo -e "${GREEN}✓ Frontend уже собран${NC}"
fi

echo ""

# Пересборка образов
if [ "$BUILD" = true ]; then
    echo -e "${YELLOW}🔨 Пересборка Docker образов...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" build
else
    echo -e "${YELLOW}🔨 Проверка Docker образов...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" pull || echo -e "${YELLOW}⚠️  Не удалось загрузить образы, используем локальные${NC}"
fi

echo ""

# Запуск сервисов (используем --remove-orphans для надёжности)
echo -e "${GREEN}🚀 Запуск сервисов...${NC}"

# Останавливаем старые контейнеры если есть
$COMPOSE_CMD -f "$COMPOSE_FILE" down --remove-orphans 2>/dev/null || true

# Запускаем сервисы в фоновом режиме
$COMPOSE_CMD -f "$COMPOSE_FILE" up -d --remove-orphans

echo ""

# Ожидание готовности сервисов
wait_for_service "PostgreSQL" || {
    echo -e "${RED}❌ Ошибка запуска PostgreSQL${NC}"
    echo -e "${YELLOW}💡 Проверьте логи: docker logs semaphore-db${NC}"
    exit 1
}

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Semaphore UI Demo запущен!                     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Проверка статуса через docker ps (надёжнее чем docker-compose ps)
echo -e "${BLUE}📊 Статус сервисов:${NC}"
docker ps --filter "name=semaphore" --format "table {{.Names}}\t{{.Status}}" 2>/dev/null || \
    $COMPOSE_CMD -f "$COMPOSE_FILE" ps

echo ""
echo -e "${GREEN}✅ Frontend и БД готовы!${NC}"
echo ""
echo -e "${BLUE}📋 Информация:${NC}"
echo -e "   🌐 Frontend: ${GREEN}http://localhost${NC}"
echo -e "   💾 PostgreSQL: ${GREEN}localhost:5432${NC}"
echo -e "   🔧 Backend (запустите отдельно): ${YELLOW}./start.sh --backend${NC}"
echo ""
echo -e "${YELLOW}💡 Полезные команды:${NC}"
echo -e "   ${CYAN}./start.sh --backend${NC}    - Запуск backend"
echo -e "   ${CYAN}./start.sh --logs${NC}       - Просмотр логов"
echo -e "   ${CYAN}./start.sh --stop${NC}       - Остановка сервисов"
echo -e "   ${CYAN}./start.sh --restart${NC}    - Перезапуск сервисов"
echo -e "   ${CYAN}./start.sh --clean${NC}      - Очистка данных (БД)"
echo -e "   ${CYAN}./cleanup.sh --all${NC}      - Полная очистка (контейнеры, volumes, сети)"
echo -e "   ${CYAN}docker logs semaphore-db${NC}    - Лог БД"
echo -e "   ${CYAN}docker logs semaphore-frontend${NC} - Лог frontend"
echo ""
