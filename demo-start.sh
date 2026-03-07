#!/bin/bash
# ============================================================================
# Semaphore UI - CRUD Демо Быстрый Старт
# ============================================================================
# Автоматически запускает:
#   1. Docker контейнеры (PostgreSQL + Frontend)
#   2. Backend (Rust)
#   3. Логирование в режиме INFO
# ============================================================================
# Использование: ./demo-start.sh [OPTIONS]
#
# Опции:
#   --stop          Остановить все сервисы
#   --restart       Перезапустить все сервисы
#   --logs          Переключиться в режим просмотра логов
#   --clean         Очистить volumes перед запуском
#   --no-backend    Не запускать backend (только Docker)
#   --help, -h      Показать эту справку
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"
LOG_FILE="$SCRIPT_DIR/logs/demo-start.log"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Флаги
STOP_MODE=false
RESTART_MODE=false
LOGS_MODE=false
CLEAN_MODE=false
NO_BACKEND=false

# PID процесса backend
BACKEND_PID=""

# ============================================================================
# Функции для вывода (режим INFO)
# ============================================================================

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
    log_message "INFO" "$1"
}

success() {
    echo -e "${GREEN}[INFO]${NC} ✅ $1"
    log_message "INFO" "$1"
}

warning() {
    echo -e "${YELLOW}[INFO]${NC} ⚠️  $1"
    log_message "WARN" "$1"
}

error() {
    echo -e "${RED}[ERROR]${NC} ❌ $1"
    log_message "ERROR" "$1"
    exit 1
}

debug() {
    if [ "${DEBUG:-false}" = "true" ]; then
        echo -e "${CYAN}[DEBUG]${NC} $1"
        log_message "DEBUG" "$1"
    fi
}

# Логирование в файл
log_message() {
    local level="$1"
    local message="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Создаём директорию для логов если не существует
    mkdir -p "$(dirname "$LOG_FILE")"
    
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
}

# ============================================================================
# Проверка зависимостей
# ============================================================================

check_docker() {
    info "Проверка Docker..."
    
    if ! command -v docker &> /dev/null; then
        error "Docker не найден. Установите Docker."
    fi

    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        error "docker-compose не найден. Установите docker-compose."
    fi
    
    # Проверка, что Docker запущен
    if ! docker info &> /dev/null; then
        error "Docker не запущен. Запустите Docker daemon."
    fi

    success "Docker и docker-compose найдены"
    log_message "INFO" "Docker проверка пройдена"
}

check_rust() {
    info "Проверка Rust/Cargo..."
    
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo не найден. Установите Rust: https://rustup.rs/"
    fi

    success "Rust/Cargo найден (версия: $(cargo --version))"
    log_message "INFO" "Rust проверка пройдена"
}

check_frontend() {
    info "Проверка frontend..."
    
    if [ ! -f "$SCRIPT_DIR/web/public/app.js" ] || [ ! -s "$SCRIPT_DIR/web/public/app.js" ]; then
        warning "Frontend не собран. Запуск сборки..."
        build_frontend
    else
        success "Frontend уже собран"
        log_message "INFO" "Frontend проверка пройдена"
    fi
}

# ============================================================================
# Сборка frontend
# ============================================================================

build_frontend() {
    info "Сборка frontend..."
    log_message "INFO" "Запуск сборки frontend"
    
    if [ -f "$SCRIPT_DIR/web/build.sh" ]; then
        "$SCRIPT_DIR/web/build.sh"
        success "Frontend собран"
        log_message "INFO" "Frontend сборка завершена"
    else
        error "Скрипт web/build.sh не найден"
    fi
}

# ============================================================================
# Управление Docker сервисами
# ============================================================================

get_compose_cmd() {
    if docker compose version &> /dev/null 2>&1; then
        echo "docker compose"
    else
        echo "docker-compose"
    fi
}

