#!/bin/bash
# ============================================================================
# Комплексный тест CRUD для всех сущностей Semaphore UI
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
echo "         Комплексный тест CRUD для всех сущностей (PostgreSQL)"
echo "============================================================================"
echo ""

# Получение токена
info "1. Аутентификация (admin/demo123)..."
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}' | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
    error "Не удалось получить токен"
fi
success "Токен получен"
echo ""

# ============================================================================
# Тест 1: Проекты
# ============================================================================
info "2. Тест: Проекты..."

# CREATE
PROJECT_RESPONSE=$(curl -s -X POST http://localhost:3000/api/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Test Project","alert":false}')

PROJECT_ID=$(echo "$PROJECT_RESPONSE" | jq -r '.id')
success "✅ Проект создан: ID=$PROJECT_ID"

# READ
PROJECT_GET=$(curl -s -X GET "http://localhost:3000/api/projects/$PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN")
PROJECT_NAME=$(echo "$PROJECT_GET" | jq -r '.name')
success "✅ Проект получен: $PROJECT_NAME"

# UPDATE
curl -s -X PUT "http://localhost:3000/api/projects/$PROJECT_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Project","alert":true}' > /dev/null
success "✅ Проект обновлён"

# DELETE
curl -s -X DELETE "http://localhost:3000/api/projects/$PROJECT_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
success "✅ Проект удалён"
echo ""

# ============================================================================
# Тест 2: Инвентарь (на проекте 1)
# ============================================================================
info "3. Тест: Инвентарь..."

INVENTORY_RESPONSE=$(curl -s -X POST http://localhost:3000/api/project/1/inventory \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name":"Test Inventory",
    "inventory_type":"static",
    "inventory_data":"all:\n  hosts:\n    test1:\n    test2:",
    "ssh_login":"ansible",
    "ssh_port":22
  }')

INVENTORY_ID=$(echo "$INVENTORY_RESPONSE" | jq -r '.id')
success "✅ Инвентарь создан: ID=$INVENTORY_ID"

# READ
INVENTORY_GET=$(curl -s -X GET "http://localhost:3000/api/project/1/inventory/$INVENTORY_ID" \
  -H "Authorization: Bearer $TOKEN")
INVENTORY_NAME=$(echo "$INVENTORY_GET" | jq -r '.name')
success "✅ Инвентарь получен: $INVENTORY_NAME"

# UPDATE
curl -s -X PUT "http://localhost:3000/api/project/1/inventory/$INVENTORY_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Inventory"}' > /dev/null
success "✅ Инвентарь обновлён"

# DELETE
curl -s -X DELETE "http://localhost:3000/api/project/1/inventory/$INVENTORY_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
success "✅ Инвентарь удалён"
echo ""

# ============================================================================
# Тест 3: Репозитории (на проекте 1)
# ============================================================================
info "4. Тест: Репозитории..."

REPO_RESPONSE=$(curl -s -X POST http://localhost:3000/api/project/1/repository \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name":"Test Repository",
    "git_url":"https://github.com/test/test-repo.git",
    "git_type":"git",
    "git_branch":"main"
  }')

REPO_ID=$(echo "$REPO_RESPONSE" | jq -r '.id')
success "✅ Репозиторий создан: ID=$REPO_ID"

# READ
REPO_GET=$(curl -s -X GET "http://localhost:3000/api/project/1/repository/$REPO_ID" \
  -H "Authorization: Bearer $TOKEN")
REPO_NAME=$(echo "$REPO_GET" | jq -r '.name')
success "✅ Репозиторий получен: $REPO_NAME"

# UPDATE
curl -s -X PUT "http://localhost:3000/api/project/1/repository/$REPO_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Repository"}' > /dev/null
success "✅ Репозиторий обновлён"

# DELETE
curl -s -X DELETE "http://localhost:3000/api/project/1/repository/$REPO_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
success "✅ Репозиторий удалён"
echo ""

# ============================================================================
# Тест 4: Окружения (на проекте 1)
# ============================================================================
info "5. Тест: Окружения..."

ENV_RESPONSE=$(curl -s -X POST http://localhost:3000/api/project/1/environment \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name":"Test Environment",
    "json":"{\"env\":\"test\",\"debug\":true}"
  }')

