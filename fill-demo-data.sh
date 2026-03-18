#!/bin/bash
# ============================================================================
# Наполнение БД тестовыми данными для Velum (Rust)
# Совместим с Go semaphore API: inventory_type + inventory (data)
# ============================================================================
# Использование:
#   bash fill-demo-data.sh              # localhost:8088 (docker-compose.demo.yml)
#   bash fill-demo-data.sh native       # localhost:3000 (cargo run)
# ============================================================================
set -e

MODE="${1:-docker}"
case $MODE in
    native) API_URL="http://localhost:3000/api" ;;
    *) API_URL="http://localhost:8088/api" ;;
esac
USERNAME="admin"; PASSWORD="admin123"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'
# Все сообщения в stderr — не мешают захвату ID через $()
info()    { echo -e "${BLUE}[INFO]${NC} $1" >&2; }
success() { echo -e "${GREEN}[OK]${NC} $1" >&2; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1" >&2; }
error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; exit 1; }
step()    { echo -e "${CYAN}-->${NC} $1" >&2; }

# Извлечение ID из JSON без python/jq (работает на Windows Git Bash)
extract_id() { grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2; }

get_token() {
    step "Авторизация..."
    local resp
    resp=$(curl -sf -X POST "$API_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}") \
        || error "Не удалось подключиться к $API_URL"
    TOKEN=$(echo "$resp" | grep -o '"token":"[^"]*"' | head -1 | cut -d'"' -f4)
    [ -z "$TOKEN" ] && error "Токен не получен. Ответ: $resp"
    success "Токен получен"
}

post() {
    curl -sf -X POST "$API_URL$1" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "$2"
}

create_project() {
    step "Проект: $1"
    local resp; resp=$(post "/projects" "{\"name\":\"$1\",\"alert\":false,\"max_parallel_tasks\":0}")
    local id; id=$(echo "$resp" | extract_id)
    [ -n "$id" ] && success "Проект '$1' (ID: $id)" || { warn "Ошибка: $resp"; id=0; }
    echo "${id:-0}"
}

create_key() {
    # $1=pid $2=name $3=type $4=login(opt) $5=secret(opt)
    step "Ключ: $2 (type=$3)"
    local body="{\"name\":\"$2\",\"type\":\"$3\""
    [ -n "$4" ] && body="${body},\"login\":\"$4\""
    [ -n "$5" ] && body="${body},\"secret\":\"$5\""
    body="${body}}"
    local id; id=$(post "/project/$1/keys" "$body" | extract_id)
    [ -n "$id" ] && success "Ключ '$2' (ID: $id)" || warn "Ошибка ключа '$2'"
    echo "${id:-0}"
}

create_repo() {
    # $1=pid $2=name $3=url $4=branch
    step "Репозиторий: $2"
    local id
    id=$(post "/project/$1/repositories" \
        "{\"name\":\"$2\",\"git_url\":\"$3\",\"git_branch\":\"${4:-main}\"}" | extract_id)
    [ -n "$id" ] && success "Репозиторий '$2' (ID: $id)" || warn "Ошибка репозитория '$2'"
    echo "${id:-0}"
}

create_inventory() {
    # $1=pid $2=name $3=type $4=data (multiline ok — will be JSON-escaped)
    step "Инвентарь: $2 (type=$3)"
    # escape newlines for JSON
    local data_esc; data_esc=$(printf '%s' "$4" | sed 's/\\/\\\\/g; s/"/\\"/g' | tr '\n' '\\' | sed 's/\\/\\n/g')
    local id
    id=$(post "/project/$1/inventory" \
        "{\"name\":\"$2\",\"inventory_type\":\"$3\",\"inventory\":\"${data_esc}\"}" | extract_id)
    [ -n "$id" ] && success "Инвентарь '$2' (ID: $id)" || warn "Ошибка инвентаря '$2'"
    echo "${id:-0}"
}

create_environment() {
    # $1=pid $2=name $3=json_vars (single-line JSON object)
    step "Окружение: $2"
    local id
    id=$(post "/project/$1/environment" \
        "{\"name\":\"$2\",\"json\":\"$3\"}" | extract_id)
    [ -n "$id" ] && success "Окружение '$2' (ID: $id)" || warn "Ошибка окружения '$2'"
    echo "${id:-0}"
}

create_template() {
    # $1=pid $2=name $3=playbook $4=inventory_id $5=repo_id(opt) $6=env_id(opt) $7=app
    step "Шаблон: $2"
    local body="{\"name\":\"$2\",\"playbook\":\"$3\",\"inventory_id\":$4,\"app\":\"${7:-ansible}\""
    [ -n "$5" ] && [ "$5" -gt 0 ] 2>/dev/null && body="${body},\"repository_id\":$5"
    [ -n "$6" ] && [ "$6" -gt 0 ] 2>/dev/null && body="${body},\"environment_id\":$6"
    body="${body}}"
    local id; id=$(post "/project/$1/templates" "$body" | extract_id)
    [ -n "$id" ] && success "Шаблон '$2' (ID: $id)" || warn "Ошибка шаблона '$2'"
    echo "${id:-0}"
}

create_schedule() {
    # $1=pid $2=name $3=template_id $4=cron
    step "Расписание: $2 ($4)"
    local id
    id=$(post "/project/$1/schedules" \
        "{\"id\":0,\"name\":\"$2\",\"template_id\":$3,\"cron\":\"$4\",\"active\":true,\"project_id\":$1}" | extract_id)
    [ -n "$id" ] && success "Расписание '$2' (ID: $id)" || warn "Ошибка расписания '$2'"
    echo "${id:-0}"
}

