#!/bin/bash
# ============================================================================
# Velum - Универсальный скрипт запуска и управления
# ============================================================================
# Все команды управления в одном скрипте
#
# Использование: ./velum.sh <КОМАНДА> [ОПЦИИ]
#
# Команды:
#   start [РЕЖИМ]   Запуск (dev|docker)
#   stop            Остановка сервисов
#   restart         Перезапуск
#   clean           Очистка данных
#   init            Инициализация БД (создание админа)
#   status          Показать статус
#   logs            Показать логи
#   build           Сборка проекта
#   demo            Запуск с демо-данными
#   help            Показать справку
#
# Режимы запуска:
#   dev       PostgreSQL в Docker + Backend на хосте (рекомендуется для разработки)
#   docker    Всё в Docker: PostgreSQL + Backend + Frontend (продакшен)
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
COMPOSE_FULL_FILE="$SCRIPT_DIR/docker-compose.full.yml"
COMPOSE_POSTGRES_FILE="$SCRIPT_DIR/docker-compose.postgres.yml"
ENV_FILE="$SCRIPT_DIR/.env"
LOG_DIR="$SCRIPT_DIR/logs"
DATA_DIR="$SCRIPT_DIR/data"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# ============================================================================
# Функции вывода
# ============================================================================

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
step() { echo -e "${CYAN}➜${NC} $1"; }

header() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC} $1"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

banner() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}              🚀 Velum - Rust Edition               ${NC}"
    echo -e "${CYAN}║${NC}      Стать лучше AWX и Ansible Tower               ${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# ============================================================================
# Проверка зависимостей
# ============================================================================

check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker не найден. Установите Docker: https://docs.docker.com/get-docker/"
    fi
    if command -v docker-compose &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    elif docker compose version &> /dev/null 2>&1; then
        COMPOSE_CMD="docker compose"
    else
        error "Docker Compose не найден"
    fi
    success "Docker и Docker Compose найдены ($COMPOSE_CMD)"
}

check_rust() {
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo не найдены. Установите Rust: https://rustup.rs/"
    fi
    success "Rust найден ($(cargo --version))"
}

check_node() {
    if ! command -v node &> /dev/null; then
        warning "Node.js не найден. Frontend не будет собран автоматически."
        return 1
    fi
    success "Node.js найден ($(node --version))"
    return 0
}

check_frontend() {
    [ -f "$SCRIPT_DIR/web/public/index.html" ] || [ -f "$SCRIPT_DIR/web/public/app.js" ]
}

check_curl() {
    if ! command -v curl &> /dev/null; then
        warning "curl не найден. Некоторые функции могут быть недоступны."
        return 1
    fi
    return 0
}

check_jq() {
    if ! command -v jq &> /dev/null; then
        warning "jq не найден. Работа с JSON будет ограничена."
        return 1
    fi
    return 0
}

build_frontend() {
    step "Сборка frontend..."
    cd "$SCRIPT_DIR/web"
    if [ -f "build.sh" ]; then
        ./build.sh
    elif [ -f "package.json" ]; then
        npm install && npm run build
    else
        warning "Скрипт сборки frontend не найден, пропускаем"
        return 0
    fi
    success "Frontend собран"
}

build_backend() {
    step "Сборка backend..."
    cd "$SCRIPT_DIR/rust"
    cargo build --release
    success "Backend собран ($(ls -lh target/release/velum | awk '{print $5}'))"
}

# ============================================================================
# Переменные окружения
# ============================================================================

setup_env_dev() {
    step "Настройка переменных окружения (dev)..."
    mkdir -p "$LOG_DIR"

    export VELUM_DB_DIALECT=postgres
    export VELUM_DB_HOST="localhost"
    export VELUM_DB_PORT="5432"
    export VELUM_DB_USER="velum"
    export VELUM_DB_PASSWORD="velum_pass"
    export VELUM_DB_NAME="velum"
    export VELUM_DB_URL="postgres://velum:velum_pass@localhost:5432/velum"
    export VELUM_WEB_PATH="$SCRIPT_DIR/web/public"
    export VELUM_TMP_PATH="/tmp/velum"
    export VELUM_TCP_ADDRESS="0.0.0.0:3000"
    export RUST_LOG="${RUST_LOG:-info}"

    cat > "$ENV_FILE" <<EOF
# Velum - Dev Mode (PostgreSQL in Docker)
VELUM_DB_DIALECT=postgres
VELUM_DB_URL=$VELUM_DB_URL
VELUM_WEB_PATH=$VELUM_WEB_PATH
VELUM_TMP_PATH=$VELUM_TMP_PATH
VELUM_TCP_ADDRESS=$VELUM_TCP_ADDRESS
RUST_LOG=$RUST_LOG
EOF
    success "Переменные окружения установлены"
    info "  DB: PostgreSQL (localhost:5432)"
    info "  Web: $VELUM_WEB_PATH"
    info "  Port: 3000"
}

