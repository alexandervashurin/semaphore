# Документация Rust API

> Автоматически сгенерированная документация Rust API Velum
>
> 📖 См. также: [Структура кода](./code-structure.md), [Настройка окружения](./dev-setup.md), [REST API](../api-reference/rest-api.md)

---

## Онлайн-версия

👉 [Посмотреть Rust API Docs](../rustdoc/index.html)

Документация генерируется через `cargo doc` и доступна на GitHub Pages.

## Локальная генерация

```bash
cd rust
cargo doc --no-deps --document-private-items --open
```

Документация откроется в браузере по пути `rust/target/doc/index.html`.

## Что включено

- Все публичные функции и типы
- Модули `api`, `config`, `db`, `models`, `services`, `kubernetes`
- Комментарии к функциям и структурам

---

## Следующие шаги

- [Структура кода](./code-structure.md) — организация модулей
- [REST API](../api-reference/rest-api.md) — HTTP API-эндпоинты
- [Настройка окружения](./dev-setup.md) — локальное окружение
