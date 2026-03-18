#!/bin/bash
# Скрипт заполнения БД через API

BASE_URL="http://localhost:3000/api"
LOGIN="admin"
PASSWORD="demo123"

echo "📊 Заполнение Velum тестовыми данными через API"
echo "======================================================="
echo ""

# Авторизация
echo "🔐 Авторизация..."
AUTH_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$LOGIN\",\"password\":\"$PASSWORD\",\"expire\":true}")

TOKEN=$(echo "$AUTH_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin).get('token',''))" 2>/dev/null)

if [ -z "$TOKEN" ]; then
    echo "❌ Ошибка авторизации: $AUTH_RESPONSE"
    exit 1
fi

echo "✅ Токен получен"
echo ""

# Функция для API запросов
api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ "$method" = "GET" ]; then
        curl -s "$BASE_URL$endpoint" -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json"
    else
        curl -s -X "$method" "$BASE_URL$endpoint" \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/json" \
            -d "$data"
    fi
}

# Создание проектов
echo "📁 Создание проектов..."
PROJECTS=(
    "Web Application|Основной веб-проект"
    "Database Migration|Миграция БД на PostgreSQL"
    "Infrastructure|Инфраструктурные скрипты"
    "CI/CD Pipeline|Пайплайны непрерывной интеграции"
    "Monitoring Setup|Настройка мониторинга"
)

for proj_data in "${PROJECTS[@]}"; do
    IFS='|' read -r name desc <<< "$proj_data"
    response=$(api_call "POST" "/projects" "{\"name\":\"$name\",\"description\":\"$desc\"}")
    proj_id=$(echo "$response" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('id',d.get('ID','?')))" 2>/dev/null)
    if [ "$proj_id" != "?" ] && [ -n "$proj_id" ]; then
        echo "  ✅ $name (ID: $proj_id)"
    else
        echo "  ⚠️  $name (возможно уже существует)"
    fi
done

echo ""

# Получение списка проектов
echo "📋 Получение списка проектов..."
ALL_PROJECTS=$(api_call "GET" "/projects")
PROJ_COUNT=$(echo "$ALL_PROJECTS" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d) if isinstance(d,list) else 0)" 2>/dev/null)
echo "  Всего проектов: $PROJ_COUNT"
echo ""

# Создание шаблонов
echo "📋 Создание шаблонов..."
TEMPLATES=(
    "1|Deploy to Production|deploy.yml"
    "1|Run Tests|test.yml"
    "2|Backup Database|backup.yml"
    "2|Migrate Schema|migrate.yml"
    "3|Setup Server|setup.yml"
    "4|Deploy App|deploy-app.yml"
    "4|Run Lint|lint.yml"
    "5|Setup Prometheus|prometheus.yml"
)

for tpl_data in "${TEMPLATES[@]}"; do
    IFS='|' read -r proj_id name playbook <<< "$tpl_data"
    response=$(api_call "POST" "/templates" "{\"name\":\"$name\",\"project_id\":$proj_id,\"playbook\":\"$playbook\",\"type\":\"playbook\"}")
    tpl_id=$(echo "$response" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('id',d.get('ID','?')))" 2>/dev/null)
    if [ "$tpl_id" != "?" ] && [ -n "$tpl_id" ]; then
        echo "  ✅ $name (Project: $proj_id, ID: $tpl_id)"
    else
        echo "  ⚠️  $name (возможно уже существует)"
    fi
done

echo ""

# Получение списка шаблонов
echo "📊 Получение списка шаблонов..."
ALL_TEMPLATES=$(api_call "GET" "/templates")
TPL_COUNT=$(echo "$ALL_TEMPLATES" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d) if isinstance(d,list) else 0)" 2>/dev/null)
echo "  Всего шаблонов: $TPL_COUNT"
echo ""

# Создание задач для первых 3 шаблонов
echo "✅ Создание тестовых задач..."
TEMPLATE_IDS=$(echo "$ALL_TEMPLATES" | python3 -c "import sys,json; d=json.load(sys.stdin); print(' '.join([str(t.get('id',t.get('ID')) for t in d[:3]]) if isinstance(d,list) else '')" 2>/dev/null)

for tpl_id in $TEMPLATE_IDS; do
    response=$(api_call "POST" "/tasks" "{\"template_id\":$tpl_id}")
    task_id=$(echo "$response" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('id',d.get('ID','?')))" 2>/dev/null)
    if [ "$task_id" != "?" ] && [ -n "$task_id" ]; then
        echo "  ✅ Задача создана (Template: $tpl_id, Task ID: $task_id)"
    else
        echo "  ⚠️  Ошибка создания задачи для шаблона $tpl_id"
    fi
done

echo ""
echo "🎉 Заполнение завершено!"
echo ""
echo "📍 Откройте http://localhost:3000 и войдите как admin/demo123"
