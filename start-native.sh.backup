#!/bin/bash
# ============================================================================
# Semaphore UI - Запуск на чистом железе (SQLite + Rust + Frontend)
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$SCRIPT_DIR/logs/semaphore.pid"
LOG_FILE="$SCRIPT_DIR/logs/semaphore-native.log"
DB_PATH="/tmp/semaphore.db"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

stop_server() {
    info "Остановка сервера..."
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        kill $PID 2>/dev/null || true
        rm -f "$PID_FILE"
        success "Сервер остановлен"
    else
        PID=$(lsof -ti:3000 2>/dev/null || true)
        [ -n "$PID" ] && kill $PID 2>/dev/null || true
        info "Сервер остановлен"
    fi
}

start_server() {
    info "Запуск Semaphore UI на чистом железе..."
    
    BINARY="$SCRIPT_DIR/rust/target/release/semaphore"
    [ ! -f "$BINARY" ] && error "Бинарник не найден: $BINARY"
    
    export SEMAPHORE_DB_DIALECT=sqlite
    export SEMAPHORE_DB_PATH=$DB_PATH
    export SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
    export SEMAPHORE_WEB_PATH=$SCRIPT_DIR/web/public
    export RUST_LOG=info
    export SEMAPHORE_JWT_SECRET=demo-secret-key-for-local-development-only
    
    mkdir -p "$SCRIPT_DIR/logs"
    
    cd "$SCRIPT_DIR"
    nohup "$BINARY" server --host 0.0.0.0 --port 3000 > "$LOG_FILE" 2>&1 &
    PID=$!
    echo $PID > "$PID_FILE"
    
    sleep 3
    success "Сервер запущен (PID: $PID)"
}

show_status() {
    echo ""
    echo "============================================================================"
    echo "                   Semaphore UI запущен!"
    echo "============================================================================"
    echo ""
    echo "📍 Frontend: http://localhost:3000"
    echo "📍 CRUD Демо: http://localhost:3000/demo-crud.html"
    echo "📚 API: http://localhost:3000/api"
    echo ""
    echo "👤 Учетные данные: admin / demo123"
    echo ""
    echo "📋 Логи: tail -f $LOG_FILE"
    echo "🛑 Остановка: ./start-native.sh --stop"
    echo "============================================================================"
}

case "${1:-}" in
    --stop|-s) stop_server; exit 0 ;;
    --restart|-r) stop_server; sleep 1; start_server; show_status; exit 0 ;;
    --logs|-l) tail -f "$LOG_FILE"; exit 0 ;;
    --help|-h) head -10 "$0"; exit 0 ;;
esac

echo ""
echo "=== Semaphore UI - Чистое железо (SQLite + Rust) ==="
echo ""

if [ -f "$PID_FILE" ] && ps -p $(cat "$PID_FILE") > /dev/null 2>&1; then
    warning "Сервер уже запущен"
    show_status
    exit 0
fi

start_server
show_status
