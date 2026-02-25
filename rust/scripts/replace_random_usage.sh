#!/bin/bash

# Скрипт для замены pkg/random на стандартный crypto/rand
# Использование: ./replace_random_usage.sh

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SEMAPHORE_ROOT="$(dirname "$PROJECT_ROOT")"

cd "$SEMAPHORE_ROOT"

echo "=============================================="
echo "🔄 Замена pkg/random на crypto/rand"
echo "=============================================="
echo ""

# Файлы для замены
FILES=(
    "api/login.go"
    "api/projects/integration_alias.go"
    "api/projects/environment.go"
    "services/server/secret_storage_svc.go"
    "services/project/restore.go"
    "services/project/backup.go"
    "services/tasks/TaskPool.go"
    "pkg/ssh/agent.go"
    ".dredd/hooks/helpers.go"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "Обработка: $file"
        
        # Замена импорта
        sed -i 's|"github.com/semaphoreui/semaphore/pkg/random"|"crypto/rand"|g' "$file"
        
        # Замена random.String(n) на генерацию через crypto/rand
        # Генерируем hex строку нужной длины
        sed -i 's/random\.String(\([0-9]*\))/func() string { b := make([]byte, \1); rand.Read(b); return fmt.Sprintf("%x", b)[:\1] }()/g' "$file"
        
        # Замена random.Number(n)
        sed -i 's/random\.Number(\([0-9]*\))/func() string { b := make([]byte, \1); rand.Read(b); return fmt.Sprintf("%d", b[:\1]) }()/g' "$file"
        
        echo "  ✅ Обработано"
    else
        echo "  ⚠️  Файл не найден: $file"
    fi
done

echo ""
echo "=============================================="
echo "✅ Замена завершена"
echo "=============================================="
echo ""
echo "📌 Следующие шаги:"
echo "   1. Проверьте изменения: git diff"
echo "   2. Запустите компиляцию: go build ./..."
echo "   3. Запустите тесты: go test ./..."
echo ""