start_docker() {
    info "Запуск Docker сервисов (PostgreSQL + Frontend)..."
    log_message "INFO" "Запуск Docker сервисов"
    
    local COMPOSE_CMD=$(get_compose_cmd)
    
    # Очистка volumes если нужно
    if [ "$CLEAN_MODE" = true ]; then
        info "Очистка volumes..."
        log_message "INFO" "Очистка volumes"
        $COMPOSE_CMD -f "$COMPOSE_FILE" down -v
    fi
    
    # Запуск сервисов
    $COMPOSE_CMD -f "$COMPOSE_FILE" up -d db frontend
    
    # Ожидание готовности PostgreSQL
    info "Ожидание готовности PostgreSQL..."
    log_message "INFO" "Ожидание готовности PostgreSQL"
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if docker exec semaphore-db pg_isready -U semaphore -d semaphore &> /dev/null 2>&1; then
            success "PostgreSQL готов (попыток: $attempt)"
            log_message "INFO" "PostgreSQL готов после $attempt попыток"
            break
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            error "PostgreSQL не запустился за $max_attempts секунд"
        fi
        
        sleep 1
        ((attempt++))
    done
    
    success "Docker сервисы запущены"
    log_message "INFO" "Docker сервисы запущены успешно"
}

stop_docker() {
    info "Остановка Docker сервисов..."
    log_message "INFO" "Остановка Docker сервисов"
    
    local COMPOSE_CMD=$(get_compose_cmd)
    $COMPOSE_CMD -f "$COMPOSE_FILE" down
    
    success "Docker сервисы остановлены"
    log_message "INFO" "Docker сервисы остановлены"
}

restart_docker() {
    info "Перезапуск Docker сервисов..."
    log_message "INFO" "Перезапуск Docker сервисов"
    
    stop_docker
    start_docker
    
    success "Docker сервисы перезапущены"
    log_message "INFO" "Docker сервисы перезапущены"
}

# ============================================================================
# Управление backend
# ============================================================================

start_backend() {
    info "Запуск Rust backend..."
    log_message "INFO" "Запуск Rust backend"
    
    # Проверка переменных окружения
    export SEMAPHORE_DB_URL="${SEMAPHORE_DB_URL:-postgres://semaphore:semaphore_pass@localhost:5432/semaphore}"
    export SEMAPHORE_WEB_PATH="$SCRIPT_DIR/web/public"
    export RUST_LOG="${RUST_LOG:-info}"
    
    info "Переменные окружения:"
    info "  SEMAPHORE_DB_URL: $SEMAPHORE_DB_URL"
    info "  SEMAPHORE_WEB_PATH: $SEMAPHORE_WEB_PATH"
    info "  RUST_LOG: $RUST_LOG"
    
    log_message "INFO" "Backend переменные: DB=$SEMAPHORE_DB_URL, WEB=$SEMAPHORE_WEB_PATH, LOG=$RUST_LOG"
    
    cd "$SCRIPT_DIR/rust"
    
    # Сборка и запуск в фоне
    info "Сборка backend (это может занять некоторое время)..."
    log_message "INFO" "Начало сборки backend"
    
    cargo build --release 2>&1 | while read line; do
        debug "cargo: $line"
    done
    
    info "Запуск сервера на порту 3000..."
    log_message "INFO" "Запуск backend сервера на порту 3000"
    
    # Запуск в фоне с перенаправлением вывода в лог
    nohup cargo run --release -- server --host 0.0.0.0 --port 3000 > "$SCRIPT_DIR/logs/backend.log" 2>&1 &
    BACKEND_PID=$!
    
    echo $BACKEND_PID > "$SCRIPT_DIR/logs/backend.pid"
    
    success "Backend запущен (PID: $BACKEND_PID)"
    log_message "INFO" "Backend запущен с PID=$BACKEND_PID"
    
    # Ожидание готовности backend
    info "Ожидание готовности backend..."
    sleep 3
    
    local max_attempts=20
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s http://localhost:3000/api &> /dev/null; then
            success "Backend готов (попыток: $attempt)"
            log_message "INFO" "Backend готов после $attempt попыток"
            break
        fi
        
        if [ $attempt -eq $max_attempts ]; then
            warning "Backend не ответил за $max_attempts секунд, но продолжает запускаться"
            log_message "WARN" "Backend не ответил после $max_attempts попыток"
        fi
        
        sleep 1
        ((attempt++))
    done
}

