#!/bin/bash

# ============================================================================
# Скрипт проверки CRUD операций Velum
# ============================================================================
# Тестирует Create, Read, Update, Delete для всех сущностей
# ============================================================================

set -e

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Настройки
API_URL="${API_URL:-http://localhost:3000/api}"
USERNAME="${USERNAME:-admin}"
PASSWORD="${PASSWORD:-demo123}"

# Счётчики
TESTS_PASSED=0
TESTS_FAILED=0

# ============================================================================
# Функции
# ============================================================================

info() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

pass() {
    echo -e "${GREEN}✓ PASS${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}✗ FAIL${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

section() {
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
    echo ""
}

# ============================================================================
# Авторизация
# ============================================================================

section "Авторизация"

info "Получение токена..."

LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

# Парсинг токена через sed
TOKEN=$(echo "$LOGIN_RESPONSE" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')

if [ -z "$TOKEN" ]; then
    fail "Авторизация не удалась"
    echo "Ответ: $LOGIN_RESPONSE"
    exit 1
fi

pass "Авторизация успешна"
AUTH_HEADER="Authorization: Bearer $TOKEN"

# ============================================================================
# Тесты CRUD для проектов
# ============================================================================

section "CRUD: Проекты"

# CREATE
info "CREATE: Создание проекта..."
PROJECT_CREATE=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Project",
        "alert": false,
        "max_parallel_tasks": 2,
        "type": "default"
    }')

PROJECT_ID=$(echo "$PROJECT_CREATE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -n "$PROJECT_ID" ] && [ "$PROJECT_ID" != "null" ]; then
    pass "Проект создан (ID: $PROJECT_ID)"
else
    fail "Не удалось создать проект"
    echo "Ответ: $PROJECT_CREATE"
fi

# READ (список)
info "READ: Получение списка проектов..."
PROJECTS=$(curl -s "$API_URL/projects" -H "$AUTH_HEADER")

if echo "$PROJECTS" | grep -q "CRUD Test Project"; then
    pass "Проект найден в списке"
else
    fail "Проект не найден в списке"
fi

# READ (один)
info "READ: Получение проекта по ID..."
PROJECT=$(curl -s "$API_URL/projects/$PROJECT_ID" -H "$AUTH_HEADER")

if echo "$PROJECT" | grep -q "CRUD Test Project"; then
    pass "Проект получен по ID"
else
    fail "Не удалось получить проект по ID"
fi

# UPDATE
info "UPDATE: Обновление проекта..."
PROJECT_UPDATE=$(curl -s -X PUT "$API_URL/projects/$PROJECT_ID" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Project - Updated",
        "alert": true,
        "max_parallel_tasks": 5,
        "type": "default",
        "alert_chat": null,
        "default_secret_storage_id": null
    }')

if echo "$PROJECT_UPDATE" | grep -q "Updated"; then
    pass "Проект обновлён"
else
    fail "Не удалось обновить проект"
    echo "Ответ: $PROJECT_UPDATE"
fi

# DELETE
info "DELETE: Удаление проекта..."
DELETE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$API_URL/projects/$PROJECT_ID" -H "$AUTH_HEADER")

if [ "$DELETE_STATUS" = "204" ]; then
    pass "Проект удалён"
else
    fail "Не удалось удалить проект (HTTP: $DELETE_STATUS)"
fi

# ============================================================================
# Тесты CRUD для SSH ключей
# ============================================================================

section "CRUD: Ключи доступа"

# Сначала создадим проект для ключа
info "Создание проекта для ключа..."
TEST_PROJECT=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Key Test Project",
        "alert": false,
        "max_parallel_tasks": 1,
        "type": "default"
    }')
TEST_PROJECT_ID=$(echo "$TEST_PROJECT" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

# CREATE
info "CREATE: Создание SSH ключа..."
KEY_CREATE=$(curl -s -X POST "$API_URL/projects/$TEST_PROJECT_ID/keys" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Key",
        "type": "ssh",
        "ssh_key": "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----",
        "owner": "project"
    }')

KEY_ID=$(echo "$KEY_CREATE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -n "$KEY_ID" ]; then
    pass "SSH ключ создан (ID: $KEY_ID)"
else
    fail "Не удалось создать SSH ключ"
fi

# READ
info "READ: Получение списка ключей..."
KEYS=$(curl -s "$API_URL/projects/$TEST_PROJECT_ID/keys" -H "$AUTH_HEADER")

if echo "$KEYS" | grep -q "CRUD Test Key"; then
    pass "SSH ключ найден в списке"
else
    fail "SSH ключ не найден"
fi

# UPDATE
info "UPDATE: Обновление SSH ключа..."
KEY_UPDATE=$(curl -s -X PUT "$API_URL/projects/$TEST_PROJECT_ID/keys/$KEY_ID" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Key - Updated",
        "type": "ssh",
        "ssh_key": "-----BEGIN OPENSSH PRIVATE KEY-----\nupdated\n-----END OPENSSH PRIVATE KEY-----",
        "owner": "project"
    }')

if echo "$KEY_UPDATE" | grep -q "Updated"; then
    pass "SSH ключ обновлён"
else
    fail "Не удалось обновить SSH ключ"
fi

# DELETE
info "DELETE: Удаление SSH ключа..."
KEY_DELETE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$API_URL/projects/$TEST_PROJECT_ID/keys/$KEY_ID" -H "$AUTH_HEADER")

if [ "$KEY_DELETE_STATUS" = "204" ]; then
    pass "SSH ключ удалён"
else
    fail "Не удалось удалить SSH ключ (HTTP: $KEY_DELETE_STATUS)"
fi

# Очистка тестового проекта
curl -s -X DELETE "$API_URL/projects/$TEST_PROJECT_ID" -H "$AUTH_HEADER" > /dev/null 2>&1

# ============================================================================
# Тесты CRUD для Инвентаря
# ============================================================================

section "CRUD: Инвентарь"

# Создаём проект
info "Создание проекта для инвентаря..."
INV_PROJECT=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Inventory Test Project",
        "alert": false,
        "max_parallel_tasks": 1,
        "type": "default"
    }')
INV_PROJECT_ID=$(echo "$INV_PROJECT" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

# CREATE
info "CREATE: Создание инвентаря..."
INV_CREATE=$(curl -s -X POST "$API_URL/projects/$INV_PROJECT_ID/inventories" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Inventory",
        "inventory_type": "static",
        "inventory_data": "[test]\nhost1 ansible_host=10.0.0.1",
        "ssh_login": "root",
        "ssh_port": 22
    }')

INV_ID=$(echo "$INV_CREATE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -n "$INV_ID" ]; then
    pass "Инвентарь создан (ID: $INV_ID)"
else
    fail "Не удалось создать инвентарь"
fi

# READ
info "READ: Получение списка инвентарей..."
INVS=$(curl -s "$API_URL/projects/$INV_PROJECT_ID/inventories" -H "$AUTH_HEADER")

if echo "$INVS" | grep -q "CRUD Test Inventory"; then
    pass "Инвентарь найден в списке"
else
    fail "Инвентарь не найден"
fi

# UPDATE
info "UPDATE: Обновление инвентаря..."
INV_UPDATE=$(curl -s -X PUT "$API_URL/projects/$INV_PROJECT_ID/inventories/$INV_ID" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Inventory - Updated",
        "inventory_type": "static",
        "inventory_data": "[updated]\nhost2 ansible_host=10.0.0.2",
        "ssh_login": "ansible",
        "ssh_port": 22
    }')

if echo "$INV_UPDATE" | grep -q "Updated"; then
    pass "Инвентарь обновлён"
else
    fail "Не удалось обновить инвентарь"
fi

# DELETE
info "DELETE: Удаление инвентаря..."
INV_DELETE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$API_URL/projects/$INV_PROJECT_ID/inventories/$INV_ID" -H "$AUTH_HEADER")

if [ "$INV_DELETE_STATUS" = "204" ]; then
    pass "Инвентарь удалён"
else
    fail "Не удалось удалить инвентарь (HTTP: $INV_DELETE_STATUS)"
fi

# Очистка
curl -s -X DELETE "$API_URL/projects/$INV_PROJECT_ID" -H "$AUTH_HEADER" > /dev/null 2>&1

# ============================================================================
# Тесты CRUD для Репозиториев
# ============================================================================

section "CRUD: Репозитории"

# Создаём проект
info "Создание проекта для репозитория..."
REPO_PROJECT=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Repository Test Project",
        "alert": false,
        "max_parallel_tasks": 1,
        "type": "default"
    }')
REPO_PROJECT_ID=$(echo "$REPO_PROJECT" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

# CREATE
info "CREATE: Создание репозитория..."
REPO_CREATE=$(curl -s -X POST "$API_URL/projects/$REPO_PROJECT_ID/repositories" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Repository",
        "git_url": "https://github.com/test/test.git",
        "git_type": "git",
        "git_branch": "main"
    }')

REPO_ID=$(echo "$REPO_CREATE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -n "$REPO_ID" ]; then
    pass "Репозиторий создан (ID: $REPO_ID)"
else
    fail "Не удалось создать репозиторий"
fi

# READ
info "READ: Получение списка репозиториев..."
REPOS=$(curl -s "$API_URL/projects/$REPO_PROJECT_ID/repositories" -H "$AUTH_HEADER")

if echo "$REPOS" | grep -q "CRUD Test Repository"; then
    pass "Репозиторий найден в списке"
else
    fail "Репозиторий не найден"
fi

# UPDATE
info "UPDATE: Обновление репозитория..."
REPO_UPDATE=$(curl -s -X PUT "$API_URL/projects/$REPO_PROJECT_ID/repositories/$REPO_ID" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Repository - Updated",
        "git_url": "https://github.com/updated/updated.git",
        "git_type": "git",
        "git_branch": "develop"
    }')

if echo "$REPO_UPDATE" | grep -q "Updated"; then
    pass "Репозиторий обновлён"
else
    fail "Не удалось обновить репозиторий"
fi

# DELETE
info "DELETE: Удаление репозитория..."
REPO_DELETE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$API_URL/projects/$REPO_PROJECT_ID/repositories/$REPO_ID" -H "$AUTH_HEADER")

if [ "$REPO_DELETE_STATUS" = "204" ]; then
    pass "Репозиторий удалён"
else
    fail "Не удалось удалить репозиторий (HTTP: $REPO_DELETE_STATUS)"
fi

# Очистка
curl -s -X DELETE "$API_URL/projects/$REPO_PROJECT_ID" -H "$AUTH_HEADER" > /dev/null 2>&1

# ============================================================================
# Тесты CRUD для Окружений
# ============================================================================

section "CRUD: Окружения"

# Создаём проект
info "Создание проекта для окружения..."
ENV_PROJECT=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Environment Test Project",
        "alert": false,
        "max_parallel_tasks": 1,
        "type": "default"
    }')
ENV_PROJECT_ID=$(echo "$ENV_PROJECT" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

# CREATE
info "CREATE: Создание окружения..."
ENV_CREATE=$(curl -s -X POST "$API_URL/projects/$ENV_PROJECT_ID/environments" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Environment",
        "json": "{\"test\": true}"
    }')

ENV_ID=$(echo "$ENV_CREATE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -n "$ENV_ID" ]; then
    pass "Окружение создано (ID: $ENV_ID)"
else
    fail "Не удалось создать окружение"
fi

# READ
info "READ: Получение списка окружений..."
ENVS=$(curl -s "$API_URL/projects/$ENV_PROJECT_ID/environments" -H "$AUTH_HEADER")

if echo "$ENVS" | grep -q "CRUD Test Environment"; then
    pass "Окружение найдено в списке"
else
    fail "Окружение не найдено"
fi

# UPDATE
info "UPDATE: Обновление окружения..."
ENV_UPDATE=$(curl -s -X PUT "$API_URL/projects/$ENV_PROJECT_ID/environments/$ENV_ID" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Environment - Updated",
        "json": "{\"updated\": true, \"value\": 123}"
    }')

if echo "$ENV_UPDATE" | grep -q "Updated"; then
    pass "Окружение обновлено"
else
    fail "Не удалось обновить окружение"
fi

# DELETE
info "DELETE: Удаление окружения..."
ENV_DELETE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$API_URL/projects/$ENV_PROJECT_ID/environments/$ENV_ID" -H "$AUTH_HEADER")

if [ "$ENV_DELETE_STATUS" = "204" ]; then
    pass "Окружение удалено"
else
    fail "Не удалось удалить окружение (HTTP: $ENV_DELETE_STATUS)"
fi

# Очистка
curl -s -X DELETE "$API_URL/projects/$ENV_PROJECT_ID" -H "$AUTH_HEADER" > /dev/null 2>&1

# ============================================================================
# Тесты CRUD для Шаблонов
# ============================================================================

section "CRUD: Шаблоны"

# Создаём проект
info "Создание проекта для шаблона..."
TPL_PROJECT=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Template Test Project",
        "alert": false,
        "max_parallel_tasks": 1,
        "type": "default"
    }')
TPL_PROJECT_ID=$(echo "$TPL_PROJECT" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

# CREATE
info "CREATE: Создание шаблона..."
TPL_CREATE=$(curl -s -X POST "$API_URL/projects/$TPL_PROJECT_ID/templates" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Template",
        "playbook": "test.yml",
        "description": "Test template",
        "type": "ansible",
        "app": "ansible"
    }')

TPL_ID=$(echo "$TPL_CREATE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -n "$TPL_ID" ]; then
    pass "Шаблон создан (ID: $TPL_ID)"
else
    fail "Не удалось создать шаблон"
fi

# READ
info "READ: Получение списка шаблонов..."
TPLS=$(curl -s "$API_URL/projects/$TPL_PROJECT_ID/templates" -H "$AUTH_HEADER")

if echo "$TPLS" | grep -q "CRUD Test Template"; then
    pass "Шаблон найден в списке"
else
    fail "Шаблон не найден"
fi

# UPDATE
info "UPDATE: Обновление шаблона..."
TPL_UPDATE=$(curl -s -X PUT "$API_URL/projects/$TPL_PROJECT_ID/templates/$TPL_ID" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "CRUD Test Template - Updated",
        "playbook": "updated.yml",
        "description": "Updated template",
        "type": "ansible",
        "app": "ansible"
    }')

if echo "$TPL_UPDATE" | grep -q "Updated"; then
    pass "Шаблон обновлён"
else
    fail "Не удалось обновить шаблон"
fi

# DELETE
info "DELETE: Удаление шаблона..."
TPL_DELETE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$API_URL/projects/$TPL_PROJECT_ID/templates/$TPL_ID" -H "$AUTH_HEADER")

if [ "$TPL_DELETE_STATUS" = "204" ]; then
    pass "Шаблон удалён"
else
    fail "Не удалось удалить шаблон (HTTP: $TPL_DELETE_STATUS)"
fi

# Очистка
curl -s -X DELETE "$API_URL/projects/$TPL_PROJECT_ID" -H "$AUTH_HEADER" > /dev/null 2>&1

# ============================================================================
# Итоги
# ============================================================================

section "Результаты тестирования"

echo -e "${GREEN}✓ Пройдено тестов: $TESTS_PASSED${NC}"
echo -e "${RED}✗ Провалено тестов: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ВСЕ ТЕСТЫ ПРОЙДЕНЫ УСПЕШНО! 🎉${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${RED}════════════════════════════════════════${NC}"
    echo -e "${RED}  ЕСТЬ ПРОВАЛЬНЫЕ ТЕСТЫ ⚠️${NC}"
    echo -e "${RED}════════════════════════════════════════${NC}"
    exit 1
fi
