#!/bin/bash

# ============================================================================
# Скрипт запуска Semaphore UI (полный стек через Docker)
# ============================================================================
# Использование: ./start.sh [OPTIONS]
#
# Опции:
#   --build, -b      Пересобрать образы перед запуском
#   --clean, -c      Очистить volumes (удалить данные БД)
#   --stop, -s       Остановить сервисы
#   --restart, -r    Перезапустить сервисы
#   --logs, -l       Показать логи
#   --help, -h       Показать эту справку
#
# Примеры:
#   ./start.sh                    # Запуск сервиса
#   ./start.sh --build            # Запуск с пересборкой образов
#   ./start.sh --clean --build    # Полный сброс и запуск
#   ./start.sh --stop             # Остановка сервисов
#   ./start.sh --logs             # Просмотр логов
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Флаги
BUILD=false
CLEAN=false
STOP=false
RESTART=false
LOGS=false

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
        --help|-h)
            head -20 "$0" | tail -17
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Неизвестный параметр: $1${NC}"
            echo "Используйте --help для справки"
            exit 1
            ;;
    esac
done

# Проверка наличия Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker не установлен. Установите Docker.${NC}"
    exit 1
fi

# Проверка наличия docker-compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose не установлен.${NC}"
    exit 1
fi

# Определение команды docker-compose
if docker compose version &> /dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Semaphore UI - Запуск через Docker              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

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

# Сборка frontend (если нужно)
if [ ! -f "$SCRIPT_DIR/web/public/app.js" ] || [ ! -s "$SCRIPT_DIR/web/public/app.js" ]; then
    echo -e "${YELLOW}📦 Frontend не собран. Запуск сборки...${NC}"
    
    if [ -f "$SCRIPT_DIR/web/build.sh" ]; then
        "$SCRIPT_DIR/web/build.sh"
    else
        echo -e "${RED}❌ Скрипт web/build.sh не найден${NC}"
        echo -e "${YELLOW}💡 Соберите frontend вручную: cd web && npm install && npm run build${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ Frontend уже собран${NC}"
fi

echo ""

# Пересборка образов
if [ "$BUILD" = true ]; then
    echo -e "${YELLOW}🔨 Пересборка Docker образов...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" build --no-cache
else
    echo -e "${YELLOW}🔨 Сборка Docker образов...${NC}"
    $COMPOSE_CMD -f "$COMPOSE_FILE" build
fi

echo ""

# Запуск сервисов
echo -e "${GREEN}🚀 Запуск сервисов...${NC}"
$COMPOSE_CMD -f "$COMPOSE_FILE" up -d

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Semaphore UI запущен!                     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Ожидание готовности сервисов
echo -e "${YELLOW}⏳ Ожидание готовности сервисов...${NC}"
sleep 10

# Проверка статуса
echo ""
echo -e "${BLUE}📊 Статус сервисов:${NC}"
$COMPOSE_CMD -f "$COMPOSE_FILE" ps

echo ""
echo -e "${GREEN}✅ Semaphore UI готов к работе!${NC}"
echo ""
echo -e "${BLUE}📋 Информация:${NC}"
echo -e "   🌐 URL: ${GREEN}http://localhost:3000${NC}"
echo -e "   👤 Логин: ${GREEN}admin${NC}"
echo -e "   🔑 Пароль: ${GREEN}admin123${NC}"
echo ""
echo -e "${YELLOW}💡 Демо-пользователи:${NC}"
echo -e "   - john.doe / demo123"
echo -e "   - jane.smith / demo123"
echo -e "   - devops / demo123"
echo ""
echo -e "${YELLOW}📚 Полезные команды:${NC}"
echo -e "   ./start.sh --logs       # Просмотр логов"
echo -e "   ./start.sh --stop       # Остановка сервисов"
echo -e "   ./start.sh --restart    # Перезапуск сервисов"
echo -e "   ./start.sh --clean      # Очистка данных (БД)"
echo -e "   docker-compose ps       # Статус сервисов"
echo ""
