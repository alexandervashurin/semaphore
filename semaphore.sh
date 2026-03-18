#!/bin/bash
# ============================================================================
# Velum - Универсальный скрипт управления
# ============================================================================
# Все команды управления в одном скрипте
#
# Использование: ./semaphore.sh <КОМАНДА> [ОПЦИИ]
#
# Команды:
#   start [РЕЖИМ]   Запуск (native|hybrid|docker)
#   stop            Остановка сервисов
#   restart         Перезапуск
#   clean           Очистка данных
#   init            Инициализация БД
#   status          Показать статус
#   logs            Показать логи
#   build           Сборка проекта
#   help            Показать справку
#
# Режимы запуска:
#   native    Чистый запуск: SQLite + Backend + Frontend на хосте
#   hybrid    Гибрид: PostgreSQL в Docker + остальное на хосте (рекомендуется)
#   docker    Всё в Docker: PostgreSQL + Backend + Frontend
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
COMPOSE_FULL_FILE="$SCRIPT_DIR/docker-compose.full.yml"
COMPOSE_POSTGRES_FILE="$SCRIPT_DIR/docker-compose.postgres.yml"
ENV_FILE="$SCRIPT_DIR/.env"
LOG_DIR="$SCRIPT_DIR/logs"

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
    echo -e "${CYAN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC} $1"
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

build_frontend() {
    step "Сборка frontend..."
    cd "$SCRIPT_DIR/web"
    if [ -f "build.sh" ]; then
        ./build.sh
    elif [ -f "package.json" ]; then
        npm install && npm run build
    else
        error "Скрипт сборки frontend не найден"
    fi
    success "Frontend собран"
}

build_backend() {
    step "Сборка backend..."
    cd "$SCRIPT_DIR/rust"
    cargo build --release
    success "Backend собран"
}

# ============================================================================
# Переменные окружения
# ============================================================================

setup_env_native() {
    step "Настройка переменных окружения (native)..."
    unset SEMAPHORE_DB_URL SEMAPHORE_DB_HOST SEMAPHORE_DB_PORT
    export SEMAPHORE_DB_DIALECT=sqlite
    export SEMAPHORE_DB_PATH="${SEMAPHORE_DB_PATH:-$SCRIPT_DIR/data/semaphore.db}"
    export SEMAPHORE_WEB_PATH="$SCRIPT_DIR/web/public"
    export SEMAPHORE_TMP_PATH="/tmp/semaphore"
    export SEMAPHORE_TCP_ADDRESS="0.0.0.0:3000"
    export RUST_LOG="${RUST_LOG:-info}"
    mkdir -p "$(dirname "$SEMAPHORE_DB_PATH")" "$LOG_DIR"
    cat > "$ENV_FILE" <<EOF
# Velum - Native Mode (SQLite)
SEMAPHORE_DB_DIALECT=sqlite
SEMAPHORE_DB_PATH=$SEMAPHORE_DB_PATH
SEMAPHORE_WEB_PATH=$SEMAPHORE_WEB_PATH
SEMAPHORE_TMP_PATH=$SEMAPHORE_TMP_PATH
SEMAPHORE_TCP_ADDRESS=$SEMAPHORE_TCP_ADDRESS
RUST_LOG=$RUST_LOG
EOF
    success "Переменные окружения установлены"
    info "  DB: SQLite ($SEMAPHORE_DB_PATH)"
    info "  Web: $SEMAPHORE_WEB_PATH"
    info "  Port: 3000"
}

