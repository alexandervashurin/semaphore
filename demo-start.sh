#!/bin/bash
# ============================================================================
# Semaphore UI - CRUD Демо Быстрый Старт
# ============================================================================
# Этот скрипт автоматически запускает всё необходимое для работы CRUD демо
# ============================================================================

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Функции для вывода
info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

# Проверка наличия Docker
check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker не найден. Установите Docker."
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        error "docker-compose не найден. Установите docker-compose."
    fi
    
    success "Docker и docker-compose найдены"
}

# Проверка наличия Rust/Cargo
check_rust() {
    if ! command -v cargo &> /dev/null; then
        warning "Rust/Cargo не найден. Backend не будет запущен автоматически."
        return 1
    fi
    
    success "Rust/Cargo найден"
    return 0
}

# Запуск Docker сервисов
start_docker() {
    info "Запуск PostgreSQL и Frontend..."
    
    docker-compose up -d db frontend
    
    # Ожидание готовности PostgreSQL
    info "Ожидание готовности PostgreSQL..."
    sleep 5
    
    for i in {1..30}; do
        if docker-compose exec -T db pg_isready -U semaphore -d semaphore &> /dev/null; then
            success "PostgreSQL готов"
            break
        fi
        if [ $i -eq 30 ]; then
            error "PostgreSQL не запустился"
        fi
        sleep 1
    done
    
    success "Docker сервисы запущены"
}

# Запуск backend
start_backend() {
    info "Запуск Rust backend..."
    
    # Проверка переменных окружения
    if [ -z "$SEMAPHORE_DB_URL" ]; then
        export SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@localhost:5432/semaphore"
    fi
    
    # Установка пути к web файлам (абсолютный путь)
    export SEMAPHORE_WEB_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/web/public"
    
    cd rust
    
    # Сборка и запуск
    info "Сборка backend (это может занять некоторое время)..."
    cargo build --release
    
    info "Запуск сервера на порту 3000..."
    cargo run --release -- server --host 0.0.0.0 --port 3000
}

# Вывод информации
show_info() {
    echo ""
    echo "============================================================================"
    echo "                   Semaphore UI - CRUD Демо запущено!"
    echo "============================================================================"
    echo ""
    echo "📍 Frontend доступен по адресу:"
    echo "   http://localhost/demo-crud.html"
    echo ""
    echo "🔧 Backend API доступен по адресу:"
    echo "   http://localhost:3000/api"
    echo ""
    echo "📚 Swagger документация:"
    echo "   http://localhost:3000/swagger"
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
    echo "📖 Документация:"
    echo "   - CRUD_DEMO.md - полное руководство по демо"
    echo "   - API.md - документация API"
    echo "   - README.md - общая информация"
    echo ""
    echo "🛑 Для остановки выполните:"
    echo "   ./stop.sh"
    echo ""
    echo "============================================================================"
    echo ""
}

# Остановка сервисов
stop_services() {
    info "Остановка сервисов..."
    docker-compose down
    success "Сервисы остановлены"
}

# Перезапуск сервисов
restart_services() {
    stop_services
    start_docker
}

# Просмотр логов
view_logs() {
    info "Просмотр логов (Ctrl+C для выхода)..."
    docker-compose logs -f
}

# Сброс демо данных
reset_demo() {
    warning "Сброс демо-данных..."
    docker-compose down -v
    start_docker
    success "Демо-данные сброшены"
}

# Главная функция
main() {
    echo ""
    echo "============================================================================"
    echo "              Semaphore UI - CRUD Демо Быстрый Старт"
    echo "============================================================================"
    echo ""
    
    case "${1:-}" in
        --backend|-b)
            check_docker
            check_rust && start_backend
            ;;
        --stop|-s)
            check_docker
            stop_services
            ;;
        --restart|-r)
            check_docker
            restart_services
            ;;
        --logs|-l)
            check_docker
            view_logs
            ;;
        --reset)
            check_docker
            reset_demo
            ;;
        --help|-h)
            echo "Использование: $0 [опции]"
            echo ""
            echo "Опции:"
            echo "  --backend, -b     Запустить backend (Rust)"
            echo "  --stop, -s        Остановить все сервисы"
            echo "  --restart, -r     Перезапустить сервисы"
            echo "  --logs, -l        Просмотр логов"
            echo "  --reset           Сброс демо-данных"
            echo "  --help, -h        Эта справка"
            echo ""
            echo "Без опций: запуск Docker сервисов (PostgreSQL + Frontend)"
            echo ""
            echo "Примеры:"
            echo "  $0                # Запуск Docker сервисов"
            echo "  $0 --backend      # Запуск backend в текущем терминале"
            echo "  $0 --stop         # Остановка всех сервисов"
            echo "  $0 --reset        # Сброс демо-данных"
            echo ""
            ;;
        *)
            check_docker
            start_docker
            show_info
            
            info "Для запуска backend откройте новый терминал и выполните:"
            echo "   $0 --backend"
            echo ""
            ;;
    esac
}

# Запуск
main "$@"
