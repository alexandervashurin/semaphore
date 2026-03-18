#!/bin/bash
# ============================================================================
# Velum - Restore Script
# ============================================================================
# Восстановление из резервной копии
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# ============================================================================
# Проверка аргументов
# ============================================================================

if [ $# -lt 1 ]; then
    echo "Использование: $0 <backup_file.tar.gz>"
    echo ""
    echo "Примеры:"
    echo "  $0 backups/semaphore_backup_20260310_120000.tar.gz"
    echo "  $0 /path/to/backup.tar.gz"
    exit 1
fi

BACKUP_FILE="$1"

if [ ! -f "$BACKUP_FILE" ]; then
    error "Файл бэкапа не найден: $BACKUP_FILE"
fi

# ============================================================================
# Конфигурация
# ============================================================================

DB_TYPE="${SEMAPHORE_DB_TYPE:-postgres}"
DB_HOST="${SEMAPHORE_DB_HOST:-localhost}"
DB_PORT="${SEMAPHORE_DB_PORT:-5432}"
DB_NAME="${SEMAPHORE_DB_NAME:-semaphore}"
DB_USER="${SEMAPHORE_DB_USER:-semaphore}"
DB_PASS="${SEMAPHORE_DB_PASS:-}"

# Для SQLite
SQLITE_PATH="${SEMAPHORE_SQLITE_PATH:-/tmp/semaphore.db}"

# Для PostgreSQL
PGPASSWORD="$DB_PASS"
export PGPASSWORD

# ============================================================================
# Проверка зависимостей
# ============================================================================

check_dependencies() {
    info "Проверка зависимостей..."
    
    local deps=("tar" "gzip")
    
    if [[ "$DB_TYPE" == "postgres" ]]; then
        deps+=("psql")
    elif [[ "$DB_TYPE" == "mysql" ]]; then
        deps+=("mysql")
    fi
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            error "Зависимость не найдена: $dep"
        fi
    done
    
    success "Все зависимости найдены"
}

# ============================================================================
# Подтверждение
# ============================================================================

confirm_restore() {
    warning "ВНИМАНИЕ: Восстановление заменит текущие данные!"
    echo ""
    echo "  Файл бэкапа: $BACKUP_FILE"
    echo "  Тип БД: $DB_TYPE"
    echo "  База данных: $DB_NAME"
    echo ""
    read -p "Продолжить? (yes/no): " -r
    echo
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        info "Восстановление отменено"
        exit 0
    fi
}

# ============================================================================
# Распаковка архива
# ============================================================================

extract_backup() {
    local temp_dir="$SCRIPT_DIR/restore_temp_$(date '+%Y%m%d_%H%M%S')"
    mkdir -p "$temp_dir"
    
    info "Распаковка архива..."
    
    tar -xzf "$BACKUP_FILE" -C "$temp_dir"
    
    # Найти файлы
    DB_FILE=$(find "$temp_dir" -name "*.sql" -o -name "*.sql.sqlite" | head -1)
    CONFIG_DIR=$(find "$temp_dir" -type d -name "config_backup_*" | head -1)
    
    if [ -z "$DB_FILE" ]; then
        error "Файл базы данных не найден в архиве"
    fi
    
    success "Архив распакован: $temp_dir"
    
    echo "$temp_dir"
}

# ============================================================================
# Восстановление базы данных
# ============================================================================

restore_database() {
    local db_file="$1"
    
    info "Восстановление базы данных ($DB_TYPE)..."
    
    case "$DB_TYPE" in
        postgres)
            # Drop existing tables
            info "Очистка существующей базы..."
            psql \
                -h "$DB_HOST" \
                -p "$DB_PORT" \
                -U "$DB_USER" \
                -d "$DB_NAME" \
                -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
            
            # Restore
            psql \
                -h "$DB_HOST" \
                -p "$DB_PORT" \
                -U "$DB_USER" \
                -d "$DB_NAME" \
                -f "$db_file"
            
            success "PostgreSQL восстановлена"
            ;;
        
        mysql)
            # Drop existing database
            info "Очистка существующей базы..."
            mysql \
                -h "$DB_HOST" \
                -P "$DB_PORT" \
                -u "$DB_USER" \
                -p"$DB_PASS" \
                -e "DROP DATABASE IF EXISTS $DB_NAME; CREATE DATABASE $DB_NAME;"
            
            # Restore
            mysql \
                -h "$DB_HOST" \
                -P "$DB_PORT" \
                -u "$DB_USER" \
                -p"$DB_PASS" \
                "$DB_NAME" \
                < "$db_file"
            
            success "MySQL восстановлена"
            ;;
        
        sqlite)
            local sqlite_file="${db_file%.sql}.sqlite"
            if [ -f "$sqlite_file" ]; then
                cp "$sqlite_file" "$SQLITE_PATH"
                success "SQLite восстановлена: $SQLITE_PATH"
            else
                error "SQLite файл не найден"
            fi
            ;;
        
        *)
            error "Неподдерживаемый тип БД: $DB_TYPE"
            ;;
    esac
}

# ============================================================================
# Восстановление конфигурации
# ============================================================================

restore_config() {
    local config_dir="$1"
    
    info "Восстановление конфигурации..."
    
    # Копирование файлов конфигурации
    if [ -d "$config_dir" ]; then
        for file in "$config_dir"/*; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                if [ -f "$PROJECT_DIR/$filename" ]; then
                    warning "  Файл уже существует: $filename (пропущен)"
                else
                    cp "$file" "$PROJECT_DIR/"
                    info "  + $filename"
                fi
            fi
        done
        success "Конфигурация восстановлена"
    fi
}

# ============================================================================
# Очистка
# ============================================================================

cleanup() {
    local temp_dir="$1"
    
    info "Очистка временных файлов..."
    rm -rf "$temp_dir"
    
    success "Временные файлы удалены"
}

# ============================================================================
# Вывод информации
# ============================================================================

show_summary() {
    echo ""
    echo "============================================================================"
    echo "                    Восстановление завершено!"
    echo "============================================================================"
    echo ""
    echo "  База данных: $DB_NAME ($DB_TYPE)"
    echo "  Дата бэкапа: $(stat -c %y "$BACKUP_FILE" 2>/dev/null | cut -d' ' -f1 || echo 'N/A')"
    echo ""
    echo "  Для запуска Semaphore выполните:"
    echo "    ./demo-start.sh"
    echo ""
    echo "============================================================================"
}

# ============================================================================
# Основная функция
# ============================================================================

main() {
    echo ""
    echo "============================================================================"
    echo "              Velum - Restore Script"
    echo "============================================================================"
    echo ""
    
    confirm_restore
    check_dependencies
    
    temp_dir=$(extract_backup)
    
    # Найти файлы
    DB_FILE=$(find "$temp_dir" -name "*.sql" -o -name "*.sql.sqlite" | head -1)
    CONFIG_DIR=$(find "$temp_dir" -type d -name "config_backup_*" | head -1)
    
    restore_database "$DB_FILE"
    restore_config "$CONFIG_DIR"
    cleanup "$temp_dir"
    
    show_summary
}

# Запуск
main "$@"
