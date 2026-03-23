#!/bin/bash
# ============================================================================
# fill-postgres-demo-data.sh — заполняет демо-данными запущенный Velum
# Запуск: bash fill-postgres-demo-data.sh
# Требует: curl, jq
# ============================================================================

BASE="http://localhost:3000/api"
ADMIN="admin"
PASS="admin123"

echo "🔑 Получение токена..."
TOKEN=$(curl -sf -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"auth\":\"$ADMIN\",\"password\":\"$PASS\"}" | jq -r '.token')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "❌ Ошибка авторизации"
  exit 1
fi
echo "✅ Токен получен"

call() {
  curl -sf -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" "$@"
}

echo "📁 Создание проекта..."
PROJECT=$(call -X POST "$BASE/projects" \
  -d '{"name":"Demo Project","max_parallel_tasks":0}')
PID=$(echo "$PROJECT" | jq -r '.id')
echo "   Project ID: $PID"

echo "🔐 Создание SSH-ключа (login/password)..."
KEY=$(call -X POST "$BASE/project/$PID/keys" \
  -d '{"name":"demo-ssh-password","type":"login_password","login_password":{"login":"demo","password":"demo123"}}')
KID=$(echo "$KEY" | jq -r '.id')
echo "   Key ID: $KID"

echo "📋 Создание инвентаря..."
INV=$(call -X POST "$BASE/project/$PID/inventory" \
  -d "{\"name\":\"ansible-target\",\"project_id\":$PID,\"type\":\"static\",
    \"inventory\":\"[targets]\nlocalhost ansible_connection=local\",
    \"ssh_key_id\":$KID}")
IID=$(echo "$INV" | jq -r '.id')
echo "   Inventory ID: $IID"

echo "📦 Создание репозитория..."
REPO=$(call -X POST "$BASE/project/$PID/repositories" \
  -d "{\"name\":\"demo-playbooks\",\"project_id\":$PID,\"git_url\":\"https://github.com/velum/velum-demo-playbooks.git\",\"git_branch\":\"main\",\"ssh_key_id\":$KID}")
RID=$(echo "$REPO" | jq -r '.id')
echo "   Repository ID: $RID"

echo "🌍 Создание окружения..."
ENV=$(call -X POST "$BASE/project/$PID/environment" \
  -d "{\"name\":\"default\",\"project_id\":$PID,\"password\":\"\",\"json\":\"{}\",\"env\":\"ANSIBLE_HOST_KEY_CHECKING=False\"}")
EID=$(echo "$ENV" | jq -r '.id')
echo "   Environment ID: $EID"

echo "📝 Создание шаблонов..."
call -X POST "$BASE/project/$PID/templates" \
  -d "{\"name\":\"Ping & Facts\",\"project_id\":$PID,\"inventory_id\":$IID,\"repository_id\":$RID,\"environment_id\":$EID,\"app\":\"ansible\",\"playbook\":\"ping.yml\",\"description\":\"Ping target and gather facts\"}" > /dev/null
call -X POST "$BASE/project/$PID/templates" \
  -d "{\"name\":\"Hello World\",\"project_id\":$PID,\"inventory_id\":$IID,\"repository_id\":$RID,\"environment_id\":$EID,\"app\":\"ansible\",\"playbook\":\"hello.yml\",\"description\":\"Hello from Velum (Rust)\"}" > /dev/null
echo "   Templates created"

echo ""
echo "✅ Готово! Откройте http://localhost:3000"
echo "   Логин: admin / admin123"
