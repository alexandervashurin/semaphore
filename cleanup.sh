#!/bin/bash

# ============================================================================
# Скрипт остановки и удаления контейнеров Semaphore UI
# ============================================================================
# Останавливает и удаляет Docker контейнеры, volumes, сети и образы
#
# Использование: ./cleanup.sh [OPTIONS]
#
# Опции:
#   --all, -a          Удалить всё (контейнеры, volumes, сети, образы)
#   --volumes, -v      Удалить volumes (данные БД)
#   --networks, -n     Удалить сети
#   --images, -i       Удалить образы
#   --dry-run          Показать, что будет удалено (без удаления)
#   --force, -f        Не запрашивать подтверждение
#   --help, -h         Показать эту справку
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
REMOVE_ALL=false
REMOVE_VOLUMES=false
REMOVE_NETWORKS=false
REMOVE_IMAGES=false
DRY_RUN=false
FORCE=false

# Функция для вывода справки
show_help() {
    head -22 "$0" | tail -19
    exit 0
}

# Функция для запроса подтверждения
confirm() {
    if [ "$FORCE" = true ]; then
        return 0
    fi
    
    local prompt="$1"
    echo -e "${YELLOW}⚠️  $prompt${NC}"
    read -p "Продолжить? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Отменено пользователем${NC}"
        exit 0
    fi
}

# Парсинг аргументов
while [[ $# -gt 0 ]]; do
    case $1 in
        --all|-a)
            REMOVE_ALL=true
            shift
            ;;
        --volumes|-v)
            REMOVE_VOLUMES=true
            shift
            ;;
        --networks|-n)
            REMOVE_NETWORKS=true
            shift
            ;;
        --images|-i)
            REMOVE_IMAGES=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --force|-f)
            FORCE=true
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

# Проверка наличия Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker не установлен. Установите Docker.${NC}"
    exit 1
fi

# Определение команды docker-compose
if docker compose version &> /dev/null 2>&1; then
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     Semaphore UI - Очистка Docker ресурсов             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Если не указано ни одного режима, показываем справку
if [ "$REMOVE_ALL" = false ] && [ "$REMOVE_VOLUMES" = false ] && \
   [ "$REMOVE_NETWORKS" = false ] && [ "$REMOVE_IMAGES" = false ]; then
    echo -e "${YELLOW}ℹ️  Не указан режим очистки. Используйте:${NC}"
    echo -e "   ${CYAN}--all, -a${NC}       - Удалить всё (контейнеры, volumes, сети, образы)"
    echo -e "   ${CYAN}--volumes, -v${NC}   - Удалить volumes (данные БД)"
    echo -e "   ${CYAN}--networks, -n${NC}  - Удалить сети"
    echo -e "   ${CYAN}--images, -i${NC}    - Удалить образы"
    echo ""
    echo -e "${YELLOW}💡 Примеры:${NC}"
    echo -e "   ${CYAN}./cleanup.sh --all${NC}           - Полная очистка"
    echo -e "   ${CYAN}./cleanup.sh --volumes${NC}       - Удалить только volumes"
    echo -e "   ${CYAN}./cleanup.sh --all --force${NC}   - Полная очистка без подтверждения"
    echo ""
    exit 0
fi

# Режим dry-run: показываем, что будет удалено
if [ "$DRY_RUN" = true ]; then
    echo -e "${CYAN}📋 РЕЖИМ ПРОВЕРКИ (ничего не удаляется)${NC}"
    echo ""
fi

# Остановка контейнеров
echo -e "${YELLOW}⏹️  Остановка Docker контейнеров...${NC}"
if [ "$DRY_RUN" = true ]; then
    echo "   [DRY-RUN] $COMPOSE_CMD -f \"$COMPOSE_FILE\" down"
else
    $COMPOSE_CMD -f "$COMPOSE_FILE" down 2>/dev/null || true
fi
echo -e "${GREEN}✓ Контейнеры остановлены${NC}"
echo ""

