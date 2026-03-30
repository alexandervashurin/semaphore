#!/bin/bash
# ============================================================================
# fill-debian-target.sh — настройка Velum с Debian 192.168.0.18 как target
# Запуск: bash fill-debian-target.sh
# ============================================================================

BASE="http://localhost:8088/api"
DEBIAN_HOST="192.168.0.18"
DEBIAN_USER="administrator"
DEBIAN_PASS="Yfcnhjqrf!23"

echo "🔐 Авторизация..."
TOKEN=$(curl -sf -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"auth":"admin","password":"admin123"}' | jq -r '.token')

[ -z "$TOKEN" ] || [ "$TOKEN" = "null" ] && echo "❌ Ошибка авторизации" && exit 1
echo "✅ Токен получен"

call() {
  curl -sf -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" "$@"
}

echo ""
echo "📁 Создание проекта 'Debian Lab'..."
PROJECT=$(call -X POST "$BASE/projects" \
  -d '{"name":"Debian Lab","max_parallel_tasks":4}')
PID=$(echo "$PROJECT" | jq -r '.id')
echo "   Project ID: $PID"

echo "🔑 Создание SSH ключа (login/password) для Debian..."
KEY=$(call -X POST "$BASE/project/$PID/keys" \
  -d "{\"name\":\"debian-admin\",\"type\":\"login_password\",\"login_password\":{\"login\":\"$DEBIAN_USER\",\"password\":\"$DEBIAN_PASS\"}}")
KID=$(echo "$KEY" | jq -r '.id')
echo "   Key ID: $KID"

echo "📋 Создание инвентаря с Debian-сервером..."
INVENTORY="[debian]
$DEBIAN_HOST ansible_host=$DEBIAN_HOST ansible_port=22 ansible_user=$DEBIAN_USER ansible_ssh_pass=$DEBIAN_PASS ansible_become=true ansible_become_method=sudo ansible_become_pass=$DEBIAN_PASS ansible_python_interpreter=/usr/bin/python3"

INV=$(call -X POST "$BASE/project/$PID/inventory" \
  -d "{\"name\":\"Debian 192.168.0.18\",\"project_id\":$PID,\"type\":\"static\",
    \"inventory\":$(echo "$INVENTORY" | jq -Rs .),
    \"ssh_key_id\":$KID}")
IID=$(echo "$INV" | jq -r '.id')
echo "   Inventory ID: $IID"

echo "📦 Создание репозитория (встроенные плейбуки)..."
REPO=$(call -X POST "$BASE/project/$PID/repositories" \
  -d "{\"name\":\"demo-playbooks\",\"project_id\":$PID,
    \"git_url\":\"file:///app/playbooks\",\"git_branch\":\"master\",\"ssh_key_id\":$KID}")
RID=$(echo "$REPO" | jq -r '.id')
echo "   Repository ID: $RID"

echo "⚙️  Создание окружения..."
ENV=$(call -X POST "$BASE/project/$PID/environment" \
  -d "{\"name\":\"default\",\"project_id\":$PID,\"password\":\"\",
    \"json\":\"{}\",\"env\":\"ANSIBLE_HOST_KEY_CHECKING=False\nANSIBLE_TIMEOUT=30\"}")
EID=$(echo "$ENV" | jq -r '.id')
echo "   Environment ID: $EID"

echo ""
echo "📝 Создание шаблонов задач..."

create_template() {
  local name="$1" playbook="$2" desc="$3"
  call -X POST "$BASE/project/$PID/templates" \
    -d "{\"name\":\"$name\",\"project_id\":$PID,
      \"inventory_id\":$IID,\"repository_id\":$RID,\"environment_id\":$EID,
      \"app\":\"ansible\",\"playbook\":\"$playbook\",\"description\":\"$desc\"}" | jq -r '.id'
}

T1=$(create_template "Ping & Facts" "ping.yml" "Ping и факты о хосте")
echo "   ✅ Ping & Facts: $T1"

T2=$(create_template "Hello World" "hello.yml" "Hello World проверка")
echo "   ✅ Hello World: $T2"

T3=$(create_template "Debian System Info" "debian-sysinfo.yml" "CPU/RAM/диск/температура")
echo "   ✅ Debian System Info: $T3"

T4=$(create_template "Debian Services Check" "debian-services.yml" "Проверка сервисов и портов")
echo "   ✅ Debian Services Check: $T4"

T5=$(create_template "Debian Update Packages" "debian-update.yml" "apt update + safe upgrade")
echo "   ✅ Debian Update: $T5"

T6=$(create_template "Debian System Cleanup" "debian-cleanup.yml" "Очистка apt/journald/tmp")
echo "   ✅ Debian Cleanup: $T6"

T7=$(create_template "Deploy Web App" "deploy-web.yml" "Демо деплой приложения")
echo "   ✅ Deploy Web App: $T7"

echo ""
echo "✅ Готово! Открой http://localhost:8088"
echo "   Логин: admin / admin123"
echo "   Проект: Debian Lab (ID: $PID)"
echo ""
echo "🚀 Запусти задачу 'Debian System Info' первой!"