setup_env_docker() {
    step "Настройка переменных окружения (docker)..."
    mkdir -p "$LOG_DIR"
    
    cat > "$ENV_FILE" <<EOF
# Velum - Docker Mode
VELUM_DB_DIALECT=postgres
VELUM_DB_URL=postgres://velum:velum_pass@db:5432/velum
VELUM_WEB_PATH=/app/web/public
VELUM_TMP_PATH=/tmp/velum
VELUM_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
EOF
    success "Переменные окружения установлены"
}

# ============================================================================
# PostgreSQL функции
# ============================================================================

start_postgres_docker() {
    step "Запуск PostgreSQL в Docker..."
    docker rm -f velum-db 2>/dev/null || true
    docker run -d \
        --name velum-db \
        -e POSTGRES_DB=velum \
        -e POSTGRES_USER=velum \
        -e POSTGRES_PASSWORD=velum_pass \
        -p 5432:5432 \
        -v velum_postgres_data:/var/lib/postgresql/data \
        --restart unless-stopped \
        postgres:15-alpine
    wait_for_postgres
    success "PostgreSQL запущен"
}

start_postgres_demo() {
    step "Запуск PostgreSQL с демо-данными..."
    cd "$SCRIPT_DIR"
    $COMPOSE_CMD -f "$COMPOSE_POSTGRES_FILE" up -d
    wait_for_postgres
    success "PostgreSQL с демо-данными запущен"
}

wait_for_postgres() {
    step "Ожидание готовности PostgreSQL..."
    local max_attempts=30
    local attempt=1
    while [ $attempt -le $max_attempts ]; do
        if docker exec velum-db pg_isready -U velum -d velum &> /dev/null 2>&1; then
            success "PostgreSQL готов"
            return 0
        fi
        sleep 1
        ((attempt++))
    done
    error "PostgreSQL не запустился за $max_attempts секунд"
}

wait_for_server() {
    step "Ожидание готовности сервера..."
    local max_attempts=30
    local attempt=1
    while [ $attempt -le $max_attempts ]; do
        if curl -s http://localhost:3000/api/health > /dev/null 2>&1; then
            success "Сервер готов"
            return 0
        fi
        sleep 1
        ((attempt++))
    done
    warning "Сервер не ответил за $max_attempts секунд (возможно ещё запускается)"
}

# ============================================================================
# Native режим (SQLite) - УСТАРЕЛ, НЕ ИСПОЛЬЗУЕТСЯ
# ============================================================================
# Эти функции оставлены для обратной совместимости, но выводят предупреждение

cmd_init_native() {
    warning "Режим SQLite устарел и больше не поддерживается. Используйте dev (PostgreSQL)."
    cmd_init_dev
}

cmd_start_native() {
    warning "Режим SQLite устарел и больше не поддерживается. Используйте dev (PostgreSQL)."
    cmd_start_dev
}

cmd_stop_native() {
    step "Остановка backend..."
    pkill -f "velum server" 2>/dev/null || true
    rm -f "$LOG_DIR/backend.pid"
    success "Backend остановлен"
}

cmd_clean_native() {
    step "Очистка данных SQLite..."
    if [ -n "$VELUM_DB_PATH" ] && [ -f "$VELUM_DB_PATH" ]; then
        rm -f "$VELUM_DB_PATH"
        success "SQLite БД удалена"
    else
        info "SQLite БД не найдена"
    fi
}

cmd_logs_native() {
    if [ -f "$LOG_DIR/backend.log" ]; then
        tail -f "$LOG_DIR/backend.log"
    else
        info "Лог файл не найден"
    fi
}

# ============================================================================
# Dev режим (PostgreSQL в Docker)
# ============================================================================

cmd_init_dev() {
    setup_env_dev
    check_docker
    check_rust

    if ! docker ps --format '{{.Names}}' | grep -q velum-db; then
        start_postgres_docker
    fi

    step "Инициализация PostgreSQL БД..."
    cd "$SCRIPT_DIR/rust"
    cargo run --release --bin velum-cli -- migrate --upgrade
    success "Миграции применены"

    step "Создание пользователя admin..."
    cargo run --release --bin velum-cli -- user add \
        --username admin \
        --name "Administrator" \
        --email admin@localhost \
        --password admin123 \
        --admin
    success "Пользователь admin создан"
    echo ""
    info "Теперь запустите сервер: $0 start dev"
}