# Удаление volumes
if [ "$REMOVE_VOLUMES" = true ] || [ "$REMOVE_ALL" = true ]; then
    if [ "$DRY_RUN" = true ]; then
        echo -e "${CYAN}📋 Будут удалены volumes:${NC}"
        echo "   [DRY-RUN] $COMPOSE_CMD -f \"$COMPOSE_FILE\" down -v"
        docker volume ls --filter name=semaphore 2>/dev/null || true
    else
        echo -e "${YELLOW}🗑️  Удаление volumes (данные БД будут потеряны)...${NC}"
        confirm "Вы уверены, что хотите удалить volumes? Все данные БД будут потеряны!"
        $COMPOSE_CMD -f "$COMPOSE_FILE" down -v 2>/dev/null || true
        echo -e "${GREEN}✓ Volumes удалены${NC}"
    fi
    echo ""
fi

# Удаление сетей
if [ "$REMOVE_NETWORKS" = true ] || [ "$REMOVE_ALL" = true ]; then
    if [ "$DRY_RUN" = true ]; then
        echo -e "${CYAN}📋 Будут удалены сети:${NC}"
        docker network ls --filter name=semaphore 2>/dev/null || true
    else
        echo -e "${YELLOW}🗑️  Удаление сетей...${NC}"
        # Удаление через docker-compose
        $COMPOSE_CMD -f "$COMPOSE_FILE" down --remove-orphans 2>/dev/null || true
        # Принудительное удаление оставшихся сетей
        for network in $(docker network ls --filter name=semaphore --format "{{.Name}}" 2>/dev/null); do
            docker network rm "$network" 2>/dev/null || true
        done
        echo -e "${GREEN}✓ Сети удалены${NC}"
    fi
    echo ""
fi

# Удаление образов
if [ "$REMOVE_IMAGES" = true ] || [ "$REMOVE_ALL" = true ]; then
    if [ "$DRY_RUN" = true ]; then
        echo -e "${CYAN}📋 Будут удалены образы:${NC}"
        docker images --format "{{.Repository}}:{{.Tag}}" | grep -E "(postgres|nginx|semaphore)" || true
    else
        echo -e "${YELLOW}🗑️  Удаление Docker образов...${NC}"
        confirm "Вы уверены, что хотите удалить образы? При следующем запуске они будут загружены заново."
        
        # Удаляем контейнеры, связанные с образами
        docker rm -f semaphore-db 2>/dev/null || true
        docker rm -f semaphore-frontend 2>/dev/null || true
        docker rm -f semaphore_postgres 2>/dev/null || true
        
        # Удаляем образы
        docker rmi -f postgres:15-alpine 2>/dev/null || true
        docker rmi -f postgres:16-alpine 2>/dev/null || true
        docker rmi -f nginx:alpine 2>/dev/null || true
        
        echo -e "${GREEN}✓ Образы удалены${NC}"
    fi
    echo ""
fi

# Итоговый отчет
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Очистка завершена!                        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Показываем оставшиеся ресурсы
if [ "$DRY_RUN" = false ]; then
    echo -e "${CYAN}📊 Текущее состояние Docker ресурсов:${NC}"
    echo ""
    
    # Контейнеры
    echo "Контейнеры:"
    docker ps -a --filter name=semaphore --format "  {{.Names}} - {{.Status}}" 2>/dev/null || echo "  Нет контейнеров semaphore"
    echo ""
    
    # Volumes
    echo "Volumes:"
    docker volume ls --filter name=semaphore --format "  {{.Name}}" 2>/dev/null || echo "  Нет volumes semaphore"
    echo ""
    
    # Сети
    echo "Сети:"
    docker network ls --filter name=semaphore --format "  {{.Name}}" 2>/dev/null || echo "  Нет сетей semaphore"
    echo ""
fi

echo -e "${GREEN}✅ Очистка завершена!${NC}"
echo ""
echo -e "${YELLOW}💡 Для повторного запуска выполните:${NC}"
echo -e "   ${CYAN}./start.sh${NC}"
echo ""
