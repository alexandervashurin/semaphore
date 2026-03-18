#!/bin/bash
# ============================================================================
# Тестирование CRUD Демо
# ============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; }

echo ""
echo "============================================================================"
echo "                   Velum - CRUD Демо Тесты"
echo "============================================================================"
echo ""

# Проверка файлов
info "Проверка файлов демо..."

FILES=(
    "web/public/demo-crud.html"
    "web/public/demo-crud.js"
    "web/public/demo-styles.css"
    "CRUD_DEMO.md"
    "CRUD_DEMO_CHEATSHEET.md"
    "demo-start.sh"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        size=$(ls -lh "$file" | awk '{print $5}')
        success "$file ($size)"
    else
        error "$file не найден"
        exit 1
    fi
done

echo ""

# Проверка Docker
info "Проверка Docker сервисов..."

if docker-compose ps | grep -q "semaphore-db"; then
    success "PostgreSQL запущен"
else
    warning "PostgreSQL не запущен"
fi

if docker-compose ps | grep -q "semaphore-frontend"; then
    success "Frontend (Nginx) запущен"
else
    warning "Frontend не запущен"
fi

echo ""

# Проверка API
info "Проверка API endpoints..."

# Проверка доступности API
if curl -s http://localhost:3000/api > /dev/null 2>&1; then
    success "API доступно (http://localhost:3000/api)"
else
    warning "API недоступно (возможно backend не запущен)"
fi

# Проверка демо-страницы через Nginx
if curl -s http://localhost/demo-crud.html > /dev/null 2>&1; then
    success "Демо-страница доступна (http://localhost/demo-crud.html)"
else
    warning "Демо-страница недоступна через Nginx"
fi

echo ""

# Проверка БД
info "Проверка БД..."

if docker-compose exec -T db pg_isready -U semaphore -d semaphore > /dev/null 2>&1; then
    success "PostgreSQL готов к работе"
    
    # Проверка наличия демо-данных
    USERS_COUNT=$(docker-compose exec -T db psql -U semaphore -d semaphore -t -c "SELECT COUNT(*) FROM \"user\";" 2>/dev/null | tr -d ' ')
    if [ -n "$USERS_COUNT" ] && [ "$USERS_COUNT" -gt 0 ]; then
        success "Демо-пользователи загружены ($USERS_COUNT)"
    else
        warning "Демо-пользователи не найдены"
    fi
    
    PROJECTS_COUNT=$(docker-compose exec -T db psql -U semaphore -d semaphore -t -c "SELECT COUNT(*) FROM project;" 2>/dev/null | tr -d ' ')
    if [ -n "$PROJECTS_COUNT" ] && [ "$PROJECTS_COUNT" -gt 0 ]; then
        success "Демо-проекты загружены ($PROJECTS_COUNT)"
    else
        warning "Демо-проекты не найдены"
    fi
else
    warning "PostgreSQL недоступен"
fi

echo ""

# Итоги
echo "============================================================================"
echo "                              Итоги"
echo "============================================================================"
echo ""
echo "📍 URLs для доступа:"
echo "   - Демо-страница: http://localhost/demo-crud.html"
echo "   - Frontend (Nginx): http://localhost/"
echo "   - Backend API: http://localhost:3000/api"
echo "   - Swagger: http://localhost:3000/swagger"
echo ""
echo "👤 Учетные данные:"
echo "   - admin / demo123 (Администратор)"
echo "   - john.doe / demo123 (Менеджер)"
echo "   - jane.smith / demo123 (Менеджер)"
echo "   - devops / demo123 (Исполнитель)"
echo ""
echo "📖 Документация:"
echo "   - CRUD_DEMO.md - полное руководство"
echo "   - CRUD_DEMO_CHEATSHEET.md - шпаргалка"
echo ""
echo "============================================================================"
echo ""
