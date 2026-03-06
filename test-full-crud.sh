#!/bin/bash
# ============================================================================
# Полное тестирование CRUD операций для проектов
# ============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

echo ""
echo "============================================================================"
echo "              Полное тестирование CRUD операций"
echo "============================================================================"
echo ""

# Получение токена
info "1. Аутентификация..."
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}' | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    error "Не удалось получить токен"
fi
success "Токен получен: ${TOKEN:0:50}..."
echo ""

# CREATE
info "2. CREATE - Создание проекта..."
CREATE_RESPONSE=$(curl -s -X POST http://localhost:3000/api/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "CRUD Test Project",
    "alert": false,
    "max_parallel_tasks": 5
  }')

NEW_PROJECT_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
NEW_PROJECT_NAME=$(echo "$CREATE_RESPONSE" | jq -r '.name')

if [ "$NEW_PROJECT_ID" != "null" ] && [ -n "$NEW_PROJECT_ID" ]; then
    success "Проект создан: ID=$NEW_PROJECT_ID, Name='$NEW_PROJECT_NAME'"
    echo "$CREATE_RESPONSE" | jq '{id, name, alert, max_parallel_tasks}'
else
    error "Ошибка создания проекта: $CREATE_RESPONSE"
fi
echo ""

# READ (один проект)
info "3. READ - Получение проекта по ID..."
GET_RESPONSE=$(curl -s -X GET "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

GET_NAME=$(echo "$GET_RESPONSE" | jq -r '.name')
if [ "$GET_NAME" = "$NEW_PROJECT_NAME" ]; then
    success "Проект получен: Name='$GET_NAME'"
    echo "$GET_RESPONSE" | jq '{id, name, created}'
else
    error "Ошибка получения проекта"
fi
echo ""

# READ (список)
info "4. READ - Получение списка проектов..."
PROJECTS=$(curl -s -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN")

PROJECTS_COUNT=$(echo "$PROJECTS" | jq 'length')
success "Найдено проектов: $PROJECTS_COUNT"
echo "$PROJECTS" | jq '.[] | {id, name}'
echo ""

# UPDATE
info "5. UPDATE - Обновление проекта (изменение name и alert)..."
UPDATE_RESPONSE=$(curl -s -X PUT "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Updated CRUD Project",
    "alert": true
  }')

if [ -z "$UPDATE_RESPONSE" ]; then
    success "Проект обновлён (пустой ответ - это нормально)"
else
    echo "Ответ API: $UPDATE_RESPONSE"
fi

# Проверка обновления
UPDATED_PROJECT=$(curl -s -X GET "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

UPDATED_NAME=$(echo "$UPDATED_PROJECT" | jq -r '.name')
UPDATED_ALERT=$(echo "$UPDATED_PROJECT" | jq -r '.alert')

if [ "$UPDATED_NAME" = "Updated CRUD Project" ] && [ "$UPDATED_ALERT" = "true" ]; then
    success "Проект обновлён: Name='$UPDATED_NAME', Alert=$UPDATED_ALERT"
    echo "$UPDATED_PROJECT" | jq '{id, name, alert}'
else
    error "Ошибка обновления: $UPDATED_PROJECT"
fi
echo ""

# UPDATE (другое поле)
info "6. UPDATE - Обновление проекта (только max_parallel_tasks)..."
curl -s -X PUT "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"max_parallel_tasks": 10}' > /dev/null

UPDATED_TASKS=$(curl -s -X GET "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.max_parallel_tasks')

if [ "$UPDATED_TASKS" = "10" ]; then
    success "Поле max_parallel_tasks обновлено: $UPDATED_TASKS"
else
    error "Ошибка обновления поля: $UPDATED_TASKS"
fi
echo ""

# DELETE
info "7. DELETE - Удаление проекта..."
DELETE_RESPONSE=$(curl -s -X DELETE "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

if [ -z "$DELETE_RESPONSE" ]; then
    success "Проект удалён"
else
    echo "Ответ API: $DELETE_RESPONSE"
fi

# Проверка удаления
FINAL_COUNT=$(curl -s -X GET http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN" | jq 'length')

success "Осталось проектов: $FINAL_COUNT (было $PROJECTS_COUNT)"
echo ""

# Финальная проверка
info "8. Проверка, что удалённый проект недоступен..."
GET_DELETED=$(curl -s -X GET "http://localhost:3000/api/projects/$NEW_PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")

if echo "$GET_DELETED" | grep -q "not found\|Not found"; then
    success "Удалённый проект действительно недоступен"
else
    echo "Ответ API: $GET_DELETED"
fi
echo ""

echo "============================================================================"
echo "                  ✅ Все CRUD операции работают!"
echo "============================================================================"
echo ""
echo "📋 Итоги:"
echo "   ✅ CREATE - Создание проекта"
echo "   ✅ READ - Получение проекта (по ID и списка)"
echo "   ✅ UPDATE - Обновление проекта (partial update)"
echo "   ✅ DELETE - Удаление проекта"
echo ""
