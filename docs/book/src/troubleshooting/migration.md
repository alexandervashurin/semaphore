# Руководство по миграции

> Обновление между версиями Velum
>
> 📖 См. также: [Типовые проблемы](./common-issues.md), [Список изменений](../resources/changelog.md), [Релизы](../resources/releases.md)

---

## Автоматические миграции

Velum применяет миграции базы данных автоматически при запуске. Дополнительных действий не требуется.

## Обновление Docker

```bash
# Остановка
docker compose -f docker-compose.yml down

# Обновление образа
docker pull ghcr.io/alexandervashurin/semaphore:latest

# Запуск
docker compose -f docker-compose.yml up -d
```

## Обновление из исходного кода

```bash
git pull origin main
cd rust && cargo build --release
# Перезапуск сервера
```

## Проверка после обновления

```bash
# Проверка здоровья
curl http://localhost:3000/healthz

# Проверка версии (если доступна)
curl http://localhost:3000/api/health
```

## Откат

При проблемах можно вернуться к предыдущей версии:

```bash
# Docker
docker pull ghcr.io/alexandervashurin/semaphore:<previous-tag>

# Миграции БД обратно не откатываются автоматически
```

---

## Следующие шаги

- [Типовые проблемы](./common-issues.md) — частые проблемы
- [Список изменений](../resources/changelog.md) — что изменилось
- [Релизы](../resources/releases.md) — страница релизов
