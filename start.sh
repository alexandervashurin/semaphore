#!/bin/bash
# ============================================================================
# Velum - Универсальный скрипт запуска
# ============================================================================
# Поддерживаемые режимы:
#   1. native   - Чистый запуск: SQLite + Backend + Frontend на хосте
#   2. hybrid   - Гибрид: PostgreSQL в Docker + Backend + Frontend на хосте
#   3. docker   - Всё в Docker: PostgreSQL + Backend + Frontend
# ============================================================================
# Использование: ./start.sh [РЕЖИМ] [ОПЦИИ]
#
# Режимы:
#   native    Чистый запуск на хосте (SQLite, минимальные зависимости)
#   hybrid    Гибрид: БД в Docker, остальное на хосте (рекомендуется)
#   docker    Всё в Docker (продакшен)
#
# Опции:
#   --stop          Остановить все сервисы
#   --restart       Перезапустить сервисы
#   --clean         Очистить данные/volumes
#   --logs          Показать логи
#   --build         Пересобрать образы/бинарник
#   --init          Инициализировать БД (создать админа)
#   --help, -h      Показать эту справку
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
COMPOSE_FULL_FILE="$SCRIPT_DIR/docker-compose.full.yml"
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

# Режим по умолчанию
MODE="${1:-native}"
shift 2>/dev/null || true

# Флаги
STOP_MODE=false
RESTART_MODE=false
CLEAN_MODE=false
LOGS_MODE=false
BUILD_MODE=false
INIT_MODE=false

# ============================================================================
# Функции вывода
# ============================================================================

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

step() {
    echo -e "${CYAN}➜${NC} $1"
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
    if [ ! -f "$SCRIPT_DIR/web/public/index.html" ] && [ ! -f "$SCRIPT_DIR/web/public/app.js" ]; then
        return 1
    fi
    return 0
}

