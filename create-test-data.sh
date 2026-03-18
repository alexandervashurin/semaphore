#!/bin/bash

# ============================================================================
# Скрипт создания тестовых сущностей в Velum через API
# ============================================================================
# Создаёт дополнительные тестовые данные для демонстрации CRUD
# ============================================================================

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Настройки API
API_URL="${API_URL:-http://localhost:3000/api}"
USERNAME="${USERNAME:-admin}"
PASSWORD="${PASSWORD:-demo123}"

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# ============================================================================
# Авторизация
# ============================================================================

info "Авторизация..."

LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    error "Не удалось авторизоваться. Проверьте логин/пароль."
    echo "Ответ API: $LOGIN_RESPONSE"
    exit 1
fi

success "Авторизация успешна"
AUTH_HEADER="Authorization: Bearer $TOKEN"

# ============================================================================
# Создание тестового проекта
# ============================================================================

info "Создание тестового проекта..."

PROJECT_RESPONSE=$(curl -s -X POST "$API_URL/projects" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Test Project - Auto Created",
        "alert": false,
        "max_parallel_tasks": 3,
        "type": "default"
    }')

PROJECT_ID=$(echo "$PROJECT_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

if [ -z "$PROJECT_ID" ]; then
    error "Не удалось создать проект"
    echo "Ответ API: $PROJECT_RESPONSE"
    exit 1
fi

success "Проект создан (ID: $PROJECT_ID)"

# ============================================================================
# Создание SSH ключа
# ============================================================================

info "Создание SSH ключа..."

KEY_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/keys" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Test SSH Key",
        "type": "ssh",
        "ssh_key": "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAlwAAAAdzc2gtcn\nNhAAAAAwEAAQAAAIEA0Z3VS5+X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X\n5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X5X\n-----END OPENSSH PRIVATE KEY-----",
        "owner": "project"
    }')

KEY_ID=$(echo "$KEY_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "SSH ключ создан (ID: $KEY_ID)"

# ============================================================================
# Создание репозитория
# ============================================================================

info "Создание репозитория..."

REPO_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/repositories" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d "{
        \"name\": \"Test Repository\",
        \"git_url\": \"https://github.com/ansible/ansible-examples.git\",
        \"git_type\": \"git\",
        \"git_branch\": \"main\",
        \"key_id\": $KEY_ID
    }")

REPO_ID=$(echo "$REPO_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "Репозиторий создан (ID: $REPO_ID)"

# ============================================================================
# Создание инвентаря
# ============================================================================

info "Создание инвентаря..."

INVENTORY_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/inventories" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d "{
        \"name\": \"Test Inventory\",
        \"inventory_type\": \"static\",
        \"inventory_data\": \"[webservers]\\nweb1.example.com ansible_host=192.168.1.10\\nweb2.example.com ansible_host=192.168.1.11\\n\\n[databases]\\ndb1.example.com ansible_host=192.168.1.20\\n\",
        \"ssh_login\": \"ansible\",
        \"ssh_port\": 22,
        \"ssh_key_id\": $KEY_ID
    }")

INVENTORY_ID=$(echo "$INVENTORY_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "Инвентарь создан (ID: $INVENTORY_ID)"

# ============================================================================
# Создание окружения
# ============================================================================

info "Создание окружения..."

ENV_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/environments" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d '{
        "name": "Test Environment",
        "json": "{\n  \"env\": \"test\",\n  \"debug\": true,\n  \"log_level\": \"debug\"\n}"
    }')

ENV_ID=$(echo "$ENV_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "Окружение создано (ID: $ENV_ID)"

# ============================================================================
# Создание шаблона
# ============================================================================

info "Создание шаблона..."

TEMPLATE_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/templates" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d "{
        \"name\": \"Test Template - Deploy\",
        \"playbook\": \"lemp-setup.yml\",
        \"description\": \"Test template for LEMP stack deployment\",
        \"inventory_id\": $INVENTORY_ID,
        \"repository_id\": $REPO_ID,
        \"environment_id\": $ENV_ID,
        \"type\": \"ansible\",
        \"app\": \"ansible\",
        \"git_branch\": \"main\"
    }")

TEMPLATE_ID=$(echo "$TEMPLATE_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "Шаблон создан (ID: $TEMPLATE_ID)"

# ============================================================================
# Создание расписания
# ============================================================================

info "Создание расписания..."

SCHEDULE_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/schedules" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d "{
        \"name\": \"Test Schedule - Daily Backup\",
        \"cron\": \"0 2 * * *\",
        \"template_id\": $TEMPLATE_ID,
        \"active\": true
    }")

SCHEDULE_ID=$(echo "$SCHEDULE_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "Расписание создано (ID: $SCHEDULE_ID)"

# ============================================================================
# Запуск задачи
# ============================================================================

info "Запуск задачи..."

TASK_RESPONSE=$(curl -s -X POST "$API_URL/projects/$PROJECT_ID/tasks" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d "{
        \"template_id\": $TEMPLATE_ID,
        \"debug\": false
    }")

TASK_ID=$(echo "$TASK_RESPONSE" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
success "Задача запущена (ID: $TASK_ID)"

# ============================================================================
# Вывод результатов
# ============================================================================

echo ""
echo "============================================================================"
echo "                   Тестовые сущности созданы!"
echo "============================================================================"
echo ""
echo "📁 Проект:              ${CYAN}$PROJECT_ID${NC}"
echo "🔑 SSH Ключ:            ${CYAN}$KEY_ID${NC}"
echo "📦 Репозиторий:         ${CYAN}$REPO_ID${NC}"
echo "🖥️  Инвентарь:           ${CYAN}$INVENTORY_ID${NC}"
echo "⚙️  Окружение:           ${CYAN}$ENV_ID${NC}"
echo "📋 Шаблон:              ${CYAN}$TEMPLATE_ID${NC}"
echo "🕐 Расписание:          ${CYAN}$SCHEDULE_ID${NC}"
echo "⚡ Задача:               ${CYAN}$TASK_ID${NC}"
echo ""
echo "📍 Проверить в UI: ${GREEN}http://localhost:80/demo-crud.html${NC}"
echo ""
echo "============================================================================"
echo ""

# Проверка всех сущностей
info "Проверка созданных сущностей..."

echo ""
echo "Проекты:"
curl -s "$API_URL/projects" -H "$AUTH_HEADER" | python3 -m json.tool 2>/dev/null || curl -s "$API_URL/projects" -H "$AUTH_HEADER"

echo ""
echo "Шаблоны проекта $PROJECT_ID:"
curl -s "$API_URL/projects/$PROJECT_ID/templates" -H "$AUTH_HEADER" | python3 -m json.tool 2>/dev/null || curl -s "$API_URL/projects/$PROJECT_ID/templates" -H "$AUTH_HEADER"

echo ""
echo "Задачи проекта $PROJECT_ID:"
curl -s "$API_URL/projects/$PROJECT_ID/tasks" -H "$AUTH_HEADER" | python3 -m json.tool 2>/dev/null || curl -s "$API_URL/projects/$PROJECT_ID/tasks" -H "$AUTH_HEADER"

echo ""
success "Все сущности проверены!"
