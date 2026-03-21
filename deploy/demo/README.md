# Demo — Быстрый запуск Velum

Самый простой способ запустить Velum. Использует **SQLite**, не требует настройки.

## Запуск за 1 минуту

```bash
# Из папки deploy/demo/
cp .env.example .env
docker compose up --build -d
```

Открой: **http://localhost:8088**
Логин: **admin** / **admin123**

## Наполнение тестовыми данными

```bash
# Из корня репозитория
bash fill-sqlite-demo-data.sh
```

## Остановка

```bash
docker compose down        # остановить контейнеры
docker compose down -v     # остановить + удалить данные
```

## Переменные окружения

| Переменная | По умолчанию | Описание |
|---|---|---|
| `VELUM_PORT` | `8088` | Порт для доступа к UI |
| `SEMAPHORE_ADMIN` | `admin` | Логин администратора |
| `SEMAPHORE_ADMIN_PASSWORD` | `admin123` | Пароль администратора |
| `SEMAPHORE_JWT_SECRET` | `demo-jwt-secret` | JWT секрет |

> ⚠️ Demo-режим предназначен только для ознакомления. Для продакшена используй `../prod/`.
