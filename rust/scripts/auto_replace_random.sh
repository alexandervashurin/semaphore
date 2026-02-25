#!/bin/bash

# Автоматическая замена pkg/random на crypto/rand + hex

set -e

SEMAPHORE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$SEMAPHORE_ROOT"

echo "=============================================="
echo "🔄 Автоматическая замена pkg/random"
echo "=============================================="
echo ""

FILES=(
    "services/server/secret_storage_svc.go"
    "services/project/restore.go"
    "services/project/backup.go"
    "services/tasks/TaskPool.go"
    ".dredd/hooks/helpers.go"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "Обработка: $file"
        
        # Добавляем импорты после package
        if ! grep -q "crypto/rand" "$file"; then
            sed -i '/^package/a\\nimport (\n\t"crypto/rand"\n\t"encoding/hex"\n)' "$file"
        fi
        
        # Удаляем импорт pkg/random
        sed -i '/"github.com\/semaphoreui\/semaphore\/pkg\/random"/d' "$file"
        
        # Заменяем random.String(n) на hex.EncodeToString(make([]byte, n))[:n]
        # Это простая замена, может потребовать ручной доработки
        sed -i 's/random\.String(\([0-9]*\))/func() string { b := make([]byte, \1); rand.Read(b); return hex.EncodeToString(b)[:\1] }()/g' "$file"
        
        echo "  ✅ Обработано"
    else
        echo "  ⚠️  Файл не найден: $file"
    fi
done

echo ""
echo "=============================================="
echo "⚠️  Требуется ручная проверка!"
echo "=============================================="
echo ""
echo "Проверьте изменения: git diff"
echo ""
