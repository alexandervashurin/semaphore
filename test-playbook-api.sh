#!/bin/bash
# Тестирование Playbook API

BASE_URL="${BASE_URL:-http://localhost:3000/api}"
PROJECT_ID="${PROJECT_ID:-1}"
TOKEN="${TOKEN:-}"

# Функция для получения токена аутентификации
get_token() {
    local username="${1:-admin}"
    local password="${2:-admin123}"
    
    echo "📝 Получение токена для пользователя: $username"
    TOKEN=$(curl -s -X POST "$BASE_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"login\":\"$username\",\"password\":\"$password\"}" \
        | jq -r '.token')
    
    if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
        echo "❌ Не удалось получить токен"
        exit 1
    fi
    
    echo "✅ Токен получен: ${TOKEN:0:20}..."
    export TOKEN
}

# Функция для выполнения запроса с авторизацией
auth_request() {
    curl -s -X "$1" "$2" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        ${3:+-d "$3"}
}

echo "🚀 Тестирование Playbook API"
echo "=============================="
echo ""

# Получить токен
get_token

echo ""
echo "1️⃣  Создание Playbook"
echo "---------------------"
PLAYBOOK_CREATE='{
    "name": "Test Playbook",
    "content": "- hosts: localhost\n  tasks:\n    - name: Test task\n      debug:\n        msg: \"Hello from Semaphore!\"",
    "description": "Тестовый плейбук",
    "playbook_type": "ansible",
    "repository_id": null
}'

CREATE_RESPONSE=$(auth_request POST "$BASE_URL/project/$PROJECT_ID/playbooks" "$PLAYBOOK_CREATE")
echo "$CREATE_RESPONSE" | jq .

PLAYBOOK_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
if [ -z "$PLAYBOOK_ID" ] || [ "$PLAYBOOK_ID" = "null" ]; then
    echo "❌ Не удалось создать playbook"
    exit 1
fi
echo "✅ Playbook создан с ID: $PLAYBOOK_ID"

echo ""
echo "2️⃣  Получение списка Playbooks"
echo "------------------------------"
auth_request GET "$BASE_URL/project/$PROJECT_ID/playbooks" | jq .

echo ""
echo "3️⃣  Получение конкретного Playbook"
echo "-----------------------------------"
auth_request GET "$BASE_URL/project/$PROJECT_ID/playbooks/$PLAYBOOK_ID" | jq .

echo ""
echo "4️⃣  Обновление Playbook"
echo "-----------------------"
PLAYBOOK_UPDATE='{
    "name": "Updated Test Playbook",
    "content": "- hosts: localhost\n  tasks:\n    - name: Updated task\n      debug:\n        msg: \"Updated!\"",
    "description": "Обновленный тестовый плейбук",
    "playbook_type": "ansible"
}'

auth_request PUT "$BASE_URL/project/$PROJECT_ID/playbooks/$PLAYBOOK_ID" "$PLAYBOOK_UPDATE" | jq .

echo ""
echo "5️⃣  Удаление Playbook"
echo "---------------------"
auth_request DELETE "$BASE_URL/project/$PROJECT_ID/playbooks/$PLAYBOOK_ID"
if [ $? -eq 0 ]; then
    echo "✅ Playbook удален"
else
    echo "❌ Ошибка при удалении"
fi

echo ""
echo "6️⃣  Проверка удаления"
echo "---------------------"
auth_request GET "$BASE_URL/project/$PROJECT_ID/playbooks/$PLAYBOOK_ID" | jq .

echo ""
echo "✅ Тестирование завершено!"
