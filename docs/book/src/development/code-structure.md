# Структура кода

> Организация модулей Rust в Velum
>
> 📖 См. также: [Настройка окружения](./dev-setup.md), [Руководство по тестированию](./testing.md), [Rust API](./rust-api.md), [Обзор системы](../architecture/system-overview.md)

---

## Основные модули

```
rust/src/
├── api/              # HTTP-обработчики и маршруты
│   ├── handlers/     # Обработчики эндпоинтов
│   ├── middleware/   # Промежуточное ПО (auth, cache)
│   ├── routes/       # Определение маршрутов
│   └── state.rs      # Состояние приложения
├── config/           # Загрузка конфигурации
├── db/               # Работа с базой данных
│   ├── sql/          # SQL-запросы и менеджеры
│   └── store.rs      # Хранилище данных
├── models/           # Структуры данных (User, Project, Task...)
├── services/         # Бизнес-логика
│   ├── task_pool.rs  # Очередь задач
│   ├── scheduler.rs  # Планировщик
│   └── cache_service.rs  # Кэширование
├── kubernetes/       # K8s-клиент
├── error.rs          # Типы ошибок
├── logging.rs        # Настройка логирования
└── lib.rs            # Корень библиотеки
```

## Стиль кода

- **Форматирование**: `cargo fmt --all`
- **Линтер**: `cargo clippy --all-features -- -D warnings`
- **Именование**: snake_case для функций, PascalCase для типов
- **Коммиты**: Conventional Commits (`feat:`, `fix:`, `docs:`)

---

## Следующие шаги

- [Настройка окружения](./dev-setup.md) — локальное окружение
- [Rust API](./rust-api.md) — сгенерированная документация
- [Обзор системы](../architecture/system-overview.md) — архитектура целиком
