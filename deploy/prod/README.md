# Prod — Production-деплой Velum

PostgreSQL + Rust backend + Nginx. Все сервисы изолированы в Docker сетях.

## Требования

- Docker 24+ и Docker Compose v2
- 2 CPU / 2 GB RAM минимум
- Свободный порт 80 (или 443 для HTTPS)

## Первый запуск

```bash
# 1. Из папки deploy/prod/ — скопировать и заполнить переменные
cp .env.example .env
nano .env   # ОБЯЗАТЕЛЬНО сменить все пароли!

# 2. Запустить
docker compose up -d --build

# 3. Проверить
docker compose ps
curl http://localhost/api/health
```

Открыть: **http://localhost** (или ваш домен)

## Обновление

```bash
git pull origin main
docker compose up -d --build
```

## Безопасность — обязательно перед продакшеном

| Параметр | Требование |
|---|---|
| `POSTGRES_PASSWORD` | Минимум 16 символов, случайный |
| `SEMAPHORE_JWT_SECRET` | Минимум 32 символа: `openssl rand -hex 32` |
| `SEMAPHORE_ADMIN_PASSWORD` | Минимум 12 символов |
| PostgreSQL порт | **Не экспонировать** в docker-compose (закомментировано) |
| HTTPS | Рекомендуется Let's Encrypt + certbot |

## HTTPS с Let's Encrypt

```bash
# Получить сертификат
apt install certbot
certbot certonly --standalone -d semaphore.your-domain.com

# Раскомментировать HTTPS блок в nginx.conf
# Добавить в docker-compose.yml HTTPS_PORT: 443
docker compose restart nginx
```

## Команды управления

```bash
# Логи
docker compose logs -f backend
docker compose logs -f nginx

# Перезапуск отдельного сервиса
docker compose restart backend

# Резервная копия БД
docker exec semaphore-db pg_dump -U semaphore semaphore > backup_$(date +%Y%m%d).sql

# Восстановление БД
docker exec -i semaphore-db psql -U semaphore semaphore < backup.sql

# Полная остановка
docker compose down

# Остановка с удалением данных (ОСТОРОЖНО!)
docker compose down -v
```

## Структура сетей

```
Internet → Nginx (80/443)
              ↓
        [external network]
              ↓
          Backend (:3000)
              ↓
        [internal network]
              ↓
         PostgreSQL (:5432)  ← не доступен извне
```
