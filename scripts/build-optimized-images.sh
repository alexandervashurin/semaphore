#!/bin/bash
# =============================================================================
# Velum - Скрипт сборки оптимизированных Docker образов
# =============================================================================
# Использование:
#   ./scripts/build-optimized-images.sh [tag]
#
# Пример:
#   ./scripts/build-optimized-images.sh latest
#   ./scripts/build-optimized-images.sh v0.1.0
# =============================================================================

set -e

TAG=${1:-latest}
REGISTRY=${2:-""}

echo "=============================================="
echo "Velum - Сборка оптимизированных образов"
echo "=============================================="
echo "Tag: $TAG"
echo "Registry: ${REGISTRY:-<none>}"
echo ""

# Функция для получения размера образа
get_image_size() {
    docker images $1 --format "{{.Size}}"
}

# Функция для вывода сравнения размеров
print_size_comparison() {
    echo ""
    echo "=============================================="
    echo "Сравнение размеров образов:"
    echo "=============================================="
    echo ""
    
    STANDARD=$(docker images semaphore:standard --format "{{.Size}}" 2>/dev/null || echo "N/A")
    SLIM=$(docker images semaphore:slim --format "{{.Size}}" 2>/dev/null || echo "N/A")
    ALPINE=$(docker images semaphore:alpine --format "{{.Size}}" 2>/dev/null || echo "N/A")
    DISTROLESS=$(docker images semaphore:distroless --format "{{.Size}}" 2>/dev/null || echo "N/A")
    
    printf "%-20s %s\n" "Standard (Debian):" "$STANDARD"
    printf "%-20s %s\n" "Slim (Debian):" "$SLIM"
    printf "%-20s %s\n" "Alpine (musl):" "$ALPINE"
    printf "%-20s %s\n" "Distroless:" "$DISTROLESS"
    echo ""
}

# 1. Сборка стандартного образа (для сравнения)
echo "[1/4] Сборка стандартного образа..."
docker build -f Dockerfile -t semaphore:standard -t semaphore:$TAG-standard .

# 2. Сборка Slim образа
echo "[2/4] Сборка Slim образа..."
docker build -f deployment/Dockerfile.slim -t semaphore:slim -t semaphore:$TAG-slim .

# 3. Сборка Alpine образа
echo "[3/4] Сборка Alpine образа..."
docker build -f deployment/Dockerfile.alpine -t semaphore:alpine -t semaphore:$TAG-alpine .

# 4. Сборка Distroless образа
echo "[4/4] Сборка Distroless образа..."
docker build -f deployment/Dockerfile.distroless -t semaphore:distroless -t semaphore:$TAG-distroless .

# Вывод размеров
print_size_comparison

# Push в registry (если указан)
if [ -n "$REGISTRY" ]; then
    echo ""
    echo "=============================================="
    echo "Push образов в registry: $REGISTRY"
    echo "=============================================="
    
    docker tag semaphore:slim $REGISTRY/semaphore:slim
    docker tag semaphore:slim $REGISTRY/semaphore:$TAG-slim
    docker push $REGISTRY/semaphore:slim
    docker push $REGISTRY/semaphore:$TAG-slim
    
    docker tag semaphore:alpine $REGISTRY/semaphore:alpine
    docker tag semaphore:alpine $REGISTRY/semaphore:$TAG-alpine
    docker push $REGISTRY/semaphore:alpine
    docker push $REGISTRY/semaphore:$TAG-alpine
    
    docker tag semaphore:distroless $REGISTRY/semaphore:distroless
    docker tag semaphore:distroless $REGISTRY/semaphore:$TAG-distroless
    docker push $REGISTRY/semaphore:distroless
    docker push $REGISTRY/semaphore:$TAG-distroless
    
    echo ""
    echo "Образы загружены в $REGISTRY"
fi

echo ""
echo "=============================================="
echo "Сборка завершена успешно!"
echo "=============================================="
echo ""
echo "Использование:"
echo ""
echo "  # Slim образ (баланс размера и совместимости)"
echo "  docker run -d --name semaphore -p 80:3000 semaphore:slim"
echo ""
echo "  # Alpine образ (минимальный размер)"
echo "  docker run -d --name semaphore -p 80:3000 semaphore:alpine"
echo ""
echo "  # Distroless образ (максимальная безопасность)"
echo "  docker run -d --name semaphore -p 80:3000 semaphore:distroless"
echo ""