stop_backend() {
    info "Остановка backend..."
    log_message "INFO" "Остановка backend"
    
    # Чтение PID из файла
    if [ -f "$SCRIPT_DIR/logs/backend.pid" ]; then
        BACKEND_PID=$(cat "$SCRIPT_DIR/logs/backend.pid")
        
        if ps -p $BACKEND_PID > /dev/null 2>&1; then
            kill $BACKEND_PID 2>/dev/null || true
            
            # Ожидание остановки
            local count=0
            while ps -p $BACKEND_PID > /dev/null 2>&1 && [ $count -lt 10 ]; do
                sleep 1
                ((count++))
            done
            
            # Принудительная остановка если нужно
            if ps -p $BACKEND_PID > /dev/null 2>&1; then
                kill -9 $BACKEND_PID 2>/dev/null || true
            fi
            
            success "Backend остановлен (PID: $BACKEND_PID)"
            log_message "INFO" "Backend остановлен"
        else
            info "Backend не запущен"
            log_message "INFO" "Backend не был запущен"
        fi
        
        rm -f "$SCRIPT_DIR/logs/backend.pid"
    else
        # Попытка найти процесс по порту
        local pid=$(lsof -ti:3000 2>/dev/null || true)
        if [ -n "$pid" ]; then
            kill $pid 2>/dev/null || true
            success "Backend остановлен (найден по порту 3000)"
            log_message "INFO" "Backend остановлен по порту"
        else
            info "Backend не запущен"
            log_message "INFO" "Backend не был запущен (PID файл не найден)"
        fi
    fi
}

# ============================================================================
# Просмотр логов
# ============================================================================

view_logs() {
    info "Просмотр логов (Ctrl+C для выхода)..."
    log_message "INFO" "Начало просмотра логов"
    
    echo ""
    echo "============================================================================"
    echo "                         Логи в реальном времени"
    echo "============================================================================"
    echo ""
    
    # Логи Docker сервисов через docker logs
    echo -e "${CYAN}=== Docker сервисы ===${NC}"
    
    # Получаем список контейнеров
    for container in semaphore-db semaphore-frontend; do
        if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
            echo -e "\n${BLUE}--- $container ---${NC}"
            docker logs -f "$container" &
        fi
    done
    
    DOCKER_LOGS_PID=$!
    
    # Даем время на запуск
    sleep 2
    
    # Лог backend если запущен
    if [ -f "$SCRIPT_DIR/logs/backend.pid" ]; then
        BACKEND_PID=$(cat "$SCRIPT_DIR/logs/backend.pid")
        if ps -p $BACKEND_PID > /dev/null 2>&1; then
            echo ""
            echo -e "${CYAN}=== Backend (PID: $BACKEND_PID) ===${NC}"
            tail -f "$SCRIPT_DIR/logs/backend.log" &
            BACKEND_LOGS_PID=$!
        fi
    fi
    
    # Обработка Ctrl+C
    trap "kill $DOCKER_LOGS_PID $BACKEND_LOGS_PID 2>/dev/null; exit 0" INT
    
    wait
}

# ============================================================================
# Вывод информации
# ============================================================================

