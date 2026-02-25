# План удаления дублирующих Go модулей

## 📋 Обзор

Этот документ описывает план удаления Go модулей, которые были переписаны на Rust.

**Дата создания**: 2025-02-25

---

## 🎯 Критерии удаления

Go модуль может быть удалён, если:

1. ✅ Функциональность полностью переписана на Rust
2. ✅ Rust-код протестирован и работает корректно
3. ✅ Нет зависимостей от этого модуля в оставшемся Go-коде
4. ✅ Создана документация на Rust-реализацию

---

## 📊 Статус модулей

### 1. pkg/task_logger

**Статус**: ⚠️ ЧАСТИЧНО ГОТОВ К УДАЛЕНИЮ

**Go реализация**:
- Файл: `pkg/task_logger/task_logger.go`
- Функциональность:
  - Статусы задач (TaskStatus)
  - Интерфейсы логгера (Logger, StatusListener, LogListener)
  - Форматирование статусов с эмодзи
  - Валидация статусов

**Rust реализация**:
- Файл: `rust/src/services/task_logger.rs`
- Реализовано:
  - ✅ Enum `TaskStatus` (базовые статусы)
  - ✅ Trait `FromStr` для парсинга
  - ✅ Trait `Display` для форматирования
- Не реализовано:
  - ❌ Интерфейсы `Logger`, `StatusListener`, `LogListener`
  - ❌ Методы `IsValid()`, `IsNotifiable()`, `IsFinished()`
  - ❌ Форматирование с эмодзи
  - ❌ Метод `UnfinishedTaskStatuses()`

**Зависимости в Go**:
- `pkg/ssh/agent.go` - использует `task_logger.Logger`
- `db_lib/*.go` - используют `task_logger.Logger`

**Рекомендация**: 
- **НЕ УДАЛЯТЬ** до полной реализации интерфейсов в Rust
- Требуется дополнить Rust-версию недостающими методами

---

### 2. pkg/conv

**Статус**: ⚠️ ЧАСТИЧНО ГОТОВ К УДАЛЕНИЮ

**Go реализация**:
- Файл: `pkg/conv/conv.go`
- Функциональность:
  - `ConvertFloatToIntIfPossible()` - конвертация float в int
  - `StructToFlatMap()` - преобразование структуры в плоскую map

**Rust реализация**:
- Отсутствует

**Зависимости в Go**:
- Требуется анализ через `grep -r "pkg/conv"`

**Рекомендация**:
- Создать Rust-модуль `rust/src/utils/conv.rs`
- Реализовать функции конвертации
- Перенести `StructToFlatMap` в сериализацию serde

---

### 3. pkg/random

**Статус**: ✅ ГОТОВ К УДАЛЕНИЮ

**Go реализация**:
- Файл: `pkg/random/string.go`
- Функциональность:
  - `Number(strlen)` - случайные цифры
  - `String(strlen)` - случайные буквы+цифры
  - Криптографически стойкий генератор

**Rust реализация**:
- Файл: `rust/Cargo.toml` - зависимость `rand = "0.9"`
- Реализовано:
  - ✅ `rand::Rng` для генерации случайных чисел
  - ✅ `rand::distributions::Alphanumeric` для строк

**Зависимости в Go**:
- `pkg/ssh/agent.go` - использует `random.String(10)`

**Рекомендация**:
- **МОЖНО УДАЛЯТЬ** после обновления зависимостей в Go
- В Rust использовать crate `rand`

---

### 4. pkg/common_errors

**Статус**: ⚠️ ТРЕБУЕТ АНАЛИЗА

**Go реализация**:
- Файл: `pkg/common_errors/common_errors.go`
- Функциональность:
  - `UserVisibleError` - ошибки для пользователя
  - `NewUserError()`, `NewUserErrorS()` - конструкторы
  - `ErrInvalidSubscription` - ошибка подписки
  - `GetErrorContext()` - контекст ошибки

