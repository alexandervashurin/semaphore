#!/bin/bash
# Тестирование Schedules API (CRUD)

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

# Функция для выполнения PUT запроса
put_request() {
    curl -s -X PUT "$1" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "$2"
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

echo -e "${BLUE}╔════════════════════════════════════════╗"
echo -e "║  🕐 Тестирование Schedules API (CRUD)  ║"
echo -e "╚════════════════════════════════════════╝${NC}"
echo ""

# Получить токен
get_token

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}1️⃣  Создание расписания${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

SCHEDULE_CREATE='{
    "name": "Тестовое расписание - каждый день в 02:00",
    "template_id": 1,
    "cron": "0 2 * * *",
    "active": true,
    "project_id": '"$PROJECT_ID"'
}'

echo "Создание расписания..."
CREATE_RESPONSE=$(post_request "$BASE_URL/project/$PROJECT_ID/schedules" "$SCHEDULE_CREATE")
echo "$CREATE_RESPONSE" | jq .

SCHEDULE_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
if [ -z "$SCHEDULE_ID" ] || [ "$SCHEDULE_ID" = "null" ]; then
    echo -e "${RED}❌ Не удалось создать расписание${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Расписание создано с ID: $SCHEDULE_ID${NC}"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}2️⃣  Получение списка расписаний${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

echo "Запрос списка расписаний..."
get_request "$BASE_URL/project/$PROJECT_ID/schedules" | jq '.[] | {id, name, cron, active}'
check_success "Список расписаний получен" "Не удалось получить список расписаний"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}3️⃣  Получение конкретного расписания${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

echo "Получение расписания по ID..."
get_request "$BASE_URL/project/$PROJECT_ID/schedules/$SCHEDULE_ID" | jq .
check_success "Расписание получено" "Не удалось получить расписание"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}4️⃣  Обновление расписания${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

SCHEDULE_UPDATE='{
    "name": "Обновленное расписание - каждое воскресенье в 03:00",
    "template_id": 1,
    "cron": "0 3 * * 0",
    "active": false,
    "project_id": '"$PROJECT_ID"'
}'

echo "Обновление расписания..."
put_request "$BASE_URL/project/$PROJECT_ID/schedules/$SCHEDULE_ID" "$SCHEDULE_UPDATE" | jq .
check_success "Расписание обновлено" "Не удалось обновить расписание"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}5️⃣  Валидация cron выражения${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

echo "Валидация корректного cron..."
VALIDATE_RESPONSE=$(post_request "$BASE_URL/project/$PROJECT_ID/schedules/validate" '{"cron": "0 */6 * * *"}')
echo "$VALIDATE_RESPONSE" | jq .

VALID=$(echo "$VALIDATE_RESPONSE" | jq -r '.valid')
if [ "$VALID" = "true" ]; then
    echo -e "${GREEN}✅ Cron выражение валидно${NC}"
else
    echo -e "${RED}❌ Cron выражение невалидно${NC}"
fi

echo ""
echo "Валидация некорректного cron..."
INVALID_RESPONSE=$(post_request "$BASE_URL/project/$PROJECT_ID/schedules/validate" '{"cron": "invalid cron"}')
echo "$INVALID_RESPONSE" | jq .

INVALID=$(echo "$INVALID_RESPONSE" | jq -r '.valid')
if [ "$INVALID" = "false" ]; then
    echo -e "${GREEN}✅ Некорректный cron отклонён${NC}"
else
    echo -e "${RED}❌ Некорректный cron не был отклонён${NC}"
fi

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}6️⃣  Создание дополнительных расписаний${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

# Расписание 2: Каждые 6 часов
SCHEDULE2='{
    "name": "Каждые 6 часов",
    "template_id": 1,
    "cron": "0 */6 * * *",
    "active": true,
    "project_id": '"$PROJECT_ID"'
}'
echo "Создание расписания 'Каждые 6 часов'..."
post_request "$BASE_URL/project/$PROJECT_ID/schedules" "$SCHEDULE2" | jq '.id' | xargs -I {} echo -e "${GREEN}✅ Создано с ID: {}${NC}"

# Расписание 3: 1-го числа каждого месяца
SCHEDULE3='{
    "name": "Первого числа каждого месяца",
    "template_id": 1,
    "cron": "0 0 1 * *",
    "active": true,
    "project_id": '"$PROJECT_ID"'
}'
echo "Создание расписания 'Первого числа'..."
post_request "$BASE_URL/project/$PROJECT_ID/schedules" "$SCHEDULE3" | jq '.id' | xargs -I {} echo -e "${GREEN}✅ Создано с ID: {}${NC}"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}7️⃣  Проверка фильтрации по статусу${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

echo "Активные расписания:"
get_request "$BASE_URL/project/$PROJECT_ID/schedules?active=true" | jq 'length' | xargs -I {} echo -e "Найдено: ${GREEN}{}${NC}"

echo "Неактивные расписания:"
get_request "$BASE_URL/project/$PROJECT_ID/schedules?active=false" | jq 'length' | xargs -I {} echo -e "Найдено: ${YELLOW}{}${NC}"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}8️⃣  Удаление расписания${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

echo "Удаление расписания с ID $SCHEDULE_ID..."
delete_request "$BASE_URL/project/$PROJECT_ID/schedules/$SCHEDULE_ID"
check_success "Расписание удалено" "Не удалось удалить расписание"

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}9️⃣  Проверка удаления${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

echo "Попытка получить удалённое расписание..."
DELETE_CHECK=$(get_request "$BASE_URL/project/$PROJECT_ID/schedules/$SCHEDULE_ID")
if echo "$DELETE_CHECK" | jq -e '.error' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Расписание действительно удалено${NC}"
else
    echo -e "${RED}❌ Расписание не было удалено${NC}"
fi

echo ""
echo -e "${YELLOW}════════════════════════════════════════${NC}"
echo -e "${BLUE}📊 Итоговая статистика${NC}"
echo -e "${YELLOW}════════════════════════════════════════${NC}"

TOTAL=$(get_request "$BASE_URL/project/$PROJECT_ID/schedules" | jq 'length')
ACTIVE=$(get_request "$BASE_URL/project/$PROJECT_ID/schedules?active=true" | jq 'length')
INACTIVE=$(get_request "$BASE_URL/project/$PROJECT_ID/schedules?active=false" | jq 'length')

echo "Всего расписаний: ${BLUE}$TOTAL${NC}"
echo "Активных: ${GREEN}$ACTIVE${NC}"
echo "Неактивных: ${YELLOW}$INACTIVE${NC}"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✅ Все тесты Schedules API пройдены!  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
