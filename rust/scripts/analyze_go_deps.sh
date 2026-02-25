#!/bin/bash

# Скрипт для анализа зависимостей Go модулей
# Использование: ./analyze_go_deps.sh [модуль]

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SEMAPHORE_ROOT="$(dirname "$PROJECT_ROOT")"

cd "$SEMAPHORE_ROOT"

echo "🔍 Анализ зависимостей Go модулей Semaphore UI"
echo "=============================================="
echo ""

# Функция для анализа модуля
analyze_module() {
    local module=$1
    local pattern=$2
    
    echo "📦 Модуль: $module"
    echo "   Паттерн: $pattern"
    echo "   Файлы:"
    
    local count=0
    while IFS= read -r file; do
        if [ -n "$file" ]; then
            echo "     - $file"
            ((count++)) || true
        fi
    done < <(grep -rl "$pattern" --include="*.go" . 2>/dev/null | head -30 | sed 's|^\./||')
    
    if [ $count -eq 0 ]; then
        echo "     (нет зависимостей)"
    else
        echo "   Всего файлов: $count"
    fi
    echo ""
}

# Если передан конкретный модуль
if [ -n "$1" ]; then
    case $1 in
        tz)
            analyze_module "pkg/tz" "pkg/tz"
            ;;
        random)
            analyze_module "pkg/random" "pkg/random"
            ;;
        conv)
            analyze_module "pkg/conv" "pkg/conv"
            ;;
        common_errors)
            analyze_module "pkg/common_errors" "common_errors"
            ;;
        task_logger)
            analyze_module "pkg/task_logger" "pkg/task_logger"
            ;;
        ssh)
            analyze_module "pkg/ssh" "pkg/ssh"
            ;;
        all)
            analyze_module "pkg/tz" "pkg/tz"
            analyze_module "pkg/random" "pkg/random"
            analyze_module "pkg/conv" "pkg/conv"
            analyze_module "pkg/common_errors" "common_errors"
            analyze_module "pkg/task_logger" "pkg/task_logger"
            analyze_module "pkg/ssh" "pkg/ssh"
            ;;
        *)
            echo "❌ Неизвестный модуль: $1"
            echo "Доступные модули: tz, random, conv, common_errors, task_logger, ssh, all"
            exit 1
            ;;
    esac
else
    # Анализ всех модулей
    echo "📊 Полный анализ всех модулей pkg/"
    echo ""
    
    analyze_module "pkg/tz" "pkg/tz"
    analyze_module "pkg/random" "pkg/random"
    analyze_module "pkg/conv" "pkg/conv"
    analyze_module "pkg/common_errors" "common_errors"
    analyze_module "pkg/task_logger" "pkg/task_logger"
    analyze_module "pkg/ssh" "pkg/ssh"
fi

echo "=============================================="
echo "✅ Анализ завершён"
