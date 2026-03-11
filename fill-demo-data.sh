#!/bin/bash
# ============================================================================
# Скрипт наполнения БД тестовыми данными для Semaphore UI
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Режим по умолчанию
MODE="${1:-native}"

# URL API и учётные данные
case $MODE in
    native)
        API_URL="http://localhost:3000/api"
        USERNAME="admin"
        PASSWORD="admin123"
        ;;
    hybrid|docker)
        API_URL="http://localhost:3000/api"
        USERNAME="admin"
        PASSWORD="demo123"
        ;;
    *)
        echo -e "${RED}Неизвестный режим: $MODE${NC}"
        exit 1
        ;;
esac

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
step() { echo -e "${CYAN}➜${NC} $1"; }

# Получение токена
get_token() {
    step "Получение токена..."
    local response=$(curl -s -X POST "$API_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")
    TOKEN=$(echo "$response" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    [ -z "$TOKEN" ] && error "Не удалось получить токен"
    success "Токен получен"
}

# API функции
api_post() {
    curl -s -X POST "$API_URL$1" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "$2"
}

# Создание проекта
create_project() {
    step "Создание проекта: $1..."
    local response=$(api_post "/projects" "{\"name\":\"$1\"}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Проект создан (ID: $id)" && echo "$id" || echo "0"
}

# Создание ключа доступа
create_key() {
    step "Создание ключа: $2..."
    local response=$(api_post "/project/$1/keys" "{\"name\":\"$2\",\"type\":\"$3\",\"login\":\"$4\",\"password\":\"$5\"}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Ключ создан (ID: $id)" && echo "$id" || echo "0"
}

# Создание репозитория
create_repo() {
    step "Создание репозитория: $2..."
    local response=$(api_post "/project/$1/repositories" "{\"name\":\"$2\",\"git_url\":\"$3\",\"git_branch\":\"$4\"}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Репозиторий создан (ID: $id)" && echo "$id" || echo "0"
}

# Создание инвентаря
create_inventory() {
    step "Создание инвентаря: $2..."
    local response=$(api_post "/project/$1/inventories" "{\"name\":\"$2\",\"inventory_type\":\"static\",\"inventory_data\":\"$4\",\"ssh_login\":\"$5\",\"ssh_port\":$6,\"key_id\":$7}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Инвентарь создан (ID: $id)" && echo "$id" || echo "0"
}

# Создание окружения
create_env() {
    step "Создание окружения: $2..."
    local response=$(api_post "/project/$1/environments" "{\"name\":\"$2\",\"json\":\"$3\"}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Окружение создано (ID: $id)" && echo "$id" || echo "0"
}

# Создание шаблона
create_template() {
    step "Создание шаблона: $2..."
    local response=$(api_post "/project/$1/templates" "{\"name\":\"$2\",\"playbook\":\"$3\",\"inventory_id\":$4,\"repository_id\":$5,\"environment_id\":$6,\"app\":\"$7\"}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Шаблон создан (ID: $id)" && echo "$id" || echo "0"
}

# Создание задачи
create_task() {
    step "Создание задачи..."
    local response=$(api_post "/project/$1/tasks" "{\"template_id\":$2}")
    local id=$(echo "$response" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)
    [ -n "$id" ] && success "Задача создана (ID: $id)" && echo "$id" || echo "0"
}

# ============================================================================
# Основное наполнение
# ============================================================================

info "Режим: $MODE | API: $API_URL"
get_token

echo ""
echo -e "${YELLOW}=== Проекты ===${NC}"
P1=$(create_project "Demo Project")
P2=$(create_project "Infrastructure")
P3=$(create_project "Web Applications")

echo ""
echo -e "${YELLOW}=== Ключи доступа ===${NC}"
K1=$(create_key $P1 "Local User" "login_password" "root" "password123")
K2=$(create_key $P1 "Ansible Vault" "none" "" "")
K3=$(create_key $P2 "SSH Key" "ssh" "ubuntu" "")
K4=$(create_key $P2 "Admin User" "login_password" "admin" "admin123")
K5=$(create_key $P3 "Deploy User" "login_password" "deploy" "deploy123")

echo ""
echo -e "${YELLOW}=== Репозитории ===${NC}"
R1=$(create_repo $P1 "Demo Repository" "https://github.com/semaphoreui/demo-playbooks.git" "main")
R2=$(create_repo $P1 "Local Scripts" "https://github.com/example/scripts.git" "master")
R3=$(create_repo $P2 "Infrastructure Code" "https://github.com/example/infrastructure.git" "main")
R4=$(create_repo $P2 "Terraform Modules" "https://github.com/example/terraform-modules.git" "main")
R5=$(create_repo $P3 "Web App Deploy" "https://github.com/example/webapp.git" "main")
R6=$(create_repo $P3 "Nginx Configs" "https://github.com/example/nginx-configs.git" "master")

echo ""
echo -e "${YELLOW}=== Инвентари ===${NC}"
I1=$(create_inventory $P1 "Demo Inventory" "static" "[webservers]\\nweb1.example.com\\nweb2.example.com" "root" 22 $K1)
I2=$(create_inventory $P2 "Production Servers" "static" "[production]\\nprod1.example.com\\nprod2.example.com" "ubuntu" 22 $K3)
I3=$(create_inventory $P3 "Web Servers" "static" "[apps]\\napp1.example.com" "deploy" 22 $K5)

echo ""
echo -e "${YELLOW}=== Окружения ===${NC}"
E1=$(create_env $P1 "Development" "{\"ENV\": \"dev\", \"DEBUG\": \"true\"}")
E2=$(create_env $P1 "Production" "{\"ENV\": \"prod\", \"DEBUG\": \"false\"}")
E3=$(create_env $P2 "AWS" "{\"AWS_REGION\": \"us-east-1\"}")
E4=$(create_env $P2 "GCP" "{\"GOOGLE_PROJECT\": \"my-project\"}")
E5=$(create_env $P3 "Web App" "{\"APP_ENV\": \"production\"}")

echo ""
echo -e "${YELLOW}=== Шаблоны ===${NC}"
T1=$(create_template $P1 "Deploy Web App" "deploy.yml" $I1 $R1 $E1 "ansible")
T2=$(create_template $P1 "Run Tests" "test.yml" $I1 $R1 $E2 "ansible")
T3=$(create_template $P1 "Backup DB" "backup.yml" $I1 $R2 $E1 "ansible")
T4=$(create_template $P2 "Deploy Infra" "main.tf" $I2 $R3 $E3 "terraform")
T5=$(create_template $P2 "Create VPC" "vpc.tf" $I2 $R3 $E3 "terraform")
T6=$(create_template $P2 "Deploy EC2" "ec2.tf" $I2 $R4 $E3 "terraform")
T7=$(create_template $P3 "Deploy Nginx" "nginx-deploy.yml" $I3 $R5 $E5 "ansible")
T8=$(create_template $P3 "Update SSL" "ssl-update.yml" $I3 $R5 $E5 "ansible")
T9=$(create_template $P3 "Restart Services" "restart.sh" $I3 $R6 $E5 "shell")

echo ""
echo -e "${YELLOW}=== Задачи ===${NC}"
create_task $P1 $T1
create_task $P2 $T4
create_task $P3 $T7

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         Тестовые данные созданы успешно!               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Создано:"
echo "  • Проектов: 3"
echo "  • Ключей доступа: 5"
echo "  • Репозиториев: 6"
echo "  • Инвентарей: 3"
echo "  • Окружений: 5"
echo "  • Шаблонов: 9"
echo "  • Задач: 3"
echo ""
echo "Откройте веб-интерфейс: http://localhost:3000"
