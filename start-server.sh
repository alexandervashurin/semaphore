#!/bin/bash
# =============================================================================
# Velum Server Startup Script
# =============================================================================
# Запуск сервера Velum с PostgreSQL и SQLite
# =============================================================================

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Пути
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CD_DIR="${SCRIPT_DIR}"
RUST_DIR="${CD_DIR}/rust"
ENV_FILE="${CD_DIR}/.env"

# =============================================================================
# Функции
# =============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 не найден. Пожалуйста, установите $1."
        exit 1
    fi
}

wait_for_postgres() {
    log_info "Ожидание готовности PostgreSQL..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if docker ps --format '{{.Status}}' | grep -q "healthy" 2>/dev/null; then
            log_success "PostgreSQL готов"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    
    log_error "PostgreSQL не запустился за ${max_attempts} секунд"
    return 1
}

start_postgres() {
    log_info "Запуск PostgreSQL через Docker Compose..."
    
    if docker compose version &> /dev/null; then
        docker compose -f "${CD_DIR}/docker-compose.yml" up -d db 2>/dev/null || \
        docker compose up -d db
    elif docker-compose version &> /dev/null; then
        docker-compose -f "${CD_DIR}/docker-compose.yml" up -d db 2>/dev/null || \
        docker-compose up -d db
    else
        log_error "Docker Compose не найден"
        exit 1
    fi
    
    wait_for_postgres
}

stop_postgres() {
    log_info "Остановка PostgreSQL..."
    if docker compose version &> /dev/null; then
        docker compose -f "${CD_DIR}/docker-compose.yml" stop db 2>/dev/null || true
    elif docker-compose version &> /dev/null; then
        docker-compose -f "${CD_DIR}/docker-compose.yml" stop db 2>/dev/null || true
    fi
}

start_server() {
    log_info "Запуск сервера Velum..."
    
    cd "${RUST_DIR}"
    
    # Запуск в фоне
    nohup cargo run --release -- server --host 0.0.0.0 --port 3000 \
        > "${CD_DIR}/velum-server.log" 2>&1 &
    
    local pid=$!
    echo $pid > "${CD_DIR}/velum-server.pid"
    
    log_success "Сервер запущен (PID: ${pid})"
    log_info "Лог: ${CD_DIR}/velum-server.log"
}

stop_server() {
    log_info "Остановка сервера Velum..."
    
    if [ -f "${CD_DIR}/velum-server.pid" ]; then
        local pid=$(cat "${CD_DIR}/velum-server.pid")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            log_success "Сервер остановлен (PID: ${pid})"
        else
            log_warning "Сервер не запущен"
        fi
        rm -f "${CD_DIR}/velum-server.pid"
    else
        pkill -f "velum server" 2>/dev/null || true
        log_info "Сервер остановлен"
    fi
}

check_server_status() {
    if [ -f "${CD_DIR}/velum-server.pid" ]; then
        local pid=$(cat "${CD_DIR}/velum-server.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_success "Сервер работает (PID: ${pid})"
            return 0
        fi
    fi
    
    if pgrep -f "velum server" > /dev/null; then
        local pid=$(pgrep -f "velum server")
        log_success "Сервер работает (PID: ${pid})"
        return 0
    fi
    
    log_warning "Сервер не работает"
    return 1
}

wait_for_server() {
    log_info "Ожидание готовности сервера..."
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s http://localhost:3000/api/health > /dev/null 2>&1; then
            log_success "Сервер готов"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    
    log_error "Сервер не запустился за ${max_attempts} секунд"
    return 1
}

show_status() {
    echo ""
    echo "=== Статус Velum ==="
    echo ""
    
    # PostgreSQL
    if docker ps --format '{{.Names}}\t{{.Status}}' | grep -q "semaphore-db.*healthy"; then
        log_success "PostgreSQL: работает"
    else
        log_warning "PostgreSQL: не работает"
    fi
    
    # Сервер
    check_server_status
    
    # Health check
    echo ""
    echo "=== Health Check ==="
    curl -s http://localhost:3000/api/health 2>/dev/null || log_warning "Сервер недоступен"
    echo ""
}

show_help() {
    echo "Velum Server Management Script"
    echo ""
    echo "Использование: $0 {start|stop|restart|status|logs|clean}"
    echo ""
    echo "Команды:"
    echo "  start   - Запустить PostgreSQL и сервер Velum"
    echo "  stop    - Остановить сервер и PostgreSQL"
    echo "  restart - Перезапустить сервер и PostgreSQL"
    echo "  status  - Показать статус сервисов"
    echo "  logs    - Показать логи сервера (tail -f)"
    echo "  clean   - Остановить всё и удалить временные файлы"
    echo ""
    echo "Примеры:"
    echo "  $0 start      # Запуск"
    echo "  $0 stop       # Остановка"
    echo "  $0 logs       # Просмотр логов"
    echo ""
}

show_banner() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║                    Velum Server                           ║"
    echo "║         Rust Edition - PostgreSQL + SQLite                ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo ""
}

# =============================================================================
# Основная логика
# =============================================================================

cd "${CD_DIR}"

# Проверка зависимостей
check_command docker
check_command cargo

case "${1:-}" in
    start)
        show_banner
        log_info "Запуск Velum..."
        
        # Проверка .env файла
        if [ ! -f "${ENV_FILE}" ]; then
            log_warning ".env файл не найден. Создаю..."
            cat > "${ENV_FILE}" << 'EOF'
# Velum - PostgreSQL
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5432/semaphore
SEMAPHORE_WEB_PATH=/home/alex/Документы/программирование/github/semaphore/web/public
SEMAPHORE_TMP_PATH=/tmp/semaphore
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
EOF
            log_success ".env файл создан"
        fi
        
        # Запуск PostgreSQL
        start_postgres
        
        # Остановка старого сервера если есть
        stop_server 2>/dev/null || true
        
        # Запуск сервера
        start_server
        
        # Ожидание готовности
        wait_for_server
        
        echo ""
        log_success "Velum готов!"
        echo ""
        echo "  🌐 Web UI: http://localhost:3000"
        echo "  📊 API:    http://localhost:3000/api"
        echo "  ❤️ Health: http://localhost:3000/api/health"
        echo ""
        echo "Учётные данные (demo):"
        echo "  👤 admin / demo123"
        echo ""
        echo "Для просмотра логов: $0 logs"
        echo "Для остановки: $0 stop"
        echo ""
        ;;
        
    stop)
        show_banner
        stop_server
        stop_postgres
        log_success "Velum остановлен"
        echo ""
        ;;
        
    restart)
        show_banner
        log_info "Перезапуск Velum..."
        stop_server
        stop_postgres
        sleep 2
        $0 start
        ;;
        
    status)
        show_banner
        show_status
        ;;
        
    logs)
        if [ -f "${CD_DIR}/velum-server.log" ]; then
            tail -f "${CD_DIR}/velum-server.log"
        else
            log_warning "Лог файл не найден"
        fi
        ;;
        
    clean)
        show_banner
        log_info "Очистка..."
        stop_server
        stop_postgres
        rm -f "${CD_DIR}/velum-server.pid"
        rm -f "${CD_DIR}/velum-server.log"
        log_success "Очистка завершена"
        ;;
        
    *)
        show_banner
        show_help
        exit 1
        ;;
esac

exit 0