setup_env_hybrid() {
    step "Настройка переменных окружения (hybrid)..."
    unset SEMAPHORE_DB_PATH
    export SEMAPHORE_DB_DIALECT=postgres
    export SEMAPHORE_DB_HOST="localhost"
    export SEMAPHORE_DB_PORT="5432"
    export SEMAPHORE_DB_USER="semaphore"
    export SEMAPHORE_DB_PASSWORD="semaphore_pass"
    export SEMAPHORE_DB_NAME="semaphore"
    export SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@localhost:5432/semaphore"
    export SEMAPHORE_WEB_PATH="$SCRIPT_DIR/web/public"
    export SEMAPHORE_TMP_PATH="/tmp/semaphore"
    export SEMAPHORE_TCP_ADDRESS="0.0.0.0:3000"
    export RUST_LOG="${RUST_LOG:-info}"
    mkdir -p "$LOG_DIR"
    cat > "$ENV_FILE" <<EOF
# Velum - Hybrid Mode (PostgreSQL in Docker)
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=$SEMAPHORE_DB_URL
SEMAPHORE_WEB_PATH=$SEMAPHORE_WEB_PATH
SEMAPHORE_TMP_PATH=$SEMAPHORE_TMP_PATH
SEMAPHORE_TCP_ADDRESS=$SEMAPHORE_TCP_ADDRESS
RUST_LOG=$RUST_LOG
EOF
    success "Переменные окружения установлены"
    info "  DB: PostgreSQL (localhost:5432)"
    info "  Web: $SEMAPHORE_WEB_PATH"
    info "  Port: 3000"
}

setup_env_docker() {
    step "Настройка переменных окружения (docker)..."
    cat > "$ENV_FILE" <<EOF
# Velum - Docker Mode
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@db:5432/semaphore
SEMAPHORE_WEB_PATH=/app/web/public
SEMAPHORE_TMP_PATH=/tmp/semaphore
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
EOF
    success "Переменные окружения установлены"
}

# ============================================================================
# PostgreSQL функции
# ============================================================================

start_postgres_docker() {
    step "Запуск PostgreSQL в Docker..."
    docker rm -f semaphore-db 2>/dev/null || true
    docker run -d \
        --name semaphore-db \
        -e POSTGRES_DB=semaphore \
        -e POSTGRES_USER=semaphore \
        -e POSTGRES_PASSWORD=semaphore_pass \
        -p 5432:5432 \
        -v semaphore_postgres_data:/var/lib/postgresql/data \
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
        if docker exec semaphore-db pg_isready -U semaphore -d semaphore &> /dev/null 2>&1; then
            success "PostgreSQL готов"
            return 0
        fi
        sleep 1
        ((attempt++))
    done
    error "PostgreSQL не запустился за $max_attempts секунд"
}

# ============================================================================
# Native режим
# ============================================================================

cmd_init_native() {
    setup_env_native
    check_rust
    step "Инициализация SQLite БД..."
    cd "$SCRIPT_DIR/rust"
    cargo run --release -- migrate --upgrade
    success "Миграции применены"
    step "Создание пользователя admin..."
    cargo run --release -- user add \
        --username admin \
        --name "Administrator" \
        --email admin@localhost \
        --password admin123 \
        --admin
    success "Пользователь admin создан"
    echo ""
    info "Теперь запустите сервер: $0 start native"
}

cmd_start_native() {
    setup_env_native
    check_rust
    if ! check_frontend; then
        warning "Frontend не найден"
        check_node && build_frontend || warning "Запуск без frontend (только API)"
    else
        success "Frontend найден"
    fi
    step "Запуск backend..."
    cd "$SCRIPT_DIR/rust"
    pkill -f "semaphore server" 2>/dev/null || true
    sleep 1
    nohup cargo run --release -- server --host 0.0.0.0 --port 3000 > "$LOG_DIR/backend.log" 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > "$LOG_DIR/backend.pid"
    sleep 3
    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        success "Backend запущен (PID: $BACKEND_PID)"
    else
        error "Backend не запустился. Проверьте логи: $LOG_DIR/backend.log"
    fi
    print_status_native
}

cmd_stop_native() {
    step "Остановка backend..."
    pkill -f "semaphore server" 2>/dev/null || true
    rm -f "$LOG_DIR/backend.pid"
    success "Backend остановлен"
}