**Rust реализация**:
- Файл: `rust/src/error.rs`
- Реализовано:
  - ✅ Enum `Error` с вариантами
  - ✅ Trait `std::error::Error`
  - ✅ Интеграция с `thiserror`

**Зависимости в Go**:
- Требуется анализ

**Рекомендация**:
- Проверить использование в Go-коде
- Убедиться, что Rust `error.rs` покрывает все кейсы

---

### 5. pkg/ssh

**Статус**: ❌ НЕ ГОТОВ К УДАЛЕНИЮ

**Go реализация**:
- Файлы: `pkg/ssh/agent.go`, `pkg/ssh/agent_test.go`
- Функциональность:
  - `Agent` - SSH агент
  - `StartSSHAgent()` - запуск агента
  - `AccessKeyInstallation` - установка ключей
  - `KeyInstaller` - установщик ключей
  - Интеграция с `golang.org/x/crypto/ssh`

**Rust реализация**:
- Отсутствует

**Зависимости в Go**:
- `db_lib/AccessKeyInstaller.go`
- `db_lib/AnsibleApp.go`
- `db_lib/GoGitClient.go`
- И другие

**Рекомендация**:
- **НЕ УДАЛЯТЬ** - критичный модуль
- Требуется полная переписывание на Rust с использованием:
  - `russh` или `ssh2` crate для SSH
  - Интеграция с tokio для асинхронности

---

### 6. pkg/tz

**Статус**: ✅ ГОТОВ К УДАЛЕНИЮ

**Go реализация**:
- Файл: `pkg/tz/time.go`
- Функциональность:
  - `Now()` - текущее время UTC
  - `In(t)` - конвертация в UTC

**Rust реализация**:
- Файл: `rust/Cargo.toml` - зависимость `chrono = "0.4"`
- Реализовано:
  - ✅ `chrono::Utc::now()` - аналог `Now()`
  - ✅ `DateTime::to_utc()` - аналог `In()`

**Зависимости в Go**:
- Требуется анализ

**Рекомендация**:
- **МОЖНО УДАЛЯТЬ**
- В Rust использовать `chrono::Utc`

---

## 📅 План удаления

### Этап 1: Подготовка (1-2 недели)

1. **Завершить task_logger**
   - Добавить интерфейсы `Logger`, `StatusListener`, `LogListener`
   - Реализовать форматирование с эмодзи
   - Добавить методы валидации

2. **Создать utils/conv**
   - Реализовать функции конвертации
   - Интегрировать с serde

3. **Проанализировать зависимости**
   - Запустить `grep -r "pkg/..."` для каждого модуля
   - Составить список файлов, использующих модули

### Этап 2: Удаление простых модулей (1 неделя)

Порядок удаления:

1. ✅ **pkg/tz** -最简单, заменить на `time.Now().UTC()`
2. ✅ **pkg/random** - заменить на `rand` crate
3. ⚠️ **pkg/conv** - после создания Rust-аналога
4. ⚠️ **pkg/common_errors** - после анализа зависимостей

### Этап 3: Удаление сложных модулей (2-4 недели)

1. ❌ **pkg/task_logger** - после завершения Rust-реализации
2. ❌ **pkg/ssh** - **последним**, после полной реализации на Rust

---

## 🔧 Скрипты для анализа

### Поиск зависимостей

```bash
# Найти все использования pkg/task_logger
grep -r "pkg/task_logger" --include="*.go" .

# Найти все использования pkg/random
grep -r "pkg/random" --include="*.go" .

# Найти все использования pkg/ssh
grep -r "pkg/ssh" --include="*.go" .

# Найти все использования pkg/conv
grep -r "pkg/conv" --include="*.go" .

# Найти все использования pkg/common_errors
grep -r "common_errors" --include="*.go" .

# Найти все использования pkg/tz
grep -r "pkg/tz" --include="*.go" .
```

### Проверка компиляции после удаления