create_task() {
    # $1=pid $2=template_id
    step "Запуск задачи (шаблон #$2)..."
    local id; id=$(post "/project/$1/tasks" "{\"template_id\":$2}" | extract_id)
    [ -n "$id" ] && success "Задача запущена (ID: $id)" || warn "Ошибка задачи"
    echo "${id:-0}"
}

# ════════════════════════════════════════════════════════════════════════════
info "API: $API_URL"
get_token

# ── Проект 1: Demo Project ──────────────────────────────────────────────────
echo ""
echo -e "${YELLOW}== Проект 1: Demo Project ==${NC}"
P1=$(create_project "Demo Project")

create_key "$P1" "No Key (none)" "none" "" ""
create_key "$P1" "demo user (password)" "login_password" "demo" "demo123"

R1=$(create_repo "$P1" "Demo Playbooks (local)" "file:///app/playbooks" "main")

I_LOCAL=$(create_inventory "$P1" "Localhost" "static" \
"[local]
localhost ansible_connection=local")

I_TARGET=$(create_inventory "$P1" "Demo Target (ansible-target)" "static" \
"[demo_servers]
ansible-target ansible_user=demo ansible_password=demo123 ansible_ssh_common_args='-o StrictHostKeyChecking=no'")

E_DEV=$(create_environment "$P1" "Development" \
'{\"ENV\": \"development\", \"DEBUG\": \"true\", \"APP_VERSION\": \"1.0.0\"}')
E_PROD=$(create_environment "$P1" "Production" \
'{\"ENV\": \"production\", \"DEBUG\": \"false\", \"APP_VERSION\": \"2.5.1\"}')

T_HELLO=$(create_template "$P1" "Hello World (localhost)" \
    "hello.yml" "$I_LOCAL" "$R1" "$E_DEV" "ansible")
T_PING=$(create_template "$P1" "Ping Demo Servers" \
    "ping.yml" "$I_TARGET" "$R1" "$E_DEV" "ansible")
T_DEPLOY=$(create_template "$P1" "Deploy Web App" \
    "deploy-web.yml" "$I_TARGET" "$R1" "$E_PROD" "ansible")

create_schedule "$P1" "Hourly Hello" "$T_HELLO" "0 * * * *"
create_schedule "$P1" "Nightly Deploy" "$T_DEPLOY" "0 2 * * *"
create_task "$P1" "$T_HELLO"

# ── Проект 2: Infrastructure ────────────────────────────────────────────────
echo ""
echo -e "${YELLOW}== Проект 2: Infrastructure ==${NC}"
P2=$(create_project "Infrastructure")

create_key "$P2" "Deploy SSH Key" "ssh" "ubuntu" ""
create_key "$P2" "AWS Access" "login_password" "aws_key_id" "aws_secret_key"
create_repo "$P2" "Infrastructure Code" "https://github.com/example/infra.git" "main"
create_repo "$P2" "Terraform Modules" "https://github.com/example/terraform-modules.git" "main"

I_PROD2=$(create_inventory "$P2" "Production Servers" "static" \
"[production]
prod1.example.com
prod2.example.com")
create_inventory "$P2" "Staging Servers" "static" \
"[staging]
staging1.example.com"
create_environment "$P2" "AWS us-east-1" \
'{\"AWS_REGION\": \"us-east-1\", \"TF_VAR_env\": \"prod\"}'
create_environment "$P2" "GCP europe-west" \
'{\"GOOGLE_PROJECT\": \"my-project\"}'
T_PROV=$(create_template "$P2" "Provision Servers" "provision.yml" "$I_PROD2" "" "" "ansible")
create_schedule "$P2" "Weekly Provision" "$T_PROV" "0 3 * * 1"

# ── Проект 3: Web Applications ───────────────────────────────────────────────
echo ""
echo -e "${YELLOW}== Проект 3: Web Applications ==${NC}"
P3=$(create_project "Web Applications")

create_key "$P3" "Deploy User" "login_password" "deploy" "deploy123"
create_repo "$P3" "Web App Repo" "https://github.com/example/webapp.git" "main"

I_WEB=$(create_inventory "$P3" "Web Servers" "static" \
"[webservers]
web1.example.com
web2.example.com

[dbservers]
db1.example.com")
create_environment "$P3" "Production" \
'{\"APP_ENV\": \"production\", \"NODE_ENV\": \"production\"}'

T_FE=$(create_template "$P3" "Deploy Frontend" "deploy.yml" "$I_WEB" "" "" "ansible")
T_SSL=$(create_template "$P3" "Update SSL Certs" "ssl-renew.yml" "$I_WEB" "" "" "ansible")
create_template "$P3" "Restart Services" "restart.yml" "$I_WEB" "" "" "ansible"
create_schedule "$P3" "Nightly Deploy" "$T_FE" "0 1 * * *"
create_schedule "$P3" "Monthly SSL Renew" "$T_SSL" "0 4 1 * *"

# ── Итог ─────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}+----------------------------------------------------------+${NC}"
echo -e "${GREEN}|   Тестовые данные созданы!                               |${NC}"
echo -e "${GREEN}+----------------------------------------------------------+${NC}"
echo ""
echo "  Проектов: 3 | Шаблонов: 8 | Расписаний: 5"
echo ""
echo -e "${CYAN}http://localhost:8088  (admin / admin123)${NC}"
echo ""
echo "Реальный ansible:"
echo "  Demo Project -> Шаблоны -> 'Hello World (localhost)' -> Run"
echo "  Demo Project -> Шаблоны -> 'Ping Demo Servers' -> Run"