cmd_clean_native() {
    step "Очистка данных SQLite..."
    if [ -n "$SEMAPHORE_DB_PATH" ] && [ -f "$SEMAPHORE_DB_PATH" ]; then
        rm -f "$SEMAPHORE_DB_PATH"
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

print_status_native() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         Velum запущен! (Native Mode)            ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}🌐 Web-интерфейс:${NC} http://localhost:3000"
    echo -e "${GREEN}💾 База данных:${NC} $SEMAPHORE_DB_PATH"
    echo ""
    echo -e "${YELLOW}Учётные данные:${NC}"
    echo -e "   admin / admin123"
    echo ""
    echo -e "${YELLOW}Полезные команды:${NC}"
    echo -e "   ${CYAN}$0 stop${NC}   - Остановить backend"
    echo -e "   ${CYAN}$0 logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}$0 init${NC}   - Инициализировать БД"
    echo -e "   ${CYAN}$0 clean${NC}  - Удалить БД"
    echo ""
}

# ============================================================================
# Hybrid режим
# ============================================================================

cmd_init_hybrid() {
    setup_env_hybrid
    check_docker
    check_rust
    if ! docker ps --format '{{.Names}}' | grep -q semaphore-db; then
        start_postgres_docker
    fi
    step "Инициализация PostgreSQL БД..."
    cd "$SCRIPT_DIR/rust"
    cargo run --release -- migrate --upgrade
    success "Миграции применены"
    step "Создание пользователя admin..."
    cargo run --release -- user add \
        --username admin \
        --name "Administrator" \
        --email admin@localhost \
        --password admin123 \
        --admin
    success "Пользователь admin создан"
    echo ""
    info "Теперь запустите сервер: $0 start hybrid"
}

cmd_start_hybrid() {
    setup_env_hybrid
    check_docker
    check_rust
    if ! check_frontend; then
        warning "Frontend не найден"
        check_node && build_frontend || warning "Запуск без frontend (только API)"
    else
        success "Frontend найден"
    fi
    if ! docker ps --format '{{.Names}}' | grep -q semaphore-db; then
        start_postgres_docker
    fi
    step "Запуск backend..."
    cd "$SCRIPT_DIR/rust"
    pkill -f "semaphore server" 2>/dev/null || true
    sleep 1
    nohup cargo run --release -- server --host 0.0.0.0 --port 3000 > "$LOG_DIR/backend.log" 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > "$LOG_DIR/backend.pid"
    sleep 3
    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        success "Backend запущен (PID: $BACKEND_PID)"
    else
        error "Backend не запустился. Проверьте логи: $LOG_DIR/backend.log"
    fi
    print_status_hybrid
}

cmd_stop_hybrid() {
    step "Остановка backend и PostgreSQL..."
    pkill -f "semaphore server" 2>/dev/null || true
    docker stop semaphore-db 2>/dev/null || true
    rm -f "$LOG_DIR/backend.pid"
    success "Сервисы остановлены"
}

cmd_restart_hybrid() {
    step "Перезапуск сервисов..."
    docker restart semaphore-db 2>/dev/null || true
    pkill -f "semaphore server" 2>/dev/null || true
    sleep 2
    cmd_start_hybrid
    success "Сервисы перезапущены"
}

cmd_clean_hybrid() {
    step "Очистка volumes PostgreSQL..."
    docker volume rm semaphore_postgres_data 2>/dev/null || true
    success "Данные PostgreSQL удалены"
}

cmd_logs_hybrid() {
    echo "=== Backend Logs ==="
    if [ -f "$LOG_DIR/backend.log" ]; then
        tail -f "$LOG_DIR/backend.log"
    else
        docker logs -f semaphore-db 2>&1
    fi
}

print_status_hybrid() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         Velum запущен! (Hybrid Mode)            ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}🌐 Web-интерфейс:${NC} http://localhost:3000"
    echo -e "${GREEN}💾 База данных:${NC} PostgreSQL (localhost:5432)"
    echo ""
    echo -e "${YELLOW}Учётные данные (демо):${NC}"
    echo -e "   admin / demo123"
    echo ""
    echo -e "${YELLOW}Полезные команды:${NC}"
    echo -e "   ${CYAN}$0 stop${NC}   - Остановить сервисы"
    echo -e "   ${CYAN}$0 logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}$0 init${NC}   - Инициализировать БД"
    echo -e "   ${CYAN}$0 clean${NC}  - Удалить данные БД"
    echo -e "   ${CYAN}docker logs semaphore-db${NC}   - Лог PostgreSQL"
    echo ""
}

