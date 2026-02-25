#!/bin/bash

# Скрипт для безопасного удаления Go модулей
# Использование: ./remove_go_module.sh <модуль>

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SEMAPHORE_ROOT="$(dirname "$PROJECT_ROOT")"

cd "$SEMAPHORE_ROOT"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Функция для вывода сообщений
log_info() {
    echo -e "${GREEN}ℹ️  $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Проверка аргументов
if [ -z "$1" ]; then
    log_error "Не указан модуль для удаления"
    echo "Использование: $0 <модуль>"
    echo ""
    echo "Доступные модули:"
    echo "  tz            - pkg/tz (простой, 20+ зависимостей)"
    echo "  random        - pkg/random (простой, 9 зависимостей)"
    echo "  conv          - pkg/conv (средний, 4 зависимости)"
    echo "  common_errors - pkg/common_errors (средний, 20+ зависимостей)"
    echo "  task_logger   - pkg/task_logger (сложный, 30+ зависимостей)"
    echo "  ssh           - pkg/ssh (очень сложный, 7 зависимостей)"
    exit 1
fi

MODULE=$1
MODULE_PATH=""
DEPENDENCY_PATTERN=""

# Определение пути и паттерна для модуля
case $MODULE in
    tz)
        MODULE_PATH="pkg/tz"
        DEPENDENCY_PATTERN="pkg/tz"
        ;;
    random)
        MODULE_PATH="pkg/random"
        DEPENDENCY_PATTERN="pkg/random"
        ;;
    conv)
        MODULE_PATH="pkg/conv"
        DEPENDENCY_PATTERN="pkg/conv"
        ;;
    common_errors)
        MODULE_PATH="pkg/common_errors"
        DEPENDENCY_PATTERN="common_errors"
        ;;
    task_logger)
        MODULE_PATH="pkg/task_logger"
        DEPENDENCY_PATTERN="pkg/task_logger"
        ;;
    ssh)
        MODULE_PATH="pkg/ssh"
        DEPENDENCY_PATTERN="pkg/ssh"
        ;;
    *)
        log_error "Неизвестный модуль: $MODULE"
        exit 1
        ;;
esac

echo "=============================================="
echo "🗑️  Удаление Go модуля: $MODULE"
echo "=============================================="
echo ""

# Проверка существования модуля
if [ ! -d "$SEMAPHORE_ROOT/$MODULE_PATH" ]; then
    log_error "Модуль не найден: $MODULE_PATH"
    exit 1
fi

log_info "Путь к модулю: $MODULE_PATH"
log_info "Паттерн зависимости: $DEPENDENCY_PATTERN"
echo ""

# Поиск зависимостей
log_warn "🔍 Поиск зависимостей..."
echo ""

DEPENDENCY_FILES=$(grep -rl "$DEPENDENCY_PATTERN" --include="*.go" "$SEMAPHORE_ROOT" 2>/dev/null | head -50)
DEPENDENCY_COUNT=$(echo "$DEPENDENCY_FILES" | grep -c . || echo 0)

if [ "$DEPENDENCY_COUNT" -gt 0 ]; then
    log_error "⚠️  НАЙДЕНО ЗАВИСИМОСТЕЙ: $DEPENDENCY_COUNT"
    echo ""
    echo "Файлы, использующие этот модуль:"
    echo "$DEPENDENCY_FILES" | sed 's|^|  - |'
    echo ""
    
    if [ "$MODULE" == "ssh" ] || [ "$MODULE" == "task_logger" ]; then
        log_error "❌ Этот модуль КРИТИЧНЫЙ и не может быть удалён!"
        log_error "Сначала перепишите функциональность на Rust."
        exit 1
    fi
    
    read -p "🤔 Вы действительно хотите продолжить? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Отменено пользователем"
        exit 0
    fi
else
    log_info "✅ Зависимости не найдены"
    echo ""
fi

# Создание резервной копии
BACKUP_DIR="/tmp/semaphore_backup_$(date +%Y%m%d_%H%M%S)"
log_info "Создание резервной копии в: $BACKUP_DIR"
mkdir -p "$BACKUP_DIR"
cp -r "$SEMAPHORE_ROOT/$MODULE_PATH" "$BACKUP_DIR/"

# Удаление модуля
log_warn "🗑️  Удаление модуля..."
rm -rf "$SEMAPHORE_ROOT/$MODULE_PATH"
log_info "✅ Модуль удалён"
echo ""

# Проверка компиляции
log_info "🔨 Проверка компиляции Go проекта..."
cd "$SEMAPHORE_ROOT"

if go build ./... 2>&1 | head -20; then
    log_info "✅ Проект компилируется успешно"
else
    log_error "❌ Ошибки компиляции!"
    log_error "Восстановите модуль из резервной копии: $BACKUP_DIR"
    exit 1
fi

echo ""
echo "=============================================="
echo "✅ Удаление завершено"
echo "=============================================="
echo ""
echo "📝 Резервная копия: $BACKUP_DIR"
echo ""
echo "📌 Следующие шаги:"
echo "   1. Обновите CHANGELOG.md"
echo "   2. Запустите тесты: go test ./..."
echo "   3. Закоммитьте изменения"
echo ""
