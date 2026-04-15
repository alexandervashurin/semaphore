# Руководство по тестированию

> Написание и запуск тестов в Velum
>
> 📖 См. также: [Настройка окружения](./dev-setup.md), [Структура кода](./code-structure.md), [Участие в проекте](../contributing/contributing.md)

---

## Запуск тестов

```bash
# Все тесты
cargo test --all-features

# Тесты с фильтрацией
cargo test --lib -- test_name

# Без PostgreSQL
cargo test
```

## CI-тестирование

Каждый Pull Request запускает:
- Проверку форматирования (`cargo fmt --check`)
- Сборку (`cargo build`)
- Clippy (`cargo clippy -- -D warnings`)
- 6550+ юнит-тестов
- Аудит безопасности (`cargo audit`)

См. [Build Pipeline](../../.github/workflows/ci-cd.yml)

## Написание тестов

Тесты находятся рядом с тестируемым кодом в модулях `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function();
        assert!(result.is_ok());
    }
}
```

---

## Следующие шаги

- [Настройка окружения](./dev-setup.md) — локальное окружение
- [Структура кода](./code-structure.md) — организация модулей
- [Участие в проекте](../contributing/contributing.md) — процесс Pull Request
