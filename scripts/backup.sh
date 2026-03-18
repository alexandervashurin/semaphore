#!/bin/bash
# ============================================================================
# Velum - Backup Script
# ============================================================================
# Автоматическое создание резервных копий базы данных и конфигурации
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BACKUP_DIR="${SEMAPHORE_BACKUP_DIR:-$PROJECT_DIR/backups}"
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
BACKUP_FILE="semaphore_backup_${TIMESTAMP}.tar.gz"

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
        deps+=("pg_dump")
    elif [[ "$DB_TYPE" == "mysql" ]]; then
        deps+=("mysqldump")
    fi
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            error "Зависимость не найдена: $dep"
        fi
    done
    
    success "Все зависимости найдены"
}

# ============================================================================
# Создание директории для бэкапов
# ============================================================================

create_backup_dir() {
    if [ ! -d "$BACKUP_DIR" ]; then
        info "Создание директории для бэкапов: $BACKUP_DIR"
        mkdir -p "$BACKUP_DIR"
    fi
}

# ============================================================================
# Бэкап базы данных
# ============================================================================

backup_database() {
    local temp_db_file="$SCRIPT_DIR/db_backup_${TIMESTAMP}.sql"
    
    info "Бэкап базы данных ($DB_TYPE)..."
    
    case "$DB_TYPE" in
        postgres)
            pg_dump \
                -h "$DB_HOST" \
                -p "$DB_PORT" \
                -U "$DB_USER" \
                -d "$DB_NAME" \
                --no-owner \
                --no-privileges \
                -F p \
                -f "$temp_db_file"
            success "PostgreSQL бэкап создан: $temp_db_file"
            ;;
        
        mysql)
            mysqldump \
                -h "$DB_HOST" \
                -P "$DB_PORT" \
                -u "$DB_USER" \
                -p"$DB_PASS" \
                --no-tablespaces \
                "$DB_NAME" \
                > "$temp_db_file"
            success "MySQL бэкап создан: $temp_db_file"
            ;;
        
        sqlite)
            if [ -f "$SQLITE_PATH" ]; then
                cp "$SQLITE_PATH" "$temp_db_file.sqlite"
                success "SQLite бэкап создан: $temp_db_file.sqlite"
            else
                error "SQLite база не найдена: $SQLITE_PATH"
            fi
            ;;
        
        *)
            error "Неподдерживаемый тип БД: $DB_TYPE"
            ;;
    esac
    
    echo "$temp_db_file"
}

# ============================================================================
# Бэкап конфигурации
# ============================================================================

backup_config() {
    local config_dir="$SCRIPT_DIR/config_backup_${TIMESTAMP}"
    mkdir -p "$config_dir"
    
    info "Бэкап конфигурации..."
    
    # Копирование файлов конфигурации
    local config_files=(
        ".env.example"
        "docker-compose.yml"
        "nginx.conf"
    )
    
    for file in "${config_files[@]}"; do
        if [ -f "$PROJECT_DIR/$file" ]; then
            cp "$PROJECT_DIR/$file" "$config_dir/"
            info "  + $file"
        fi
    done
    
    # Копирование директорий
    local config_dirs=(
        "db/postgres"
        "deployment"
    )
    
    for dir in "${config_dirs[@]}"; do
        if [ -d "$PROJECT_DIR/$dir" ]; then
            cp -r "$PROJECT_DIR/$dir" "$config_dir/"
            info "  + $dir/"
        fi
    done
    
    success "Конфигурация скопирована: $config_dir"
    echo "$config_dir"
}

# ============================================================================
# Создание архива
# ============================================================================

create_archive() {
    local db_file="$1"
    local config_dir="$2"
    local archive_path="$BACKUP_DIR/$BACKUP_FILE"
    
    info "Создание архива..."
    
    tar -czf "$archive_path" \
        -C "$SCRIPT_DIR" \
        "$(basename "$db_file")" \
        -C "$(dirname "$config_dir")" \
        "$(basename "$config_dir")"
    
    success "Архив создан: $archive_path"
    
    # Очистка временных файлов
    rm -f "$db_file" "${db_file}.sqlite"
    rm -rf "$config_dir"
    
    # Размер архива
    local size=$(du -h "$archive_path" | cut -f1)
    info "Размер архива: $size"
    
    echo "$archive_path"
}

# ============================================================================
# Очистка старых бэкапов
# ============================================================================

cleanup_old_backups() {
    local retention_days="${SEMAPHORE_BACKUP_RETENTION_DAYS:-7}"
    
    info "Очистка бэкапов старше $retention_days дней..."
    
    find "$BACKUP_DIR" -name "semaphore_backup_*.tar.gz" -mtime +$retention_days -delete 2>/dev/null || true
    
    local remaining=$(ls -1 "$BACKUP_DIR"/semaphore_backup_*.tar.gz 2>/dev/null | wc -l)
    success "Осталось бэкапов: $remaining"
}

# ============================================================================
# Вывод информации
# ============================================================================

show_summary() {
    local archive_path="$1"
    
    echo ""
    echo "============================================================================"
    echo "                    Backup завершен успешно!"
    echo "============================================================================"
    echo ""
    echo "  Файл бэкапа: $archive_path"
    echo "  Размер: $(du -h "$archive_path" | cut -f1)"
    echo "  Дата: $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""
    echo "  Для восстановления используйте:"
    echo "    ./scripts/restore.sh $archive_path"
    echo ""
    echo "============================================================================"
}

# ============================================================================
# Основная функция
# ============================================================================

main() {
    echo ""
    echo "============================================================================"
    echo "              Velum - Backup Script"
    echo "============================================================================"
    echo ""
    
    check_dependencies
    create_backup_dir
    
    db_file=$(backup_database)
    config_dir=$(backup_config)
    archive_path=$(create_archive "$db_file" "$config_dir")
    
    cleanup_old_backups
    
    show_summary "$archive_path"
}

# Запуск
main "$@"
