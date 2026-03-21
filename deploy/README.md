# Deploy — Варианты развёртывания Velum

Выбери подходящий вариант:

| Папка | Для кого | БД | Команда |
|---|---|---|---|
| [`demo/`](demo/) | Попробовать Velum | SQLite | `docker compose up --build -d` |
| [`dev/`](dev/) | Разработка | PostgreSQL | `docker compose up -d` |
| [`prod/`](prod/) | Продакшен | PostgreSQL | `docker compose up -d --build` |

## Быстрый старт

### Demo (без настройки)

```bash
cd deploy/demo
cp .env.example .env
docker compose up --build -d
# → http://localhost:8088 (admin / admin123)
```

### Dev (разработка с hot-reload)

```bash
cd deploy/dev
cp .env.example .env
docker compose up -d
# → http://localhost:3000
```

### Prod (продакшен)

```bash
cd deploy/prod
cp .env.example .env && nano .env   # сменить пароли!
docker compose up -d --build
# → http://localhost:80
```

## Нативный запуск (без Docker)

```bash
# Только backend, SQLite
cd rust
SEMAPHORE_DB_PATH=/tmp/semaphore.db \
SEMAPHORE_WEB_PATH=../web/public \
SEMAPHORE_ADMIN=admin \
SEMAPHORE_ADMIN_PASSWORD=admin123 \
cargo run -- server

# Или используй start-server.sh из корня
bash ../../start-server.sh
```

## Полная документация

- [DEPLOY.md](../DEPLOY.md) — полное руководство по деплою
- [docs/DEPLOYMENT.md](../docs/DEPLOYMENT.md) — расширенные опции
- [docs/DEB_PACKAGE.md](../docs/DEB_PACKAGE.md) — установка через DEB-пакет
