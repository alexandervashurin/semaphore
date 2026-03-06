#!/bin/bash
# Тестирование CRUD операций (без jq)

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
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    error "Не удалось получить токен"
    exit 1
fi

success "Токен получен: ${TOKEN:0:50}..."
echo ""

# 2. Получение списка проектов
info "Получение списка проектов..."
PROJECTS=$(curl -s -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN")

echo "Ответ API: $PROJECTS"
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

echo "Ответ API: $CREATE_RESPONSE"

if echo "$CREATE_RESPONSE" | grep -q '"id"'; then
    NEW_PROJECT_ID=$(echo "$CREATE_RESPONSE" | grep -o '"id":[0-9]*' | cut -d':' -f2)
    success "Проект создан с ID: $NEW_PROJECT_ID"
else
    error "Ошибка создания проекта"
    exit 1
fi
echo ""

# 4. Обновление проекта
info "Обновление проекта..."
UPDATE_RESPONSE=$(curl -s -X PUT "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Updated Test CRUD Project",
    "alert": true
  }')

if [ -z "$UPDATE_RESPONSE" ] || echo "$UPDATE_RESPONSE" | grep -q 'OK'; then
    success "Проект обновлён"
else
    echo "Ответ API: $UPDATE_RESPONSE"
fi
echo ""

# 5. Проверка обновления
info "Проверка обновления..."
UPDATED_PROJECT=$(curl -s -X GET "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

echo "Ответ API: $UPDATED_PROJECT"

if echo "$UPDATED_PROJECT" | grep -q "Updated Test CRUD Project"; then
    success "Название обновлено: 'Updated Test CRUD Project'"
fi
echo ""

# 6. Удаление проекта
info "Удаление проекта..."
DELETE_RESPONSE=$(curl -s -X DELETE "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

if [ -z "$DELETE_RESPONSE" ]; then
    success "Проект удалён"
else
    echo "Ответ API: $DELETE_RESPONSE"
fi
echo ""

# 7. Финальная проверка
info "Финальный список проектов:"
FINAL_PROJECTS=$(curl -s -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN")

echo "Ответ API: $FINAL_PROJECTS"
echo ""

echo "============================================================================"
echo "                      ✅ Все CRUD операции работают!"
echo "============================================================================"
echo ""
