# Руководство для участников

Спасибо за интерес к проекту Velum (Rust)! Это руководство поможет вам начать вносить вклад в проект.

## 📚 Содержание

- [С чего начать](#с-чего-начать)
- [Архитектура проекта](#архитектура-проекта)
- [Стиль кода](#стиль-кода)
- [Тестирование](#тестирование)
- [Pull Request](#pull-request)
- [Сообщество](#сообщество)

## 🚀 С чего начать

### 1. Форк и клонирование

```bash
# Форкните репозиторий на GitHub
# Затем склонируйте:
git clone https://github.com/YOUR_USERNAME/semaphore.git
cd semaphore/rust
```

### 2. Установка зависимостей

```bash
# Установите Rust (если ещё не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Установите task (опционально)
go install github.com/go-task/task/v3/cmd/task@latest
```

### 3. Первая сборка

```bash
# Загрузка зависимостей
cargo fetch

# Сборка
cargo build

# Запуск тестов
cargo test
```

## 🏗 Архитектура проекта

```
src/
├── api/           # HTTP API (Axum)
├── cli/           # CLI (Clap)
├── config/        # Конфигурация
├── db/            # Слой доступа к данным
├── models/        # Модели данных
├── services/      # Бизнес-логика
├── error.rs       # Ошибки
└── logging.rs     # Логирование
```

### Основные модули

- **api** — обработчики HTTP-запросов, маршруты, middleware
- **db** — трейты хранилищ и реализации (SQL, BoltDB)
- **models** — структуры данных (User, Project, Task, etc.)
- **services** — бизнес-логика (выполнение задач, планирование)

## 📝 Стиль кода

### Общие правила

1. **Русские комментарии** — все комментарии и документация на русском языке
2. **Идентификаторы на английском** — имена переменных, функций, типов на английском
3. **Форматирование** — используйте `cargo fmt`
4. **Линтинг** — проверяйте код через `cargo clippy`

### Пример

```rust
/// Проверяет валидность пользователя
///
/// Возвращает ошибку, если имя пользователя, email или имя пустые.
pub fn validate(&self) -> Result<(), ValidationError> {
    if self.username.is_empty() {
        return Err(ValidationError::UsernameEmpty);
    }
    // ... остальные проверки
}
```

### Именование

- **Типы**: `PascalCase` — `UserProfile`, `TaskStatus`
- **Функции**: `snake_case` — `create_user`, `get_task`
- **Константы**: `UPPER_SNAKE_CASE` — `MAX_PARALLEL_TASKS`
- **Модули**: `snake_case` — `task_logger`, `access_key`

## 🧪 Тестирование

### Запуск тестов

```bash
# Все тесты
cargo test

# Тесты с выводом
cargo test -- --nocapture

# Тесты конкретного модуля
cargo test --package semaphore --module cli

# Покрытие тестами
cargo tarpaulin --out Html
```

### Написание тестов

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_validation() {
        let user = User {
            username: "".to_string(),
            // ...
        };
        
        assert!(user.validate().is_err());
    }
}
```

## 📤 Pull Request

### Перед отправкой

1. ✅ Убедитесь, что все тесты проходят
2. ✅ Запустите `cargo fmt` и `cargo clippy`
3. ✅ Обновите документацию (если нужно)
4. ✅ Добавьте тесты для новых функций

### Чеклист PR

```markdown
## Чеклист

- [ ] Код отформатирован (`cargo fmt`)
- [ ] Нет предупреждений (`cargo clippy`)
- [ ] Все тесты проходят
- [ ] Документация обновлена
- [ ] Добавлены тесты (если применимо)
- [ ] Changelog обновлён (если применимо)
```

### Название PR

Используйте понятные названия:

- ✅ `feat: Добавить поддержку WebSocket`
- ✅ `fix: Исправить утечку памяти в обработчике задач`
- ✅ `docs: Обновить README.md`
- ❌ `update`, `fix bug`, `changes`

## 🐛 Сообщение об ошибках

### Хороший баг-репорт

```markdown
**Описание**
Краткое описание проблемы.

**Шаги воспроизведения**
1. Запустить команду '...'
2. Нажать кнопку '...'
3. Увидеть ошибку

**Ожидаемое поведение**
Что должно было произойти.

**Фактическое поведение**
Что произошло вместо этого.

**Окружение**
- OS: Ubuntu 22.04
- Rust: 1.75
- База данных: SQLite

**Логи**
```
[текст логов]
```
```

## 💡 Предложения функций

### Формат предложения

```markdown
**Проблема**
Какую проблему решает эта функция?

**Решение**
Как должно работать решение?

**Альтернативы**
Какие альтернативы рассматривались?

**Дополнительно**
Скриншоты, мокапы, примеры кода.
```

## 🤝 Сообщество

- **Discord**: [https://discord.gg/5R6k7hNGcH](https://discord.gg/5R6k7hNGcH)
- **GitHub Issues**: [https://github.com/velum/velum/issues](https://github.com/velum/velum/issues)
- **YouTube**: [https://www.youtube.com/@semaphoreui](https://www.youtube.com/@semaphoreui)

## 📖 Ресурсы

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Clap Documentation](https://docs.rs/clap/)

## 🎯 Области для вклада

### Начинающим

- 📝 Документация
- 🧪 Тесты
- 🐛 Простые баг-фиксы
- 🌐 Переводы

### Опытным

- 🏗 Архитектурные улучшения
- ⚡ Оптимизация производительности
- 🔐 Безопасность
- 🚀 Новые функции

---

**Спасибо за ваш вклад!** 🎉