cmd_start_dev() {
    setup_env_dev
    check_docker
    check_rust

    if ! check_frontend; then
        warning "Frontend не найден"
        check_node && build_frontend || warning "Запуск без frontend (только API)"
    else
        success "Frontend найден"
    fi

    if ! docker ps --format '{{.Names}}' | grep -q velum-db; then
        start_postgres_docker
    fi

    step "Запуск backend..."
    cd "$SCRIPT_DIR/rust"
    pkill -f "velum server" 2>/dev/null || true
    sleep 1

    nohup cargo run --release --bin velum -- server --host 0.0.0.0 --port 3000 > "$LOG_DIR/backend.log" 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > "$LOG_DIR/backend.pid"

    sleep 3

    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        success "Backend запущен (PID: $BACKEND_PID)"
        wait_for_server
        print_status_dev
    else
        error "Backend не запустился. Проверьте логи: $LOG_DIR/backend.log"
    fi
}

cmd_stop_dev() {
    step "Остановка backend и PostgreSQL..."
    pkill -f "velum server" 2>/dev/null || true
    docker stop velum-db 2>/dev/null || true
    rm -f "$LOG_DIR/backend.pid"
    success "Сервисы остановлены"
}

cmd_restart_dev() {
    step "Перезапуск сервисов..."
    docker restart velum-db 2>/dev/null || true
    pkill -f "velum server" 2>/dev/null || true
    sleep 2
    cmd_start_dev
    success "Сервисы перезапущены"
}

cmd_clean_dev() {
    step "Очистка volumes PostgreSQL..."
    docker volume rm velum_postgres_data 2>/dev/null || true
    success "Данные PostgreSQL удалены"
}

cmd_logs_dev() {
    echo "=== Backend Logs ==="
    if [ -f "$LOG_DIR/backend.log" ]; then
        tail -f "$LOG_DIR/backend.log"
    else
        docker logs -f velum-db 2>&1
    fi
}

print_status_dev() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         Velum запущен! (Dev Mode)               ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}🌐 Web-интерфейс:${NC} http://localhost:3000"
    echo -e "${GREEN}💾 База данных:${NC} PostgreSQL (localhost:5432)"
    echo ""
    echo -e "${YELLOW}Учётные данные:${NC}"
    echo -e "   admin / admin123"
    echo ""
    echo -e "${YELLOW}Полезные команды:${NC}"
    echo -e "   ${CYAN}$0 stop${NC}   - Остановить сервисы"
    echo -e "   ${CYAN}$0 logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}$0 init${NC}   - Инициализировать БД"
    echo -e "   ${CYAN}$0 clean${NC}  - Удалить данные БД"
    echo -e "   ${CYAN}docker logs velum-db${NC}   - Лог PostgreSQL"
    echo ""
}
# ============================================================================
# Docker режим (всё в Docker)
# ============================================================================

cmd_start_docker() {
    setup_env_docker
    check_docker
    
    if ! check_frontend; then
        check_node && build_frontend || warning "Frontend не найден, используем заглушку"
    else
        success "Frontend найден"
    fi
    
    step "Запуск всех сервисов в Docker..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" down --remove-orphans 2>/dev/null || true
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" up -d --remove-orphans --build
    wait_for_postgres
    sleep 5
    success "Все сервисы запущены"
    print_status_docker
}

cmd_stop_docker() {
    step "Остановка Docker сервисов..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" down
    success "Сервисы остановлены"
}

cmd_restart_docker() {
    step "Перезапуск Docker сервисов..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" restart
    success "Сервисы перезапущены"
}

cmd_clean_docker() {
    step "Очистка Docker volumes..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" down -v
    success "Volumes очищены"
}

cmd_logs_docker() {
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" logs -f
}

print_status_docker() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         Velum запущен! (Docker Mode)            ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}🌐 Web-интерфейс:${NC} http://localhost"
    echo -e "${GREEN}💾 База данных:${NC} PostgreSQL (в Docker)"
    echo ""
    echo -e "${YELLOW}Учётные данные (демо):${NC}"
    echo -e "   admin / demo123"
    echo ""
    echo -e "${YELLOW}Полезные команды:${NC}"
    echo -e "   ${CYAN}$0 stop${NC}   - Остановить сервисы"
    echo -e "   ${CYAN}$0 logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}$0 clean${NC}  - Удалить данные"
    echo ""
}

# ============================================================================
# Демо режим (с наполнением данными)
# ============================================================================

cmd_demo() {
    header "Запуск Velum с демо-данными"
    
    local MODE="${1:-hybrid}"
    
    case "$MODE" in
        native)
            cmd_start_native
            sleep 5
            step "Наполнение демо-данными..."
            if [ -f "$SCRIPT_DIR/fill-sqlite-demo-data.sh" ]; then
                bash "$SCRIPT_DIR/fill-sqlite-demo-data.sh"
            else
                warning "Скрипт наполнения не найден"
            fi
            ;;
        hybrid)
            cmd_start_hybrid
            sleep 5
            step "Наполнение демо-данными..."
            if [ -f "$SCRIPT_DIR/fill-postgres-demo-data.sh" ]; then
                bash "$SCRIPT_DIR/fill-postgres-demo-data.sh"
            else
                warning "Скрипт наполнения не найден"
            fi
            ;;
        *)
            error "Неподдерживаемый режим: $MODE"
            ;;
    esac
}

