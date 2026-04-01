#!/bin/bash
# ============================================================================
# Velum — Postman Collection Runner
# Запуск всех API тестов из Postman коллекции
# ============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Конфигурация
BASE_URL="${BASE_URL:-http://localhost:8088}"
COLLECTION_FILE=".postman/postman/collections/Semaphore API.json"
ENV_FILE=".postman/environments/velum-demo.postman_environment.json"

echo ""
echo "============================================================================"
echo "              🚀 Velum — Postman Collection Runner"
echo "============================================================================"
echo ""

# Проверка файлов
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }

info "Проверка коллекции..."

if [ ! -f "$COLLECTION_FILE" ]; then
    error "Коллекция не найдена: $COLLECTION_FILE"
    exit 1
fi

success "Коллекция найдена: $COLLECTION_FILE"

# Проверка доступности API
info "Проверка доступности API..."

if curl -s "$BASE_URL/api/ping" > /dev/null 2>&1; then
    success "API доступно: $BASE_URL"
else
    warning "API недоступно: $BASE_URL"
    echo "Убедитесь, что сервер запущен:"
    echo "  docker compose -f docker-compose.demo.yml up"
    exit 1
fi

# Создание environment файла
info "Создание environment конфигурации..."

mkdir -p .postman/environments

cat > "$ENV_FILE" << EOF
{
    "id": "velum-demo-env",
    "name": "Velum Demo",
    "values": [
        {
            "key": "baseUrl",
            "value": "$BASE_URL/api",
            "type": "default",
            "enabled": true
        },
        {
            "key": "username",
            "value": "admin",
            "type": "default",
            "enabled": true
        },
        {
            "key": "password",
            "value": "admin123",
            "type": "default",
            "enabled": true
        }
    ],
    "_postman_variable_scope": "environment"
}
EOF

success "Environment создан: $ENV_FILE"

echo ""
echo "============================================================================"
echo "                         Запуск тестов"
echo "============================================================================"
echo ""

# Запуск Newman
newman run "$COLLECTION_FILE" \
    --environment "$ENV_FILE" \
    --reporters cli,json \
    --reporter-json-export newman-report.json \
    --delay-request 100 \
    --timeout 30000 \
    --ignore-redirects

echo ""
echo "============================================================================"
echo "                           Результаты"
echo "============================================================================"
echo ""

# Проверка результатов
if [ -f "newman-report.json" ]; then
    success "Отчёт сохранён: newman-report.json"
    
    # Парсинг результатов
    if command -v jq &> /dev/null; then
        TOTAL=$(jq '.run.stats.total' newman-report.json)
        FAILED=$(jq '.run.failures | length' newman-report.json)
        PASSED=$((TOTAL - FAILED))
        
        echo ""
        echo "📊 Статистика:"
        echo "   Всего тестов: $TOTAL"
        echo "   ✅ Успешно: $PASSED"
        echo "   ❌ Ошибок: $FAILED"
        
        if [ $FAILED -gt 0 ]; then
            echo ""
            echo "🔴 Ошибки:"
            jq -r '.run.failures[] | "   - \(.error.name): \(.error.message)"' newman-report.json
        fi
    fi
else
    warning "Отчёт не создан"
fi

echo ""
echo "============================================================================"
echo ""