build_frontend() {
    step "Сборка frontend..."
    cd "$SCRIPT_DIR/web"
    if [ -f "build.sh" ]; then
        ./build.sh
    else
        # Попытка собрать через npm если есть package.json
        if [ -f "package.json" ]; then
            npm install && npm run build
        else
            error "Скрипт web/build.sh не найден и package.json отсутствует"
        fi
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
# Установка переменных окружения (вместо setenv.sh)
# ============================================================================

setup_env_native() {
    step "Настройка переменных окружения для native режима..."
    
    # Очистка старых переменных
    unset SEMAPHORE_DB_URL
    unset SEMAPHORE_DB_HOST
    unset SEMAPHORE_DB_PORT
    
    # Установка переменных для SQLite
    export SEMAPHORE_DB_DIALECT=sqlite
    export SEMAPHORE_DB_PATH="${SEMAPHORE_DB_PATH:-$SCRIPT_DIR/data/semaphore.db}"
    export SEMAPHORE_WEB_PATH="$SCRIPT_DIR/web/public"
    export SEMAPHORE_TMP_PATH="/tmp/semaphore"
    export SEMAPHORE_TCP_ADDRESS="0.0.0.0:3000"
    export RUST_LOG="${RUST_LOG:-info}"
    
    # Создание директорий
    mkdir -p "$(dirname "$SEMAPHORE_DB_PATH")"
    mkdir -p "$LOG_DIR"
    
    # Создание .env файла
    cat > "$ENV_FILE" << EOF
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
    step "Настройка переменных окружения для hybrid режима..."
    
    # Очистка старых переменных
    unset SEMAPHORE_DB_PATH
    
    # Установка переменных для PostgreSQL
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
    
    # Создание директорий
    mkdir -p "$LOG_DIR"
    
    # Создание .env файла
    cat > "$ENV_FILE" << EOF
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
    step "Настройка переменных окружения для docker режима..."
    
    # Для docker режима переменные нужны только для docker-compose
    # Создаём .env файл для docker-compose
    cat > "$ENV_FILE" << EOF
# Velum - Docker Mode
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@db:5432/semaphore
SEMAPHORE_WEB_PATH=/app/web/public
SEMAPHORE_TMP_PATH=/tmp/semaphore
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
EOF
    
    success "Переменные окружения установлены для Docker"
}

# ============================================================================
# Режим 1: Native (чистый запуск на хосте)
# ============================================================================

mode_native() {
    info "Режим: native (SQLite + Backend + Frontend на хосте)"
    
    # Настройка окружения
    setup_env_native
    
    # Проверка зависимостей
    check_rust
    
    # Проверка и сборка frontend
    if ! check_frontend; then
        warning "Frontend не найден"
        if check_node || [ "$BUILD_MODE" = true ]; then
            build_frontend
        else
            warning "Запуск без frontend (только API)"
        fi
    else
        success "Frontend найден"
    fi
    
    # Сборка backend если нужно
    if [ "$BUILD_MODE" = true ]; then
        build_backend
    fi
    
    # Обработка команд управления
    if [ "$STOP_MODE" = true ]; then
        stop_native
        return
    fi
    
    if [ "$CLEAN_MODE" = true ]; then
        clean_native
        return
    fi
    
    if [ "$LOGS_MODE" = true ]; then
        logs_native
        return
    fi
    
    # Инициализация БД
    if [ "$INIT_MODE" = true ]; then
        init_native
        return
    fi
    
    # Запуск сервера
    start_native
    
    print_status_native
}

# ============================================================================
# Режим 2: Hybrid (PostgreSQL в Docker, остальное на хосте)
# ============================================================================

mode_hybrid() {
    info "Режим: hybrid (PostgreSQL в Docker + Backend + Frontend на хосте)"
    
    # Настройка окружения
    setup_env_hybrid
    
    # Проверка зависимостей
    check_docker
    check_rust
    
    # Проверка и сборка frontend
    if ! check_frontend; then
        warning "Frontend не найден"
        if check_node || [ "$BUILD_MODE" = true ]; then
            build_frontend
        else
            warning "Запуск без frontend (только API)"
        fi
    else
        success "Frontend найден"
    fi
    
    # Сборка backend если нужно
    if [ "$BUILD_MODE" = true ]; then
        build_backend
    fi
    
    # Обработка команд управления
    if [ "$STOP_MODE" = true ]; then
        stop_hybrid
        return
    fi
    
    if [ "$CLEAN_MODE" = true ]; then
        clean_hybrid
        return
    fi
    
    if [ "$LOGS_MODE" = true ]; then
        logs_hybrid
        return
    fi
    
    if [ "$RESTART_MODE" = true ]; then
        restart_hybrid
        return
    fi
    
    # Инициализация БД
    if [ "$INIT_MODE" = true ]; then
        init_hybrid
        return
    fi
    
    # Запуск PostgreSQL в Docker
    start_postgres_docker
    
    # Запуск сервера
    start_hybrid
    
    print_status_hybrid
}

# ============================================================================
# Режим 3: Docker (всё в Docker)
# ============================================================================

mode_docker() {
    info "Режим: docker (всё в Docker)"
    
    # Настройка окружения
    setup_env_docker
    
    # Проверка зависимостей
    check_docker
    
    # Проверка и сборка frontend
    if ! check_frontend; then
        if check_node || [ "$BUILD_MODE" = true ]; then
            build_frontend
        else
            warning "Frontend не найден, используем заглушку"
        fi
    else
        success "Frontend найден"
    fi
    
    # Сборка backend если нужно
    if [ "$BUILD_MODE" = true ]; then
        build_backend
    fi
    
    # Обработка команд управления
    if [ "$STOP_MODE" = true ]; then
        stop_docker
        return
    fi
    
    if [ "$CLEAN_MODE" = true ]; then
        clean_docker
        return
    fi
    
    if [ "$LOGS_MODE" = true ]; then
        logs_docker
        return
    fi
    
    if [ "$RESTART_MODE" = true ]; then
        restart_docker
        return
    fi
    
    # Запуск всех сервисов
    start_docker
    
    print_status_docker
}

# ============================================================================
# Функции для Native режима
# ============================================================================

init_native() {
    step "Инициализация SQLite БД..."
    cd "$SCRIPT_DIR/rust"
    
    # Применение миграций
    cargo run --release -- migrate --upgrade
    success "Миграции применены"
    
    # Создание админа
    step "Создание пользователя admin..."
    cargo run --release -- user add \
        --username admin \
        --name "Administrator" \
        --email admin@localhost \
        --password admin123 \
        --admin
    
    success "Пользователь admin создан"
    echo ""
    info "Теперь запустите сервер: ./start.sh native"
}

start_native() {
    step "Запуск backend..."
    cd "$SCRIPT_DIR/rust"
    
    # Остановка предыдущего backend
    pkill -f "semaphore server" 2>/dev/null || true
    sleep 1
    
    # Запуск в фоне
    nohup cargo run --release -- server --host 0.0.0.0 --port 3000 > "$LOG_DIR/backend.log" 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > "$LOG_DIR/backend.pid"
    
    # Ожидание запуска
    sleep 3
    
    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        success "Backend запущен (PID: $BACKEND_PID)"
    else
        error "Backend не запустился. Проверьте логи: $LOG_DIR/backend.log"
    fi
}

stop_native() {
    step "Остановка backend..."
    pkill -f "semaphore server" 2>/dev/null || true
    rm -f "$LOG_DIR/backend.pid"
    success "Backend остановлен"
}

clean_native() {
    step "Очистка данных SQLite..."
    if [ -n "$SEMAPHORE_DB_PATH" ] && [ -f "$SEMAPHORE_DB_PATH" ]; then
        rm -f "$SEMAPHORE_DB_PATH"
        success "SQLite БД удалена"
    else
        info "SQLite БД не найдена"
    fi
}

logs_native() {
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
    echo -e "   ${CYAN}./start.sh native --stop${NC}   - Остановить backend"
    echo -e "   ${CYAN}./start.sh native --logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}./start.sh native --init${NC}   - Инициализировать БД"
    echo -e "   ${CYAN}./start.sh native --clean${NC}  - Удалить БД"
    echo ""
}

# ============================================================================
# Функции для Hybrid режима
# ============================================================================

start_postgres_docker() {
    step "Запуск PostgreSQL в Docker..."
    
    # Остановка старого контейнера
    docker rm -f semaphore-db 2>/dev/null || true
    
    # Запуск PostgreSQL
    docker run -d \
        --name semaphore-db \
        -e POSTGRES_DB=semaphore \
        -e POSTGRES_USER=semaphore \
        -e POSTGRES_PASSWORD=semaphore_pass \
        -p 5432:5432 \
        -v semaphore_postgres_data:/var/lib/postgresql/data \
        --restart unless-stopped \
        postgres:15-alpine
    
    # Ожидание готовности
    wait_for_postgres
    success "PostgreSQL запущен"
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

init_hybrid() {
    # Запуск PostgreSQL если не запущен
    if ! docker ps --format '{{.Names}}' | grep -q semaphore-db; then
        start_postgres_docker
    fi
    
    step "Инициализация PostgreSQL БД..."
    cd "$SCRIPT_DIR/rust"
    
    # Применение миграций
    cargo run --release -- migrate --upgrade
    success "Миграции применены"
    
    # Создание админа
    step "Создание пользователя admin..."
    cargo run --release -- user add \
        --username admin \
        --name "Administrator" \
        --email admin@localhost \
        --password admin123 \
        --admin
    
    success "Пользователь admin создан"
    echo ""
    info "Теперь запустите сервер: ./start.sh hybrid"
}

start_hybrid() {
    step "Запуск backend..."
    cd "$SCRIPT_DIR/rust"
    
    # Остановка предыдущего backend
    pkill -f "semaphore server" 2>/dev/null || true
    sleep 1
    
    # Запуск в фоне
    nohup cargo run --release -- server --host 0.0.0.0 --port 3000 > "$LOG_DIR/backend.log" 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > "$LOG_DIR/backend.pid"
    
    # Ожидание запуска
    sleep 3
    
    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        success "Backend запущен (PID: $BACKEND_PID)"
    else
        error "Backend не запустился. Проверьте логи: $LOG_DIR/backend.log"
    fi
}

stop_hybrid() {
    step "Остановка backend и PostgreSQL..."
    pkill -f "semaphore server" 2>/dev/null || true
    docker stop semaphore-db 2>/dev/null || true
    rm -f "$LOG_DIR/backend.pid"
    success "Сервисы остановлены"
}

restart_hybrid() {
    step "Перезапуск сервисов..."
    docker restart semaphore-db 2>/dev/null || true
    pkill -f "semaphore server" 2>/dev/null || true
    sleep 2
    start_hybrid
    success "Сервисы перезапущены"
}

clean_hybrid() {
    step "Очистка volumes PostgreSQL..."
    docker volume rm semaphore_postgres_data 2>/dev/null || true
    success "Данные PostgreSQL удалены"
}

logs_hybrid() {
    echo "=== Backend Logs ==="
    if [ -f "$LOG_DIR/backend.log" ]; then
        tail -f "$LOG_DIR/backend.log"
    else
        # Запуск в фоне для просмотра логов Docker
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
    echo -e "   ${CYAN}./start.sh hybrid --stop${NC}   - Остановить сервисы"
    echo -e "   ${CYAN}./start.sh hybrid --logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}./start.sh hybrid --init${NC}   - Инициализировать БД"
    echo -e "   ${CYAN}./start.sh hybrid --clean${NC}  - Удалить данные БД"
    echo -e "   ${CYAN}docker logs semaphore-db${NC}   - Лог PostgreSQL"
    echo ""
}

# ============================================================================
# Функции для Docker режима
# ============================================================================

start_docker() {
    step "Запуск всех сервисов в Docker..."
    
    # Остановка старых контейнеров
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" down --remove-orphans 2>/dev/null || true
    
    # Запуск сервисов
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" up -d --remove-orphans --build
    
    # Ожидание готовности
    wait_for_postgres
    sleep 5
    
    success "Все сервисы запущены"
}

stop_docker() {
    step "Остановка Docker сервисов..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" down
    success "Сервисы остановлены"
}

restart_docker() {
    step "Перезапуск Docker сервисов..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" restart
    success "Сервисы перезапущены"
}

clean_docker() {
    step "Очистка Docker volumes..."
    $COMPOSE_CMD -f "$COMPOSE_FULL_FILE" down -v
    success "Volumes очищены"
}

logs_docker() {
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
    echo -e "   ${CYAN}./start.sh docker --stop${NC}   - Остановить сервисы"
    echo -e "   ${CYAN}./start.sh docker --logs${NC}   - Просмотр логов"
    echo -e "   ${CYAN}./start.sh docker --clean${NC}  - Удалить данные"
    echo ""
}

# ============================================================================
# Парсинг аргументов
# ============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --stop)
            STOP_MODE=true
            shift
            ;;
        --restart)
            RESTART_MODE=true
            shift
            ;;
        --clean)
            CLEAN_MODE=true
            shift
            ;;
        --logs)
            LOGS_MODE=true
            shift
            ;;
        --build)
            BUILD_MODE=true
            shift
            ;;
        --init)
            INIT_MODE=true
            shift
            ;;
        --help|-h)
            head -30 "$0" | tail -27
            exit 0
            ;;
        *)
            error "Неизвестный параметр: $1"
            ;;
    esac
done

# ============================================================================
# Основной запуск
# ============================================================================

# Создание директории для логов
mkdir -p "$LOG_DIR"

case $MODE in
    native)
        mode_native
        ;;
    hybrid)
        mode_hybrid
        ;;
    docker)
        mode_docker
        ;;
    --help|-h)
        head -30 "$0" | tail -27
        exit 0
        ;;
    *)
        error "Неизвестный режим: $MODE"
        echo "Доступные режимы: native, hybrid, docker"
        exit 1
        ;;
esac
