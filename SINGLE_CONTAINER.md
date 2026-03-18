# 🐳 Velum - Запуск одним Docker-контейнером

> **All-in-One решение:** Backend + Frontend + Nginx + SQLite в одном контейнере

---

## 📋 Содержание

1. [Быстрый старт](#быстрый-старт)
2. [Сборка образа](#сборка-образа)
3. [Запуск контейнера](#запуск-контейнера)
4. [Конфигурация](#конфигурация)
5. [Управление](#управление)
6. [Производительность](#производительность)

---

## 🚀 Быстрый старт

### Вариант 1: Готовый образ (рекомендуется)

```bash
# Запуск контейнера
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v semaphore_data:/app/data \
  -e SEMAPHORE_DB_URL="sqlite://data/semaphore.db" \
  --restart unless-stopped \
  ghcr.io/alexandervashurin/semaphore:latest

# Проверка
curl http://localhost/health

# Остановка
docker stop semaphore && docker rm semaphore
```

### Вариант 2: Сборка из исходников

```bash
# Сборка образа
docker build -f deployment/single/Dockerfile -t semaphore:latest .

# Запуск
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v semaphore_data:/app/data \
  semaphore:latest
```

### Вариант 3: Docker Compose

```bash
# Запуск
docker-compose -f docker-compose.single.yml up -d

# Проверка логов
docker-compose -f docker-compose.single.yml logs -f

# Остановка
docker-compose -f docker-compose.single.yml down
```

---

## 🔨 Сборка образа

### Полная сборка

```bash
# Сборка образа с тегом
docker build \
  -f deployment/single/Dockerfile \
  -t semaphore:latest \
  -t semaphore:0.1.0 \
  .

# Проверка размера
docker images semaphore
```

### Мульти-архитектурная сборка

```bash
# Установка QEMU
docker run --rm --privileged multiarch/qemu-user-static --reset -p yes

# Сборка для amd64 и arm64
docker buildx build \
  -f deployment/single/Dockerfile \
  -t semaphore:latest \
  --platform linux/amd64,linux/arm64 \
  --push \
  .
```

### Оптимизированная сборка (musl)

```bash
# Сборка с musl для минимального размера
docker build \
  -f deployment/single/Dockerfile.musl \
  -t semaphore:alpine \
  .
```

---

## ▶️ Запуск контейнера

### Базовый запуск

```bash
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v semaphore_data:/app/data \
  -e SEMAPHORE_DB_URL="sqlite://data/semaphore.db" \
  -e SEMAPHORE_HOST="0.0.0.0" \
  -e SEMAPHORE_PORT="3000" \
  -e RUST_LOG="info" \
  --restart unless-stopped \
  semaphore:latest
```

### С персистентными томами

```bash
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/config:/app/config \
  -v $(pwd)/logs:/app/logs \
  -e SEMAPHORE_DB_URL="sqlite://data/semaphore.db" \
  --restart unless-stopped \
  semaphore:latest
```

### С сетью host

```bash
docker run -d \
  --name semaphore \
  --network host \
  -v semaphore_data:/app/data \
  -e SEMAPHORE_DB_URL="sqlite://data/semaphore.db" \
  --restart unless-stopped \
  semaphore:latest
```

### С ограничением ресурсов

```bash
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v semaphore_data:/app/data \
  --cpus="2.0" \
  --memory="512m" \
  --memory-reservation="128m" \
  -e SEMAPHORE_DB_URL="sqlite://data/semaphore.db" \
  --restart unless-stopped \
  semaphore:latest
```

---

## ⚙️ Конфигурация

### Переменные окружения

| Переменная | По умолчанию | Описание |
|-----------|-------------|----------|
| `SEMAPHORE_DB_URL` | `sqlite://data/semaphore.db` | URL подключения к БД |
| `SEMAPHORE_WEB_PATH` | `/var/www/html` | Путь к frontend файлам |
| `SEMAPHORE_HOST` | `0.0.0.0` | Хост для прослушивания |
| `SEMAPHORE_PORT` | `3000` | Порт backend API |
| `RUST_LOG` | `info` | Уровень логирования |

### Форматы DB_URL

**SQLite:**
```
sqlite://data/semaphore.db
sqlite:///var/lib/semaphore/db.sqlite
```

**PostgreSQL:**
```
postgres://user:password@host:5432/dbname
postgres://semaphore:pass@db:5432/semaphore
```

**MySQL:**
```
mysql://user:password@host:3306/dbname
mysql://semaphore:pass@mysql:3306/semaphore
```

### Примеры конфигурации

**Production с PostgreSQL:**
```bash
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v semaphore_data:/app/config \
  -e SEMAPHORE_DB_URL="postgres://semaphore:pass@db:5432/semaphore" \
  -e SEMAPHORE_WEB_PATH="/var/www/html" \
  --link postgres:db \
  --restart unless-stopped \
  semaphore:latest
```

**Development с отладкой:**
```bash
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v $(pwd)/data:/app/data \
  -e RUST_LOG="debug,semaphore=trace" \
  --restart unless-stopped \
  semaphore:latest
```

---

## 🎛️ Управление

### Команды Docker

```bash
# Статус
docker ps -a | grep semaphore

# Логи
docker logs semaphore
docker logs -f semaphore
docker logs --tail 100 semaphore

# Статистика
docker stats semaphore

# Перезапуск
docker restart semaphore

# Остановка
docker stop semaphore

# Удаление
docker rm semaphore

# Очистка volumes
docker volume rm semaphore_data semaphore_config semaphore_logs
```

### Docker Compose команды

```bash
# Запуск
docker-compose -f docker-compose.single.yml up -d

# Остановка
docker-compose -f docker-compose.single.yml down

# Пересборка
docker-compose -f docker-compose.single.yml up -d --build

# Логи
docker-compose -f docker-compose.single.yml logs -f

# Статус
docker-compose -f docker-compose.single.yml ps
```

### Healthcheck

```bash
# Проверка здоровья
docker inspect --format='{{.State.Health.Status}}' semaphore

# Детальная информация
docker inspect semaphore

# Тест health endpoint
curl http://localhost/health
```

---

## 📊 Производительность

### Размеры образов

| Образ | Размер | Описание |
|------|--------|----------|
| `semaphore:latest` | ~450 MB | Стандартный (Debian) |
| `semaphore:alpine` | ~150 MB | Оптимизированный (Alpine) |
| `semaphore:distroless` | ~80 MB | Минимальный (Distroless) |

### Потребление ресурсов

| Режим | CPU | RAM | Описание |
|------|-----|-----|----------|
| **Idle** | <1% | ~50 MB | Без активных задач |
| **Normal** | 5-10% | ~150 MB | Несколько активных задач |
| **Heavy** | 50-100% | ~300 MB | Множество параллельных задач |

### Оптимизация

**1. Использование musl libc:**
```dockerfile
FROM rust:alpine AS builder
# ...
FROM alpine:latest
# ...
```

**2. Multi-stage build:**
```dockerfile
FROM rust:1.75-slim AS builder
# ...
FROM debian:bookworm-slim
# ...
```

**3. Удаление отладочной информации:**
```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

---

## 🔧 Troubleshooting

### Контейнер не запускается

```bash
# Проверка логов
docker logs semaphore

# Проверка прав доступа
docker exec semaphore ls -la /app/data

# Проверка порта
docker port semaphore
```

### Проблемы с базой данных

```bash
# Проверка файла БД
docker exec semaphore ls -la /app/data

# Резервное копирование
docker exec semaphore tar -czf /tmp/db-backup.tar.gz /app/data
docker cp semaphore:/tmp/db-backup.tar.gz ./db-backup.tar.gz

# Восстановление
docker cp ./semaphore.db semaphore:/app/data/
```

### Проблемы с frontend

```bash
# Проверка nginx
docker exec semaphore nginx -t

# Перезапуск nginx
docker exec semaphore nginx -s reload

# Проверка файлов
docker exec semaphore ls -la /var/www/html
```

---

## 📝 Примеры использования

### CI/CD Pipeline

```yaml
# .github/workflows/docker.yml
name: Docker Build

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build image
        run: |
          docker build -f deployment/single/Dockerfile \
            -t semaphore:${{ github.ref_name }} \
            -t semaphore:latest .
      
      - name: Push to registry
        run: |
          docker push semaphore:${{ github.ref_name }}
          docker push semaphore:latest
```

### Docker Compose Stack

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  semaphore:
    image: semaphore:latest
    container_name: semaphore
    ports:
      - "80:80"
    volumes:
      - semaphore_data:/app/data
    environment:
      - SEMAPHORE_DB_URL=sqlite://data/semaphore.db
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  watchtower:
    image: containrrr/watchtower
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    command: --interval 3600 semaphore
    restart: unless-stopped

volumes:
  semaphore_data:
```

---

## 📞 Поддержка

- **Документация:** [ROADMAP.md](ROADMAP.md), [README.md](README.md)
- **GitHub Issues:** https://github.com/alexandervashurin/semaphore/issues
- **Email:** alexandervashurin@yandex.ru

---

*Последнее обновление: 8 марта 2026 г.*
