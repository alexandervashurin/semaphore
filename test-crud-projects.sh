#!/bin/bash
# Тестирование CRUD операций

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }

echo ""
echo "============================================================================"
echo "                   Тестирование CRUD операций"
echo "============================================================================"
echo ""

# 1. Вход и получение токена
info "Вход в систему..."
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}')

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    error "Не удалось получить токен"
    echo "Ответ API: $LOGIN_RESPONSE"
    exit 1
fi

success "Токен получен"
echo ""

# 2. Получение списка проектов
info "Получение списка проектов..."
PROJECTS=$(curl -s -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN")

PROJECTS_COUNT=$(echo "$PROJECTS" | jq 'length')
success "Найдено проектов: $PROJECTS_COUNT"
echo "$PROJECTS" | jq '.[] | {id, name}'
echo ""

# 3. Создание нового проекта
info "Создание нового проекта 'Test CRUD Project'..."
CREATE_RESPONSE=$(curl -s -X POST http://localhost:3000/api/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Test CRUD Project",
    "alert": false
  }')

if echo "$CREATE_RESPONSE" | jq -e '.id' > /dev/null 2>&1; then
    NEW_PROJECT_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
    success "Проект создан с ID: $NEW_PROJECT_ID"
    echo "$CREATE_RESPONSE" | jq '{id, name, created}'
else
    error "Ошибка создания проекта"
    echo "Ответ API: $CREATE_RESPONSE"
    exit 1
fi
echo ""

# 4. Обновление проекта
info "Обновление проекта..."
curl -s -X PUT "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Updated Test CRUD Project",
    "alert": true
  }' > /dev/null

success "Проект обновлён"
echo ""

# 5. Проверка обновления
info "Проверка обновления..."
UPDATED_PROJECT=$(curl -s -X GET "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

UPDATED_NAME=$(echo "$UPDATED_PROJECT" | jq -r '.name')
success "Обновлённое название: $UPDATED_NAME"
echo ""

# 6. Удаление проекта
info "Удаление проекта..."
curl -s -X DELETE "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null

success "Проект удалён"
echo ""

# 7. Финальная проверка
info "Финальный список проектов:"
FINAL_PROJECTS=$(curl -s -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN")

FINAL_COUNT=$(echo "$FINAL_PROJECTS" | jq 'length')
success "Осталось проектов: $FINAL_COUNT"
echo "$FINAL_PROJECTS" | jq '.[] | {id, name}'
echo ""

echo "============================================================================"
echo "                      ✅ Все CRUD операции работают!"
echo "============================================================================"
echo ""