show_info() {
    echo ""
    echo "============================================================================"
    echo "                   Semaphore UI - CRUD Демо запущено!"
    echo "============================================================================"
    echo ""
    echo "📍 Frontend доступен по адресу:"
    echo -e "   ${GREEN}http://localhost/demo-crud.html${NC}"
    echo ""
    echo "🔧 Backend API доступен по адресу:"
    echo -e "   ${GREEN}http://localhost:3000/api${NC}"
    echo ""
    echo "📚 Swagger документация:"
    echo -e "   ${GREEN}http://localhost:3000/swagger${NC}"
    echo ""
    echo "👤 Учетные данные для входа:"
    echo "   ┌──────────────┬────────────┬─────────────────┐"
    echo "   │ Логин        │ Пароль     │ Роль            │"
    echo "   ├──────────────┼────────────┼─────────────────┤"
    echo "   │ admin        │ demo123    │ Администратор   │"
    echo "   │ john.doe     │ demo123    │ Менеджер        │"
    echo "   │ jane.smith   │ demo123    │ Менеджер        │"
    echo "   │ devops       │ demo123    │ Исполнитель     │"
    echo "   └──────────────┴────────────┴─────────────────┘"
    echo ""
    echo "📋 Логи:"
    echo -e "   ${CYAN}./demo-start.sh --logs${NC}           - Просмотр логов"
    echo -e "   ${CYAN}tail -f $LOG_FILE${NC} - Лог скрипта"
    echo -e "   ${CYAN}tail -f logs/backend.log${NC}         - Лог backend"
    echo ""
    echo "🛑 Для остановки выполните:"
    echo -e "   ${RED}./demo-start.sh --stop${NC}"
    echo ""
    echo "============================================================================"
    echo ""
    
    log_message "INFO" "Демо успешно запущено. Информация показана пользователю."
}

# ============================================================================
# Остановка всех сервисов
# ============================================================================

stop_all() {
    echo ""
    info "Остановка всех сервисов..."
    log_message "INFO" "Начало остановки всех сервисов"
    
    stop_backend
    stop_docker
    
    echo ""
    success "Все сервисы остановлены"
    log_message "INFO" "Все сервисы остановлены"
    
    echo ""
    echo "============================================================================"
    echo "                   Semaphore UI - CRUD Демо остановлено"
    echo "============================================================================"
    echo ""
}

# ============================================================================
# Парсинг аргументов
# ============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --stop|-s)
            STOP_MODE=true
            shift
            ;;
        --restart|-r)
            RESTART_MODE=true
            shift
            ;;
        --logs|-l)
            LOGS_MODE=true
            shift
            ;;
        --clean)
            CLEAN_MODE=true
            shift
            ;;
        --no-backend)
            NO_BACKEND=true
            shift
            ;;
        --help|-h)
            head -22 "$0" | tail -19
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Неизвестный параметр: $1${NC}"
            echo "Используйте --help для справки"
            exit 1
            ;;
    esac
done

# ============================================================================
# Главная функция
# ============================================================================

main() {
    echo ""
    echo "============================================================================"
    echo "              Semaphore UI - CRUD Демо Быстрый Старт"
    echo "============================================================================"
    echo ""
    
    # Режим остановки
    if [ "$STOP_MODE" = true ]; then
        stop_all
        exit 0
    fi
    
    # Режим перезапуска
    if [ "$RESTART_MODE" = true ]; then
        check_docker
        stop_all
        echo ""
        echo "============================================================================"
        echo ""
    fi
    
    # Режим просмотра логов
    if [ "$LOGS_MODE" = true ]; then
        check_docker
        view_logs
        exit 0
    fi
    
    # Основной режим - запуск всего
    check_docker
    check_frontend
    
    start_docker
    
    # Запуск backend если не отключен
    if [ "$NO_BACKEND" = false ]; then
        check_rust
        start_backend
    else
        info "Backend не запускается (--no-backend)"
        log_message "INFO" "Backend отключен пользователем"
    fi
    
    show_info
    
    info "Логирование в режиме INFO. Лог файл: $LOG_FILE"
    info "Для остановки выполните: ./demo-start.sh --stop"
    
    log_message "INFO" "Скрипт завершил выполнение успешно"
}

# Запуск
main "$@"