```bash
# Проверка Go кода
go build ./...

# Проверка Rust кода
cd rust && cargo check
```

---

## ⚠️ Предупреждения

1. **Не удаляйте модули по одному** - удаляйте только после полной готовности Rust-аналога
2. **Сохраняйте обратную совместимость** - пока Go-код используется
3. **Тестируйте после каждого удаления** - запускайте полный набор тестов
4. **Обновляйте документацию** - фиксируйте изменения в CHANGELOG.md

---

## 🔧 Детальный план удаления по модулям

### 1. pkg/tz (Приоритет 1)

**Файлы для удаления**:
- `pkg/tz/time.go`
- `pkg/tz/` (директория)

**Зависимости** (20+ файлов):
```
./api/login.go
./api/sockets/handler.go
./api/router.go
./services/runners/running_job.go
./services/tasks/TaskRunner.go
./services/tasks/RemoteJob.go
./services/tasks/TaskPool.go
./services/tasks/TaskRunner_logging.go
./db/User.go
./db/Task.go
./db/sql/migration.go
./db/sql/project.go
./db/sql/user.go
./db/sql/global_runner.go
./db/sql/event.go
./db/sql/session.go
./db/bolt/project.go
./db/bolt/user.go
./db/bolt/global_runner.go
./db/bolt/event.go
./db/bolt/session.go
./db/bolt/migration_2_14_7.go
```

**Замена в Go** (пока Rust не готов):
```go
// Было:
import "github.com/semaphoreui/semaphore/pkg/tz"
now := tz.Now()

// Стало:
import "time"
now := time.Now().UTC()
```

**Rust эквивалент**:
```rust
use chrono::Utc;
let now = Utc::now();
```

**План**:
1. [ ] Заменить все импорты на `time.Now().UTC()`
2. [ ] Протестировать компиляцию
3. [ ] Удалить директорию `pkg/tz`
4. [ ] Обновить `.gitignore` если нужно

---

### 2. pkg/random (Приоритет 2)

**Файлы для удаления**:
- `pkg/random/string.go`
- `pkg/random/` (директория)

**Зависимости** (9 файлов):
```
./api/login.go
./api/projects/integration_alias.go
./api/projects/environment.go
./services/server/secret_storage_svc.go
./services/project/restore.go
./services/project/backup.go
./services/tasks/TaskPool.go
./pkg/ssh/agent.go  ⚠️ ВАЖНО: удалять ПОСЛЕДНИМ
./.dredd/hooks/helpers.go
```

**Замена в Go**:
```go
// Было:
import "github.com/semaphoreui/semaphore/pkg/random"
str := random.String(10)

// Стало (если нужно в Go):
import (
    "crypto/rand"
    "encoding/hex"
)
bytes := make([]byte, 10)
rand.Read(bytes)
str := hex.EncodeToString(bytes)[:10]
```

**Rust эквивалент**:
```rust
use rand::{Rng, distributions::Alphanumeric};
let s: String = rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(10)
    .collect();
```

**План**:
1. [ ] Обновить `pkg/ssh/agent.go` (использовать встроенную генерацию)
2. [ ] Заменить импорты в остальных файлах
3. [ ] Протестировать компиляцию
4. [ ] Удалить директорию `pkg/random`

---

### 3. pkg/conv (Приоритет 3)

**Файлы для удаления**:
- `pkg/conv/conv.go`
- `pkg/conv/` (директория)

**Зависимости** (4 файла):
```
./api/apps_test.go
./api/apps.go
./api/integration.go
./db/bolt/migration_2_14_7.go
```

**Функциональность**:
- `ConvertFloatToIntIfPossible()` - конвертация float в int
- `StructToFlatMap()` - преобразование структуры в map

**Rust реализация** (создать):
```rust
// rust/src/utils/conv.rs
pub fn convert_float_to_int_if_possible(v: &serde_json::Value) -> Option<i64> {
    match v {
        serde_json::Value::Number(n) => n.as_i64(),
        _ => None,
    }
}

// Для StructToFlatMap использовать serde с кастомным сериализатором
```

