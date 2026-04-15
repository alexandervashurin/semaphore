# Режим отладки

> Включение подробного логирования и отладка Velum
>
> 📖 См. также: [Типовые проблемы](./common-issues.md), [Конфигурация](../getting-started/configuration.md), [Настройка окружения](../development/dev-setup.md)

---

## Уровни логирования

```bash
# Минимальный
RUST_LOG=info

# Подробный
RUST_LOG=velum=debug

# Максимальный
RUST_LOG=velum=debug,tower_http=debug,sqlx=warn
```

## Структура логов

Velum использует структурированное логирование:

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "INFO",
  "target": "velum::api::handlers",
  "message": "Запрос выполнен",
  "method": "GET",
  "path": "/api/projects",
  "status": 200,
  "duration_ms": 42
}
```

## Отладка API-запросов

Включите middleware логирование:

```bash
RUST_LOG=tower_http=debug
```

## Отладка базы данных

```bash
RUST_LOG=sqlx=debug
```

Покажет все SQL-запросы с параметрами.

---

## Следующие шаги

- [Типовые проблемы](./common-issues.md) — частые проблемы
- [Конфигурация](../getting-started/configuration.md) — переменные окружения
- [Настройка окружения](../development/dev-setup.md) — локальное окружение
