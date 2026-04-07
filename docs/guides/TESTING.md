# 🧪 Тестирование Velum

## Запуск тестов

### Базовые тесты

```bash
cd rust

# Все тесты
cargo test

# Тесты конкретной библиотеки
cargo test --lib

# Тесты с выводом stdout
cargo test -- --nocapture

# Тесты с фильтром по имени
cargo test test_login
cargo test webhook
cargo test scheduler
```

### Результат тестов (актуально на 2026-04-07)

```
test result: ok. 1283 passed; 0 failed; 3 ignored
```

---

## Покрытие кода

### Запуск tarpaulin

```bash
cd rust

# HTML отчёт
cargo tarpaulin --out Html

# XML (Cobertura)
cargo tarpaulin --out Xml --output-dir .

# Консольный вывод
cargo tarpaulin
```

### Текущее покрытие

| Модуль | Покрытие | Строк |
|--------|----------|-------|
| **task_pool.rs** | ~57% | 86/150 |
| **webhook.rs** | ~85% | 184/215 |
| **task_pool_queue.rs** | 100% | 32/32 |
| **task_pool_types.rs** | 100% | 15/15 |
| **task_runner/details.rs** | 100% | 27/27 |
| **task_runner/errors.rs** | 100% | 27/27 |
| **task_runner/logging.rs** | 100% | 36/36 |
| **task_runner/types.rs** | 100% | 15/15 |
| **utils/shell.rs** | 100% | 13/13 |
| **utils/ansi.rs** | 100% | 7/7 |
| **totp.rs** | ~96% | 54/56 |
| **hooks.rs** | ~73% | 58/79 |
| **telegram_bot/mod.rs** | ~2% | 3/153 |
| **workflow_executor.rs** | ~16% | 31/188 |
| **scheduler.rs** | ~16% | 20/127 |
| **ssh_agent.rs** | ~36% | 87/242 |

**Общее покрытие:** ~16.5% (4517/27348 строк)

> ⚠️ **Примечание:** Tarpaulin на LLVM engine может неточно считать покрытие async-кода. Реальное покрытие unit-тестов выше.

---

## Типы тестов

### Unit-тесты

Расположены в `#[cfg(test)] mod tests` блоках каждого модуля:

```bash
# Unit-тесты конкретного модуля
cargo test api::handlers::tests::
cargo test services::webhook::tests::
cargo test db::sql::managers::test_project::tests::
```

### Integration-тесты

```bash
# Integration тесты API
cargo test --test api_integration --features integration-api-tests
```

### Doc-тесты

```bash
# Doctest
cargo test --doc
```

Некоторые примеры отмечены как `ignore` из-за необходимости внешнего окружения.

---

## CI

Тесты запускаются в GitHub Actions на каждый push и PR:

- **rust.yml**: `cargo check`, `cargo clippy`, `cargo test`
- **coverage.yml**: `cargo tarpaulin` с публикацией `cobertura.xml`

---

## Добавление новых тестов

### Unit-тест

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(my_function(42), 42);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = my_async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration-тест

```rust
// tests/api_integration.rs
#[cfg(feature = "integration-api-tests")]
mod tests {
    #[tokio::test]
    async fn test_login_flow() {
        let app = create_app().await;
        // ... тестирование через TestClient
    }
}
```