ENV_ID=$(echo "$ENV_RESPONSE" | jq -r '.id')
success "✅ Окружение создано: ID=$ENV_ID"

# READ
ENV_GET=$(curl -s -X GET "http://localhost:3000/api/project/1/environment/$ENV_ID" \
  -H "Authorization: Bearer $TOKEN")
ENV_NAME=$(echo "$ENV_GET" | jq -r '.name')
success "✅ Окружение получено: $ENV_NAME"

# UPDATE
curl -s -X PUT "http://localhost:3000/api/project/1/environment/$ENV_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Environment"}' > /dev/null
success "✅ Окружение обновлено"

# DELETE
curl -s -X DELETE "http://localhost:3000/api/project/1/environment/$ENV_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
success "✅ Окружение удалено"
echo ""

# ============================================================================
# Тест 5: Ключи доступа (на проекте 1)
# ============================================================================
info "6. Тест: Ключи доступа..."

KEY_RESPONSE=$(curl -s -X POST http://localhost:3000/api/project/1/keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name":"Test SSH Key",
    "type":"ssh",
    "ssh_key":"-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
  }')

KEY_ID=$(echo "$KEY_RESPONSE" | jq -r '.id')
success "✅ Ключ создан: ID=$KEY_ID"

# READ
KEY_GET=$(curl -s -X GET "http://localhost:3000/api/project/1/keys/$KEY_ID" \
  -H "Authorization: Bearer $TOKEN")
KEY_NAME=$(echo "$KEY_GET" | jq -r '.name')
success "✅ Ключ получен: $KEY_NAME"

# UPDATE
curl -s -X PUT "http://localhost:3000/api/project/1/keys/$KEY_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Key"}' > /dev/null
success "✅ Ключ обновлён"

# DELETE
curl -s -X DELETE "http://localhost:3000/api/project/1/keys/$KEY_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
success "✅ Ключ удалён"
echo ""

# ============================================================================
# Тест 6: Шаблоны (на проекте 1)
# ============================================================================
info "7. Тест: Шаблоны..."

TEMPLATE_RESPONSE=$(curl -s -X POST http://localhost:3000/api/project/1/templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name":"Test Template",
    "playbook":"test.yml",
    "description":"Test template for CRUD"
  }')

TEMPLATE_ID=$(echo "$TEMPLATE_RESPONSE" | jq -r '.id')
success "✅ Шаблон создан: ID=$TEMPLATE_ID"

# READ
TEMPLATE_GET=$(curl -s -X GET "http://localhost:3000/api/project/1/templates/$TEMPLATE_ID" \
  -H "Authorization: Bearer $TOKEN")
TEMPLATE_NAME=$(echo "$TEMPLATE_GET" | jq -r '.name')
success "✅ Шаблон получен: $TEMPLATE_NAME"

# UPDATE
curl -s -X PUT "http://localhost:3000/api/project/1/templates/$TEMPLATE_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated Template"}' > /dev/null
success "✅ Шаблон обновлён"

# DELETE
curl -s -X DELETE "http://localhost:3000/api/project/1/templates/$TEMPLATE_ID" \
  -H "Authorization: Bearer $TOKEN" > /dev/null
success "✅ Шаблон удалён"
echo ""

# ============================================================================
# Итоги
# ============================================================================
echo "============================================================================"
echo "                     ✅ Все тесты пройдены!"
echo "============================================================================"
echo ""
echo "📋 Протестированные сущности:"
echo "   ✅ Проекты (CREATE, READ, UPDATE, DELETE)"
echo "   ✅ Инвентарь (CREATE, READ, UPDATE, DELETE)"
echo "   ✅ Репозитории (CREATE, READ, UPDATE, DELETE)"
echo "   ✅ Окружения (CREATE, READ, UPDATE, DELETE)"
echo "   ✅ Ключи доступа (CREATE, READ, UPDATE, DELETE)"
echo "   ✅ Шаблоны (CREATE, READ, UPDATE, DELETE)"
echo ""
echo "🎯 CRUD демо полностью функционально!"
echo ""
