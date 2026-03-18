# Тестирование в Velum

> **Руководство по тестированию и обеспечению качества кода**

## 📖 Оглавление

- [Обзор](#обзор)
- [Запуск тестов](#запуск-тестов)
- [Покрытие кода](#покрытие-кода)
- [Типы тестов](#типы-тестов)
- [Написание тестов](#написание-тестов)
- [CI/CD интеграция](#ci/cd-интеграция)
- [Best Practices](#best-practices)

---

## 📋 Обзор

Velum использует комплексный подход к тестированию:

- **Unit тесты** - тестирование отдельных функций и модулей
- **Integration тесты** - тестирование взаимодействия компонентов
- **API тесты** - тестирование HTTP endpoints
- **Mock объекты** - изоляция зависимостей

**Статистика тестов:**

| Метрика | Значение |
|---------|----------|
| **Всего тестов** | 560+ |
| **Покрытие кода** | ~20.20% |
| **Цель покрытия** | >80% |
| **Время прогона** | ~15 сек |
| **Последнее обновление** | 10 марта 2026 |

**История изменений:**

| Дата | Тестов | Покрытие | Изменения |
|------|--------|----------|-----------|
| 10 мар 2026 | 560 | 20.20% | +15 тестов, исправление 50 failing тестов |
| 7 мар 2026 | 545 | 19.87% | Базовое покрытие |

---

## 🚀 Запуск тестов

### Все тесты

```bash
cd rust
cargo test
```

### Unit тесты

```bash
cargo test --lib
```

### Тесты конкретного модуля

```bash
# Тесты cache модуля
cargo test cache::

# Тесты grpc модуля
cargo test grpc::

# Тесты API handlers
cargo test api::handlers::tests::
```

### Тесты с фильтрацией

```bash
# По имени теста
cargo test test_cache_key

# По тегу (игнорируемые тесты)
cargo test -- --ignored

# Только быстрые тесты
cargo test -- --test-threads=4
```

### Тесты с выводом логов

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Тесты с таймаутом

```bash
# Таймаут 60 секунд на тест
cargo test -- --test-timeout=60000
```

---

## 📊 Покрытие кода

### Установка cargo-tarpaulin

```bash
cargo install cargo-tarpaulin
```

### Запуск с покрытием

```bash
# HTML отчёт
cargo tarpaulin --out Html

# XML отчёт (для CI)
cargo tarpaulin --out Xml

# Консольный отчёт
cargo tarpaulin --out Stdout

# Все форматы
cargo tarpaulin --out Html,Xml,Stdout
```

### Настройка tarpaulin

Создайте `tarpaulin.toml`:

```toml
[default-config]
engine = "Llvm"
coverage-dir = "coverage"
fail-under = 80.0
exclude-files = [
    "src/main.rs",
    "src/ffi/*",
    "src/grpc/semaphore.rs"
]
exclude-tests = true
```

### Интерпретация результатов

```
Coverage Results:
|| Uncovered Lines ||
|| src/cache.rs: 85.71%
|| src/services/cache_service.rs: 72.34%
|| src/api/handlers/projects.rs: 45.67%

|| Covered Lines ||
|| src/models/*.rs: 95.00%
|| src/db/store.rs: 88.50%

Total Coverage: 65.43% (3456/5283)
```

### Цели покрытия

| Модуль | Цель | Текущее | Статус |
|--------|------|---------|--------|
| **webhook** | >80% | 31.85% | 🔴 |
| **cache_service** | >90% | 19.5% | 🔴 |
| **services** | >80% | ~15% | 🔴 |
| **models** | >90% | ~20% | 🔴 |
| **db/sql** | >85% | ~75% | 🟡 |
| **api/handlers** | >75% | ~60% | 🟡 |
| **Общее** | >80% | 20.20% | 🔴 |

**Приоритетные модули для добавления тестов:**

1. `models/access_key.rs` - 0% (79 строк)
2. `models/audit_log.rs` - 0% (84 строки)
3. `plugins/base.rs` - 0% (99 строк)
4. `plugins/hooks.rs` - 0% (100 строк)
5. `services/executor.rs` - 7% (276 строк)
6. `services/alert.rs` - 9% (116 строк)
7. `services/backup.rs` - 8% (153 строки)

---

## 📝 Типы тестов

### Unit тесты

Тестирование отдельных функций:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        assert_eq!(cache_key(&["user", "123"]), "user:123");
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        stats.hits = 80;
        stats.misses = 20;
        
        assert_eq!(stats.hit_ratio(), 80.0);
    }
}
```

### Async тесты

```rust
#[tokio::test]
async fn test_redis_cache_get_disabled() {
    let config = RedisConfig::default();
    let cache = RedisCache::new(config);
    
    let result: Result<Option<String>> = cache.get("test_key").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
```

### Integration тесты

```rust
#[cfg(test)]
mod integration_tests {
    use crate::api::create_app;
    use crate::db::mock::MockStore;
    use axum::{body::Body, http::{Request, StatusCode}};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_projects_list_empty() {
        let app = create_test_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/projects")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### Mock тесты

```rust
use crate::db::mock::MockStore;

#[tokio::test]
async fn test_with_mock_store() {
    let store = MockStore::new();
    let users = store.get_users(RetrieveQueryParams::default()).await;
    
    assert!(users.is_ok());
    assert!(users.unwrap().is_empty());
}
```

---

## ✍️ Написание тестов

### Структура теста

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_name() {
        // Arrange - подготовка данных
        let input = "test";
        
        // Act - выполнение действия
        let result = process(input);
        
        // Assert - проверка результата
        assert_eq!(result, "processed_test");
    }
}
```

### Тестирование ошибок

```rust
#[test]
fn test_error_case() {
    let result = fallible_function(invalid_input);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Invalid input");
}
```

### Параметризованные тесты

```rust
#[test]
fn test_multiple_cases() {
    let test_cases = vec![
        ("input1", "output1"),
        ("input2", "output2"),
        ("input3", "output3"),
    ];
    
    for (input, expected) in test_cases {
        assert_eq!(process(input), expected);
    }
}
```

### Игнорирование тестов

```rust
#[test]
#[ignore = "Requires external service"]
fn test_slow_integration() {
    // Длительный интеграционный тест
}
```

Запуск игнорируемых тестов:
```bash
cargo test -- --ignored
```

---

## 🔧 CI/CD интеграция

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-action@stable
    
    - name: Install cargo-tarpaulin
      run: cargo install cargo-tarpaulin
    
    - name: Run tests
      run: cargo test --verbose
    
    - name: Generate coverage
      run: cargo tarpaulin --out Xml
    
    - name: Upload coverage
      uses: codecov/codecov-action@v4
      with:
        files: ./cobertura.xml
```

### Проверка покрытия в CI

```yaml
- name: Check coverage threshold
  run: |
    COVERAGE=$(cargo tarpaulin --out Stdout | grep "Total Coverage" | awk '{print $4}' | tr -d '%')
    if (( $(echo "$COVERAGE < 80" | bc -l) )); then
      echo "Coverage $COVERAGE% is below threshold 80%"
      exit 1
    fi
```

---

## 📚 Best Practices

### 1. Naming conventions

```rust
// Хорошо
#[test]
fn test_cache_key_generation() { }

#[test]
fn test_redis_cache_get_returns_none_when_disabled() { }

// Плохо
#[test]
fn test1() { }

#[test]
fn cache() { }
```

### 2. Test isolation

```rust
// Хорошо - каждый тест независим
#[tokio::test]
async fn test_cache_set() {
    let cache = create_test_cache();
    // Тест использует только свои данные
}

// Плохо - тесты зависят друг от друга
#[tokio::test]
async fn test_cache_set() {
    // Использует данные из предыдущего теста
}
```

### 3. Fast tests

```rust
// Хорошо - быстрый unit тест
#[test]
fn test_cache_key() {
    assert_eq!(cache_key(&["a"]), "a");
}

// Плохо - медленный integration тест без необходимости
#[tokio::test]
async fn test_cache_key_with_redis() {
    // Подключается к Redis для простой проверки
}
```

### 4. Descriptive assertions

```rust
// Хорошо
assert_eq!(
    stats.hit_ratio(),
    80.0,
    "Hit ratio should be 80% with 80 hits and 20 misses"
);

// Плохо
assert!(stats.hit_ratio() == 80.0);
```

### 5. Test edge cases

```rust
#[test]
fn test_cache_stats_edge_cases() {
    // Zero requests
    let stats = CacheStats::default();
    assert_eq!(stats.hit_ratio(), 0.0);
    
    // All hits
    let mut stats = CacheStats::default();
    stats.hits = 100;
    assert_eq!(stats.hit_ratio(), 100.0);
    
    // All misses
    let mut stats = CacheStats::default();
    stats.misses = 100;
    assert_eq!(stats.hit_ratio(), 0.0);
}
```

---

## 🐛 Отладка тестов

### Вывод отладочной информации

```rust
#[tokio::test]
async fn test_debug() {
    let cache = RedisCache::new(RedisConfig::default());
    
    eprintln!("Cache created: {:?}", cache.is_enabled());
    
    let result = cache.get::<String>("key").await;
    dbg!(&result);
    
    assert!(result.is_ok());
}
```

Запуск:
```bash
cargo test test_debug -- --nocapture
```

### Поиск медленных тестов

```bash
cargo test -- --report-time
```

### Профилирование тестов

```bash
cargo test --profile=release
```

---

## 🔗 Ссылки

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [tokio-testing](https://tokio.rs/tokio/tutorial/testing)
- [mockall](https://docs.rs/mockall/latest/mockall/)

---

*Последнее обновление: 10 марта 2026 г.*
