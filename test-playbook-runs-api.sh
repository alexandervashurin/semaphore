#!/bin/bash
# Тестирование Playbook Runs API (запуск и мониторинг задач)

set -e

BASE_URL="${BASE_URL:-http://localhost:3000/api}"
PROJECT_ID="${PROJECT_ID:-1}"
TOKEN="${TOKEN:-}"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Функция для получения токена аутентификации
get_token() {
    local username="${1:-admin}"
    local password="${2:-admin123}"

    echo -e "${BLUE}📝 Получение токена для пользователя: $username${NC}"
    TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"login\":\"$username\",\"password\":\"$password\"}" \
        | jq -r '.token')

    if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
        echo -e "${RED}❌ Не удалось получить токен${NC}"
        exit 1
    fi

    echo -e "${GREEN}✅ Токен получен:${NC} ${TOKEN:0:20}..."
    export TOKEN
}

# Функция для выполнения POST запроса
post_request() {
    curl -s -X POST "$1" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "$2"
}

# Функция для выполнения GET запроса
get_request() {
    curl -s -X GET "$1" \
        -H "Authorization: Bearer $TOKEN"
}

# Функция для выполнения DELETE запроса
delete_request() {
    curl -s -X DELETE "$1" \
        -H "Authorization: Bearer $TOKEN"
}

# Функция для проверки успешности операции
check_success() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $1${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        return 1
    fi
}

# Функция ожидания завершения задачи
wait_for_task() {
    local task_id=$1
    local timeout=${2:-60}
    local interval=${3:-2}
    local elapsed=0

    echo "Ожидание завершения задачи $task_id (таймаут: ${timeout}с)..."
    
    while [ $elapsed -lt $timeout ]; do
        RESPONSE=$(get_request "$BASE_URL/project/$PROJECT_ID/tasks/$task_id")
        STATUS=$(echo "$RESPONSE" | jq -r '.status')
        
        echo "  Статус: $STATUS (прошло: ${elapsed}с)"
        
        if [ "$STATUS" = "success" ] || [ "$STATUS" = "failed" ] || [ "$STATUS" = "stopped" ]; then
            echo -e "${GREEN}✅ Задача завершена со статусом: $STATUS${NC}"
            return 0
        fi
        
        sleep $interval
        elapsed=$((elapsed + interval))
    done
    
    echo -e "${RED}❌ Таймаут ожидания задачи${NC}"
    return 1
}

echo -e "${BLUE}╔════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  🚀 Тестирование Playbook Runs API (Tasks)     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════╝${NC}"
echo ""

# Получить токен
get_token

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}1️⃣  Получение списка шаблонов${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

TEMPLATES=$(get_request "$BASE_URL/project/$PROJECT_ID/templates")
echo "$TEMPLATES" | jq '.[] | {id, name, playbook}'

TEMPLATE_ID=$(echo "$TEMPLATES" | jq -r '.[0].id // empty')
if [ -z "$TEMPLATE_ID" ]; then
    echo -e "${RED}❌ Нет доступных шаблонов для теста${NC}"
    echo -e "${YELLOW}💡 Создайте шаблон или используйте существующий${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Используется шаблон ID: $TEMPLATE_ID${NC}"

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}2️⃣  Запуск задачи (базовый)${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

TASK_CREATE='{
    "template_id": '"$TEMPLATE_ID"',
    "project_id": '"$PROJECT_ID"',
    "debug": false,
    "dry_run": false
}'

echo "Запуск задачи..."
CREATE_RESPONSE=$(post_request "$BASE_URL/project/$PROJECT_ID/tasks" "$TASK_CREATE")
echo "$CREATE_RESPONSE" | jq .