**План**:
1. [ ] Создать `rust/src/utils/conv.rs`
2. [ ] Реализовать функции
3. [ ] Написать тесты
4. [ ] Удалить `pkg/conv`

---

### 4. pkg/common_errors (Приоритет 4)

**Файлы для удаления**:
- `pkg/common_errors/common_errors.go`
- `pkg/common_errors/` (директория)

**Зависимости** (20+ файлов):
```
./api/helpers/write_response.go
./api/projects/tasks.go
./services/schedules/SchedulePool.go
./services/server/access_key_svc.go
./services/server/access_key_serializer_local.go
... (и другие)
```

**Rust реализация** (уже есть):
- `rust/src/error.rs` - содержит `enum Error`

**Что добавить в Rust**:
```rust
// rust/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid subscription")]
    InvalidSubscription,
    
    #[error("{0}")]
    UserVisible(String),
}

impl Error {
    pub fn new_user_error(msg: impl Into<String>) -> Self {
        Error::UserVisible(msg.into())
    }
    
    pub fn get_error_context() -> String {
        // Реализовать через std::panic::Location
        format!("{}:{}", file!(), line!())
    }
}
```

**План**:
1. [ ] Дополнить `rust/src/error.rs`
2. [ ] Заменить все импорты в Go на стандартные ошибки
3. [ ] Протестировать
4. [ ] Удалить `pkg/common_errors`

---

### 5. pkg/task_logger (Приоритет 5)

**Файлы для удаления**:
- `pkg/task_logger/task_logger.go`
- `pkg/task_logger/` (директория)

**Зависимости** (30+ файлов) - КРИТИЧНЫЙ МОДУЛЬ:
```
./api/runners/runners.go
./api/tasks/tasks.go
./pro_interfaces/log_write_svc.go
./db_lib/GoGitClient.go
./db_lib/GitRepository.go
./db_lib/AppFactory.go
./db_lib/TerraformApp.go
./db_lib/AnsibleApp.go
./db_lib/AnsiblePlaybook.go
./db_lib/AccessKeyInstaller.go
./db_lib/LocalApp.go
./db_lib/ShellApp.go
./services/schedules/SchedulePool_test.go
./services/runners/job_pool.go
./services/runners/types.go
./services/runners/running_job.go
./services/server/access_key_installation_svc.go
./services/tasks/TaskPool_test.go
./services/tasks/TaskRunner.go
./services/tasks/RemoteJob.go
./services/tasks/alert_test_sender.go
./services/tasks/LocalJob.go
./services/tasks/TaskPool.go
./services/tasks/alert.go
./services/tasks/TaskRunner_test.go
./services/tasks/TaskRunner_logging.go
./db/Store.go
./db/Task.go
./db/sql/SqlDb.go
./db/bolt/BoltDb.go
... (и другие)
```

**Rust реализация** (требуется дополнить):
```rust
// rust/src/services/task_logger.rs

// Добавить недостающие методы
pub trait Logger {
    fn log(&mut self, msg: &str);
    fn logf(&mut self, format: &str, args: ...);
    fn set_status(&mut self, status: TaskStatus);
    fn add_status_listener(&mut self, listener: Box<dyn Fn(TaskStatus)>);
    fn add_log_listener(&mut self, listener: Box<dyn Fn(DateTime<Utc>, &str)>);
    fn wait_log(&self);
}

// Добавить методы для TaskStatus
impl TaskStatus {
    pub fn is_valid(&self) -> bool { /* ... */ }
    pub fn is_notifiable(&self) -> bool { /* ... */ }
    pub fn is_finished(&self) -> bool { /* ... */ }
    
    pub fn format(&self) -> String {
        match self {
            TaskStatus::Error => "❌ ERROR".to_string(),
            TaskStatus::Success => "✅ SUCCESS".to_string(),
            TaskStatus::Stopped => "⏹️ STOPPED".to_string(),
            TaskStatus::Waiting => "❓ WAITING".to_string(),
            // ...
        }
    }
}

pub fn unfinished_task_statuses() -> Vec<TaskStatus> {
    vec![
        TaskStatus::Waiting,
        TaskStatus::Running,
        // ...
    ]
}
```

