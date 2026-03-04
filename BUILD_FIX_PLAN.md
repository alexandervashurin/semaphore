# План исправления ошибок сборки Semaphore Rust

**Дата:** 2026-03-02  
**Последнее обновление:** 2026-03-05 (сессия 12)

---

## 📊 Текущий статус

| Метрика | Значение |
|---------|----------|
| Начальное количество ошибок | 585 |
| Исправлено ошибок (lib) | 585 |
| **Осталось ошибок** | **0** ✅ |
| **Процент выполнения** | **100%** |
| Тесты | 490 passed, 0 failed, 6 ignored |

---

## ✅ Выполнено (сессии 1-6)

### Сессия 6 (28 ошибок)
- [x] Git Client ssh_key ошибки (4)
- [x] local_job/vault.rs ошибки (3)
- [x] local_job/environment.rs ошибки (3)
- [x] local_job/args.rs ошибки (4)
- [x] TemplateType match ошибки (3)
- [x] mismatched types часть 1 (11)

### Сессия 5 (61 ошибка)
- [x] Удаление BoltDB (43 файла)
- [x] Конфигурация (db_dialect, non_admin_can_create_project)
- [x] DbConfig инициализаторы
- [x] Task инициализаторы
- [x] TaskOutput инициализаторы
- [x] Moved value ошибки
- [x] [u8; 16] форматирование
- [x] HAConfig node_id
- [x] Repository.get_full_path()

### Сессия 4 (159 ошибок)
- [x] System Process (libc → nix)
- [x] Default реализации
- [x] ProjectUser модель
- [x] TaskStageType

### Сессии 1-3 (~200 ошибок)
- [x] BoltDB API
- [x] Модели данных
- [x] Конфигурация
- [x] Store Trait
- [x] TaskLogger Clone
- [x] AccessKey методы

---

## 🔴 Оставшиеся ошибки (137)

### Приоритет 1: mismatched types (~11 ошибок)

#### ansible_app.rs
- [ ] Строка 398: callback тип `FnOnce(u32)` vs `Fn(&Child)`

#### ansible_playbook.rs
- [ ] Строки 71, 96: `tokio::process::Command` vs `std::process::Command`

#### go_git_client.rs
- [ ] Строка 45: `opts.clone()` сигнатура
- [ ] Строка 78: `set_head()` сигнатура

#### shell_app.rs
- [ ] Строка 86: callback тип для status listener

---

### Приоритет 2: SQLx трейты (6 ошибок)

#### Task модель
- [ ] Реализовать `sqlx::Decode<'_, DB>` для `Task`
- [ ] Реализовать `sqlx::Type<DB>` для `Task`
- [ ] Файл: `src/models/task.rs`

#### SecretStorage модель
- [ ] Реализовать `FromRow` для `SecretStorage`
- [ ] Файл: `src/models/secret_storage.rs`

---

### Приоритет 3: Clone trait (4 ошибки)

#### LocalAppRunningArgs
- [ ] Убрать `#[derive(Clone)]`
- [ ] Использовать `Arc` для `task_params`, `template_params`
- [ ] Изменить callback тип
- [ ] Файл: `src/db_lib/local_app.rs`

---

### Приоритет 4: Job trait (1 ошибка)

#### LocalJob
- [ ] Реализовать `impl Job for LocalJob`
- [ ] Метод `run()` с 4 параметрами
- [ ] Файлы: `src/services/local_job/types.rs`, `src/services/job.rs`

---

### Приоритет 5: Прочие ошибки (~115 ошибок)

#### type annotations needed (~10)
- [ ] Добавить явные аннотации типов в closure

#### and_then for i32 (3)
- [ ] Исправить использование `and_then` для `i32`

#### Exporter traits (4)
- [ ] Реализовать `DataExporter` для `ExporterChain`
- [ ] Реализовать `TypeExporter` для `ValueMap<T>`

---

## 📅 Дорожная карта

### Сессия 7 (текущая)
**Цель: < 120 ошибок**

- [ ] Исправить mismatched types часть 2 (11 ошибок)
- [ ] Исправить SQLx трейты (6 ошибок)
- [ ] Исправить Clone trait (4 ошибки)

**Ожидаемый результат:** ~116 ошибок

### Сессия 8
**Цель: < 100 ошибок**

- [ ] Исправить Job trait (1 ошибка)
- [ ] Исправить type annotations (10 ошибок)
- [ ] Исправить and_then for i32 (3 ошибки)
- [ ] Исправить Exporter traits (4 ошибки)

**Ожидаемый результат:** ~98 ошибок

### Сессия 9-10
**Цель: < 50 ошибок**

- [ ] Исправить оставшиеся mismatched types
- [ ] Исправить прочие ошибки

**Ожидаемый результат:** ~40-50 ошибок

### Сессия 11-12
**Цель: < 10 ошибок**

- [ ] Финальная полировка
- [ ] Исправление последних ошибок

**Ожидаемый результат:** < 10 ошибок

### Сессия 13
**Цель: 0 ошибок**

- [ ] Первая успешная сборка!
- [ ] Все тесты проходят

---

## 📝 Технические заметки

### Архитектурные решения

1. **SQLx интеграция**
   - Для сложных моделей использовать кастомный `FromRow`
   - Для простых - derive макросы
   - Избегать сложных вложенных структур

2. **Clone для dyn traits**
   - Использовать `Arc` вместо `Clone`
   - Избегать хранения `Box<dyn Trait>` в Clone структурах

3. **Command типы**
   - Использовать `tokio::process::Command` для async кода
   - Использовать `std::process::Command` для sync кода
   - Не смешивать!

### Известные проблемы

1. **callback типы**
   - `FnOnce(u32)` vs `Fn(&Child)` - требуют унификации
   - Решение: использовать один тип во всех местах

2. **git2 API**
   - `RepoBuilder::clone()` требует правильную сигнатуру
   - `Repository::set_head()` требует `&str` не `Oid`

3. **JSON строки в моделях**
   - `vaults`, `secrets` - JSON строки
   - Требуют парсинга перед использованием
   - Решение: создать helper методы для парсинга

---

## 🎯 Метрики качества

| Метрика | Цель | Текущее | Статус |
|---------|------|---------|--------|
| Ошибок компиляции | 0 | 137 | 🔴 |
| Предупреждений | < 100 | ~280 | 🔴 |
| Покрытие тестами | > 50% | ? | ⚪ |
| Успешная сборка | Да | Нет | 🔴 |

---

## 📚 Ресурсы

- [BUILD_ERRORS.md](BUILD_ERRORS.md) - полный отчёт об ошибках
- [SQLx документация](https://docs.rs/sqlx/)
- [git2 документация](https://docs.rs/git2/)
- [tokio документация](https://docs.rs/tokio/)