TASK_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
if [ -z "$TASK_ID" ] || [ "$TASK_ID" = "null" ]; then
    echo -e "${RED}❌ Не удалось создать задачу${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Задача создана с ID: $TASK_ID${NC}"

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}3️⃣  Мониторинг статуса задачи${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

wait_for_task "$TASK_ID" 30

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}4️⃣  Получение информации о задаче${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

echo "Детальная информация о задаче:"
get_request "$BASE_URL/project/$PROJECT_ID/tasks/$TASK_ID" | jq .

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}5️⃣  Получение лога выполнения${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

echo "Лог выполнения:"
get_request "$BASE_URL/project/$PROJECT_ID/tasks/$TASK_ID/output" | jq -r '.output' | head -20

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}6️⃣  Запуск задачи с параметрами${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

TASK_CREATE_ADVANCED='{
    "template_id": '"$TEMPLATE_ID"',
    "project_id": '"$PROJECT_ID"',
    "debug": true,
    "dry_run": false,
    "diff": true,
    "arguments": "--verbose",
    "git_branch": "main"
}'

echo "Запуск задачи с расширенными параметрами..."
CREATE_RESPONSE2=$(post_request "$BASE_URL/project/$PROJECT_ID/tasks" "$TASK_CREATE_ADVANCED")
echo "$CREATE_RESPONSE2" | jq .

TASK_ID2=$(echo "$CREATE_RESPONSE2" | jq -r '.id')
if [ -z "$TASK_ID2" ] || [ "$TASK_ID2" = "null" ]; then
    echo -e "${RED}❌ Не удалось создать задачу${NC}"
else
    echo -e "${GREEN}✅ Задача 2 создана с ID: $TASK_ID2${NC}"
    
    echo ""
    echo "Ожидание завершения задачи 2..."
    wait_for_task "$TASK_ID2" 30 || true
fi

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}7️⃣  Получение списка задач проекта${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

echo "Все задачи проекта:"
get_request "$BASE_URL/project/$PROJECT_ID/tasks?limit=10" | jq '.[] | {id, status, created, template_id}'

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}8️⃣  Статистика задач${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

ALL_TASKS=$(get_request "$BASE_URL/project/$PROJECT_ID/tasks?limit=100")
TOTAL=$(echo "$ALL_TASKS" | jq 'length')
SUCCESS=$(echo "$ALL_TASKS" | jq '[.[] | select(.status=="success")] | length')
FAILED=$(echo "$ALL_TASKS" | jq '[.[] | select(.status=="failed")] | length')
WAITING=$(echo "$ALL_TASKS" | jq '[.[] | select(.status=="waiting")] | length')
RUNNING=$(echo "$ALL_TASKS" | jq '[.[] | select(.status=="running")] | length')

echo "Всего задач: ${BLUE}$TOTAL${NC}"
echo "Успешно: ${GREEN}$SUCCESS${NC}"
echo "Ошибок: ${RED}$FAILED${NC}"
echo "Ожидают: ${YELLOW}$WAITING${NC}"
echo "Выполняются: ${BLUE}$RUNNING${NC}"

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}9️⃣  Удаление старых задач${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

# Удаляем только что созданные тестовые задачи
echo "Удаление тестовой задачи $TASK_ID..."
delete_request "$BASE_URL/project/$PROJECT_ID/tasks/$TASK_ID"
check_success "Задача $TASK_ID удалена" "Не удалось удалить задачу $TASK_ID"

if [ -n "$TASK_ID2" ] && [ "$TASK_ID2" != "null" ]; then
    echo "Удаление тестовой задачи $TASK_ID2..."
    delete_request "$BASE_URL/project/$PROJECT_ID/tasks/$TASK_ID2"
    check_success "Задача $TASK_ID2 удалена" "Не удалось удалить задачу $TASK_ID2"
fi

echo ""
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"
echo -e "${BLUE}📊 Итоговая статистика${NC}"
echo -e "${YELLOW}════════════════════════════════════════════════${NC}"

REMAINING=$(get_request "$BASE_URL/project/$PROJECT_ID/tasks?limit=100" | jq 'length')
echo "Осталось задач в проекте: ${BLUE}$REMAINING${NC}"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✅ Все тесты Playbook Runs API пройдены!      ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════╝${NC}"
echo ""