# ============================================================================
# Docker режим
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
# Общие команды
# ============================================================================

cmd_status() {
    header "Статус Velum"
    
    echo "Контейнеры:"
    docker ps -a --filter name=semaphore --format "  {{.Names}} - {{.Status}}" 2>/dev/null || echo "  Нет контейнеров semaphore"
    echo ""
    
    echo "Volumes:"
    docker volume ls --filter name=semaphore --format "  {{.Name}}" 2>/dev/null || echo "  Нет volumes semaphore"
    echo ""
    
    echo "Backend:"
    if pgrep -f "semaphore server" > /dev/null; then
        echo "  ✓ Запущен (PID: $(pgrep -f 'semaphore server'))"
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
    header "Velum - Справка"
    cat <<EOF
Использование: $0 <КОМАНДА> [ОПЦИИ]

Команды:
  start [РЕЖИМ]   Запуск сервиса
                  Режимы: native (SQLite), hybrid (PostgreSQL в Docker), docker
  stop            Остановка сервисов
  restart         Перезапуск сервисов
  clean           Очистка данных
  init            Инициализация БД (создание админа)
  status          Показать статус сервисов
  logs            Показать логи
  build           Сборка проекта
  help            Показать эту справку

Примеры:
  $0 start native      - Запуск с SQLite
  $0 start hybrid      - Запуск с PostgreSQL в Docker (рекомендуется)
  $0 start docker      - Запуск всех сервисов в Docker
  $0 stop              - Остановить сервисы
  $0 clean             - Очистить данные
  $0 init              - Инициализировать БД
  $0 status            - Показать статус
  $0 logs              - Показать логи

Документация:
  README.md            - Основная документация
  CONFIG.md            - Конфигурация
  API.md               - API документация
  ЗАПУСК_ДЕМО.md       - Запуск демо-режима

EOF
}

# ============================================================================
# Основная функция
# ============================================================================

main() {
    mkdir -p "$LOG_DIR"
    
    local COMMAND="${1:-help}"
    shift 2>/dev/null || true
    
    case "$COMMAND" in
        start)
            local MODE="${1:-hybrid}"
            shift 2>/dev/null || true
            case "$MODE" in
                native) cmd_start_native "$@" ;;
                hybrid) cmd_start_hybrid "$@" ;;
                docker) cmd_start_docker "$@" ;;
                *) error "Неизвестный режим: $MODE. Доступные: native, hybrid, docker" ;;
            esac
            ;;
        stop)
            # Определяем что запущено и останавливаем
            if docker ps --format '{{.Names}}' | grep -q semaphore-db; then
                cmd_stop_hybrid
            elif pgrep -f "semaphore server" > /dev/null; then
                cmd_stop_native
            else
                cmd_stop_docker 2>/dev/null || info "Сервисы не запущены"
            fi
            ;;
        restart)
            if docker ps --format '{{.Names}}' | grep -q semaphore-db; then
                cmd_restart_hybrid
            else
                cmd_restart_docker
            fi
            ;;
        clean)
            # Определяем что нужно чистить
            if docker volume ls --format '{{.Name}}' | grep -q semaphore_postgres_data; then
                cmd_clean_hybrid
            elif [ -f "${SEMAPHORE_DB_PATH:-$SCRIPT_DIR/data/semaphore.db}" ]; then
                cmd_clean_native
            else
                cmd_clean_docker
            fi
            ;;
        init)
            local MODE="${1:-hybrid}"
            case "$MODE" in
                native) cmd_init_native ;;
                hybrid) cmd_init_hybrid ;;
                *) error "Инициализация поддерживается только для native и hybrid" ;;
            esac
            ;;
        status)
            cmd_status
            ;;
        logs)
            if docker ps --format '{{.Names}}' | grep -q semaphore-db; then
                cmd_logs_hybrid
            else
                cmd_logs_native
            fi
            ;;
        build)
            cmd_build
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            error "Неизвестная команда: $COMMAND. Используйте '$0 help' для справки."
            ;;
    esac
}

main "$@"