# ============================================================================
# Общие команды
# ============================================================================

cmd_status() {
    header "Статус Velum"

    echo "Контейнеры:"
    docker ps -a --filter name=velum --format "  {{.Names}} - {{.Status}}" 2>/dev/null || echo "  Нет контейнеров velum"
    echo ""

    echo "Volumes:"
    docker volume ls --filter name=velum --format "  {{.Name}}" 2>/dev/null || echo "  Нет volumes velum"
    echo ""

    echo "Backend:"
    if pgrep -f "velum server" > /dev/null; then
        echo "  ✓ Запущен (PID: $(pgrep -f 'velum server'))"
    else
        echo "  ✗ Остановлен"
    fi
    echo ""

    echo "Доступ:"
    echo "  http://localhost:3000"
    echo ""
}

cmd_build() {
    header "Сборка проекта"
    check_rust
    build_backend
    if check_node; then
        build_frontend
    fi
    success "Сборка завершена"
}

cmd_help() {
    banner
    cat <<EOF
Использование: \$0 <КОМАНДА> [ОПЦИИ]

Команды:
  start [РЕЖИМ]   Запуск сервиса
                  Режимы: dev (PostgreSQL в Docker), docker
  stop            Остановка сервисов
  restart         Перезапуск сервисов
  clean           Очистка данных
  init            Инициализация БД (создание админа)
  status          Показать статус сервисов
  logs            Показать логи
  build           Сборка проекта
  demo [РЕЖИМ]    Запуск с демо-данными
  help            Показать эту справку

Примеры:
  \$0 start dev         - Запуск с PostgreSQL в Docker (рекомендуется)
  \$0 start docker      - Запуск всех сервисов в Docker
  \$0 stop              - Остановить сервисы
  \$0 clean             - Очистить данные
  \$0 init              - Инициализировать БД
  \$0 status            - Показать статус
  \$0 logs              - Показать логи
  \$0 demo              - Запуск с демо-данными

Документация:
  README.md            - Основная документация
  CONFIG.md            - Конфигурация
  API.md               - API документация
  MASTER_PLAN_V3.md    - План развития проекта

EOF
}

# ============================================================================
# Основная функция
# ============================================================================

main() {
    mkdir -p "$LOG_DIR" "$DATA_DIR"

    local COMMAND="${1:-help}"
    shift 2>/dev/null || true

    case "$COMMAND" in
        start)
            local MODE="${1:-dev}"
            shift 2>/dev/null || true
            case "$MODE" in
                dev) cmd_start_dev "$@" ;;
                docker) cmd_start_docker "$@" ;;
                native|hybrid) warning "Режим '$MODE' устарел. Используйте 'dev' (PostgreSQL)." ; cmd_start_dev "$@" ;;
                *) error "Неизвестный режим: $MODE. Доступные: dev, docker" ;;
            esac
            ;;
        stop)
            if docker ps --format '{{.Names}}' | grep -q velum-db; then
                cmd_stop_dev
            elif pgrep -f "velum server" > /dev/null; then
                cmd_stop_native
            else
                cmd_stop_docker 2>/dev/null || info "Сервисы не запущены"
            fi
            ;;
        restart)
            if docker ps --format '{{.Names}}' | grep -q velum-db; then
                cmd_restart_dev
            else
                cmd_restart_docker
            fi
            ;;
        clean)
            if docker volume ls --format '{{.Name}}' | grep -q velum_postgres_data; then
                cmd_clean_dev
            elif [ -f "${VELUM_DB_PATH:-$DATA_DIR/velum.db}" ]; then
                cmd_clean_native
            else
                cmd_clean_docker 2>/dev/null || info "Сервисы не запущены"
            fi
            ;;
        init)
            local MODE="${1:-dev}"
            case "$MODE" in
                dev) cmd_init_dev ;;
                native|hybrid) warning "Режим '$MODE' устарел. Используйте 'dev' (PostgreSQL)." ; cmd_init_dev ;;
                *) error "Инициализация поддерживается только для dev" ;;
            esac
            ;;
        status)
            cmd_status
            ;;
        logs)
            if docker ps --format '{{.Names}}' | grep -q velum-db; then
                cmd_logs_dev
            else
                cmd_logs_native
            fi
            ;;
        build)
            cmd_build
            ;;
        demo)
            local MODE="${1:-dev}"
            cmd_demo "$MODE"
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            error "Неизвестная команда: $COMMAND. Используйте '\$0 help' для справки."
            ;;
    esac
}

main "$@"
