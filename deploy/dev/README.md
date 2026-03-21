# Dev — Среда разработки Velum

PostgreSQL + backend с **автоматической перекомпиляцией** при изменении кода.

## Запуск

```bash
# Из папки deploy/dev/
cp .env.example .env
docker compose up -d

# Посмотреть логи backend
docker compose logs -f backend
```

URL: **http://localhost:3000**

## Рабочий процесс

1. Правки в `rust/src/` → backend автоматически перекомпилируется (cargo-watch)
2. Правки в `web/public/` → видны сразу в браузере (том монтируется напрямую)
3. Для горячей замены HTML без перезапуска:
   ```bash
   docker cp web/public/file.html semaphore-backend-dev:/app/web/public/file.html
   ```

## Нативный запуск (без Docker)

Быстрее для разработки — только DB в Docker:

```bash
# Запустить только PostgreSQL
docker compose up -d db

# Backend на хосте
cd ../../rust
SEMAPHORE_DB_URL="postgres://semaphore:semaphore_dev_pass@localhost:5432/semaphore" \
SEMAPHORE_WEB_PATH=../web/public \
SEMAPHORE_ADMIN=admin \
SEMAPHORE_ADMIN_PASSWORD=admin123 \
cargo watch -x 'run -- server'
```

## Переменные окружения

| Переменная | По умолчанию | Описание |
|---|---|---|
| `POSTGRES_PASSWORD` | `semaphore_dev_pass` | Пароль PostgreSQL |
| `SEMAPHORE_JWT_SECRET` | `dev-jwt-secret` | JWT секрет |
| `RUST_LOG` | `semaphore=debug,...` | Уровень логирования |
| `BACKEND_PORT` | `3000` | Порт backend |
| `DB_PORT` | `5432` | Порт PostgreSQL |
