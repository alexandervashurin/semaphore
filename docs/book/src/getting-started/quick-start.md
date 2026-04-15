# Быстрый старт

> **⏱️ 5 минут** — от нуля до работающего экземпляра Velum
>
> 📖 См. также: [Конфигурация](./configuration.md), [Первый проект](./first-project.md), [Docker](../deployment/docker-deployment.md)

---

## Требования

- Docker и Docker Compose **или** Rust 1.80+
- PostgreSQL 15+ (если не используете Docker)

---

## Вариант 1: Docker (рекомендуется)

### Демо-режим

```bash
docker compose -f docker-compose.demo.yml up -d
```

Откройте http://localhost:8088 — Логин: `admin` / `admin123`

### Режим разработки

```bash
docker compose -f docker-compose.dev.yml up -d
```

Откройте http://localhost:3000

---

## Вариант 2: Из исходного кода

```bash
# Клонирование
git clone https://github.com/alexandervashurin/semaphore.git
cd semaphore

# Сборка
cd rust && cargo build --release

# Запуск
VELUM_DB_DIALECT=postgres \
VELUM_DB_URL="postgres://user:pass@localhost:5432/velum" \
VELUM_JWT_SECRET="your-secret-key-32-bytes-long!!" \
VELUM_WEB_PATH="../web/public" \
VELUM_ADMIN=admin \
VELUM_ADMIN_PASSWORD=admin123 \
./target/release/velum server --host 0.0.0.0 --port 3000
```

---

## Проверка установки

```bash
# Проверка здоровья
curl http://localhost:3000/healthz

# Проверка готовности
curl http://localhost:3000/readyz

# Вход
curl -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin123"}'
```

---

## Следующие шаги

1. [Конфигурация](./configuration.md) — настройка окружения
2. [Первый проект](./first-project.md) — создайте проект и запустите первую задачу
3. [Docker](../deployment/docker-deployment.md) — варианты развёртывания
4. [REST API](../api-reference/rest-api.md) — изучите полное API

---

## Поддержка платформ

| Платформа | Архитектура | Статус |
|-----------|-------------|--------|
| Linux | amd64, arm64 | ✅ Нативно |
| macOS | amd64 (Intel), arm64 (Apple Silicon) | ✅ Нативно |
| Docker | linux/amd64, linux/arm64 | ✅ Мультиархитектура |
| Kubernetes | amd64, arm64 | ✅ Helm + манифесты |
