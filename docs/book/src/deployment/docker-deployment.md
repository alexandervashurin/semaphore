# Развёртывание в Docker

> Все варианты развёртывания через Docker Compose
>
> 📖 См. также: [Быстрый старт](../getting-started/quick-start.md), [Конфигурация](../getting-started/configuration.md), [Kubernetes](./kubernetes-deployment.md), [Продакшен](./production-setup.md)

---

## Доступные файлы Compose

| Файл | Назначение | Порт |
|------|-----------|------|
| `docker-compose.demo.yml` | Быстрое демо с демо-данными | 8088 |
| `docker-compose.dev.yml` | Разработка с горячей перезагрузкой | 3000 |
| `docker-compose.yml` | Стандартный продакшен | 3000 |
| `docker-compose.prod.yml` | Продакшен с усилением | 3000 |
| `docker-compose.postgres.yml` | Только PostgreSQL | — |

---

## Демо-режим

Самый быстрый способ попробовать Velum:

```bash
docker compose -f docker-compose.demo.yml up -d
```

Открывается на http://localhost:8088 — Логин: `admin` / `admin123`

---

## Режим разработки

С горячей перезагрузкой кода:

```bash
docker compose -f docker-compose.dev.yml up -d
```

---

## Продакшен-режим

```bash
docker compose -f docker-compose.prod.yml up -d
```

Включает:
- Проверки здоровья (health checks)
- Лимиты ресурсов
- Политики перезапуска
- Сохранение томов (volume persistence)

---

## Docker-образ

Оптимизированный образ весит **~23 МБ** (FROM scratch + общие библиотеки):

```bash
docker pull ghcr.io/alexandervashurin/semaphore:latest
```

### Поддержка мультиархитектуры

| Платформа | Тег |
|-----------|-----|
| Linux amd64 | `latest`, `linux-amd64` |
| Linux arm64 | `latest`, `linux-arm64` |

---

## Своя сборка

```bash
docker build -t velum:local .
```

---

## Следующие шаги

- [Kubernetes](./kubernetes-deployment.md) — развёртывание в K8s
- [Продакшен](./production-setup.md) — усиление для продакшена
- [Конфигурация](../getting-started/configuration.md) — переменные окружения
- [Telegram-бот](../resources/telegram-bot.md) — настройка уведомлений