**План**:
1. [ ] Дополнить `TaskStatus` методами
2. [ ] Создать trait `Logger`
3. [ ] Реализовать базовый логгер
4. [ ] Написать тесты
5. [ ] Постепенно заменять использования в Go
6. [ ] Удалить `pkg/task_logger` (ПОСЛЕДНИМ из бизнес-логики)

---

### 6. pkg/ssh (Приоритет 6)

**Файлы для удаления**:
- `pkg/ssh/agent.go`
- `pkg/ssh/agent_test.go`
- `pkg/ssh/` (директория)

**Зависимости** (7 файлов) - ОЧЕНЬ КРИТИЧНЫЙ:
```
./db_lib/AccessKeyInstaller.go
./db_lib/CmdGitClient.go
./services/schedules/SchedulePool_test.go
./services/server/access_key_installation_svc.go
./services/tasks/LocalJob.go
./services/tasks/TaskRunner_test.go
./cli/cmd/runner.go
```

**Функциональность**:
- SSH агент для работы с Git
- Установка SSH ключей
- Интеграция с `golang.org/x/crypto/ssh`

**Rust реализация** (требуется создать с нуля):
```rust
// rust/src/services/ssh_agent.rs

use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;

pub struct SshAgent {
    session: Session,
    socket_file: String,
}

impl SshAgent {
    pub fn new(key_path: &Path, passphrase: Option<&str>) -> Result<Self> {
        // Реализация через ssh2 или russh crate
        unimplemented!()
    }
    
    pub fn start(&mut self) -> Result<()> {
        unimplemented!()
    }
    
    pub fn close(&mut self) -> Result<()> {
        unimplemented!()
    }
}
```

**Необходимые crate**:
- `ssh2` или `russh` - SSH клиент
- `tokio` - асинхронность
- `tempfile` - временные файлы для сокетов

**План**:
1. [ ] Добавить зависимости в `Cargo.toml`
2. [ ] Создать `rust/src/services/ssh_agent.rs`
3. [ ] Реализовать базовый SSH агент
4. [ ] Протестировать с Git
5. [ ] Интегрировать с бизнес-логикой
6. [ ] Удалить `pkg/ssh` (САМЫМ ПОСЛЕДНИМ)

---

| Модуль | Статус | Сложность | Приоритет | Зависимости в Go |
|--------|--------|-----------|-----------|------------------|
| pkg/tz | ✅ Готов | Низкая | **1** | 20+ файлов (db, api, services) |
| pkg/random | ✅ Готов | Низкая | **2** | 9 файлов (включая pkg/ssh) |
| pkg/conv | ⚠️ В работе | Средняя | **3** | 4 файла (api, db/bolt) |
| pkg/common_errors | ⚠️ Анализ | Средняя | **4** | 20+ файлов (api, services) |
| pkg/task_logger | ⚠️ В работе | Высокая | **5** | 30+ файлов (критичный) |
| pkg/ssh | ❌ Не готов | Очень высокая | **6** | 7 файлов (критичный) |

---

## 🎯 Критерии готовности к удалению

Для каждого модуля перед удалением:

- [ ] Rust-реализация полностью функциональна
- [ ] Rust-код протестирован (unit + integration тесты)
- [ ] Все зависимости в Go обновлены или удалены
- [ ] Документация обновлена
- [ ] CHANGELOG.md обновлён
- [ ] Команда уведомлена об изменениях

---

**Последнее обновление**: 2025-02-25

**Ответственный**: Комда разработки Semaphore UI
