# WASM Плагины в Velum

> **Динамическая загрузка WASM плагинов для расширения функциональности Semaphore**

## 📖 Оглавление

- [Обзор](#обзор)
- [Архитектура](#архитектура)
- [Быстрый старт](#быстрый-старт)
- [Создание WASM плагина](#создание-wasm-плагина)
- [WASM API](#wasm-api)
- [Безопасность](#безопасность)
- [Конфигурация](#конфигурация)
- [Примеры](#примеры)
- [Отладка](#отладка)

---

## 📋 Обзор

Система WASM плагинов позволяет динамически загружать и выполнять код плагинов без перекомпиляции основного приложения Velum.

**Преимущества:**

| Преимущество | Описание |
|--------------|----------|
| **Безопасность** | Песочница WASM изолирует код плагина от основной системы |
| **Переносимость** | WASM работает на любой платформе где есть WASM runtime |
| **Производительность** | Near-native производительность выполнения кода |
| **Гибкость** | Плагины можно писать на любом языке с WASM поддержкой (Rust, C/C++, Go, AssemblyScript) |
| **Динамичность** | Загрузка/выгрузка плагинов без перезапуска сервера |

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Velum                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Plugin Manager                       │  │
│  │  ┌─────────────────┐  ┌─────────────────────────┐ │  │
│  │  │  WasmLoader     │  │  Native Plugin Loader   │ │  │
│  │  │  • Загрузка     │  │  • Rust плагины         │ │  │
│  │  │  • Валидация    │  │  • Динамические библиотеки│ │  │
│  │  │  • Верификация  │  │                         │ │  │
│  │  └────────┬────────┘  └─────────────────────────┘ │  │
│  │           │                                       │  │
│  │  ┌────────▼────────┐                              │  │
│  │  │  WasmRuntime    │                              │  │
│  │  │  • Wasmtime     │                              │  │
│  │  │  • WASI         │                              │  │
│  │  │  • Sandboxing   │                              │  │
│  │  └────────┬────────┘                              │  │
│  │           │                                       │  │
│  │  ┌────────▼────────┐  ┌─────────────────────────┐ │  │
│  │  │  Plugin A       │  │  Plugin B               │ │  │
│  │  │  (hook_logger)  │  │  (task_executor)        │ │  │
│  │  │  .wasm          │  │  .wasm                  │ │  │
│  │  └─────────────────┘  └─────────────────────────┘ │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Компоненты

1. **WasmLoader** - Загрузка и валидация WASM файлов
2. **WasmRuntime** - Среда выполнения на базе Wasmtime
3. **Host Functions** - Функции предоставляемые хостом (Semaphore) плагинам
4. **Plugin Instances** - Экземпляры загруженных плагинов

---

## 🚀 Быстрый старт

### 1. Установка зависимостей

```bash
# Для сборки WASM плагинов на Rust
rustup target add wasm32-wasi
```

### 2. Создание плагина

```bash
cd rust/src/plugins/wasm_examples/hook_logger
cargo build --target wasm32-wasi --release
```

### 3. Копирование плагина

```bash
cp target/wasm32-wasi/release/hook_logger.wasm /path/to/semaphore/plugins/
```

### 4. Запуск Semaphore

```bash
cd rust
cargo run -- server
```

### 5. Проверка загрузки

```
INFO [semaphore::plugins::wasm_loader] Loaded WASM plugin: hook_logger
```

---

## 🛠️ Создание WASM плагина

### Минимальный плагин

**Cargo.toml:**
```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasi = "0.11"
```

**src/lib.rs:**
```rust
#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    // Инициализация
    0 // 0 = успех
}

#[no_mangle]
pub extern "C" fn handle_hook(event_ptr: i32, event_len: i32) -> i32 {
    // Обработка хука
    0 // 0 = успех
}
```

### Структура проекта

```
my_plugin/
├── Cargo.toml
├── src/
│   └── lib.rs
├── plugin.json          # Метаданные (опционально)
└── build.sh            # Скрипт сборки
```

### plugin.json

```json
{
  "id": "my_plugin",
  "name": "My Custom Plugin",
  "version": "1.0.0",
  "description": "Описание плагина",
  "author": "Your Name",
  "type": "hook",
  "min_semaphore_version": "0.4.0",
  "hooks": [
    "task.after_create",
    "task.after_complete",
    "task.after_fail"
  ]
}
```

---

## 🔌 WASM API

### Экспортируемые функции (Plugin → Host)

| Функция | Сигнатура | Обязательная | Описание |
|---------|-----------|--------------|----------|
| `plugin_init` | `fn() -> i32` | ✅ | Инициализация плагина |
| `handle_hook` | `fn(event_ptr: i32, event_len: i32) -> i32` | ✅ | Обработка хуков |
| `can_handle_hook` | `fn(name_ptr: i32, name_len: i32) -> i32` | ❌ | Проверка поддержки хука |
| `get_plugin_info` | `fn() -> i32` | ❌ | Информация о плагине |
| `can_execute` | `fn(task_ptr: i32, task_len: i32) -> i32` | ❌ | Проверка задачи (Task Executor) |
| `execute` | `fn(task_ptr: i32, task_len: i32) -> i32` | ❌ | Выполнение задачи |
| `send_notification` | `fn(notif_ptr: i32, notif_len: i32) -> i32` | ❌ | Отправка уведомления |

### Импортируемые функции (Host → Plugin)

#### semaphore:log

Логирование сообщений из плагина.

```rust
#[link(wasm_import_module = "semaphore")]
extern "C" {
    fn log(level: i32, message_ptr: i32, message_len: i32);
}

fn log_info(msg: &str) {
    unsafe {
        log(2, msg.as_ptr() as i32, msg.len() as i32);
    }
}
```

**Уровни логирования:**

| Level | Значение |
|-------|----------|
| Error | 0 |
| Warn | 1 |
| Info | 2 |
| Debug | 3 |
| Trace | 4 |

#### semaphore:get_config

Получение значения конфигурации.

```rust
#[link(wasm_import_module = "semaphore")]
extern "C" {
    fn get_config(key_ptr: i32, key_len: i32) -> i64;
}
```

#### semaphore:set_config

Установка значения конфигурации.

```rust
#[link(wasm_import_module = "semaphore")]
extern "C" {
    fn set_config(key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32) -> i32;
}
```

#### semaphore:call_hook

Вызов другого хука из плагина.

```rust
#[link(wasm_import_module = "semaphore")]
extern "C" {
    fn call_hook(hook_ptr: i32, hook_len: i32, data_ptr: i32, data_len: i32) -> i64;
}
```

---

## 🔒 Безопасность

### Sandboxing

WASM плагины выполняются в изолированной среде с ограничениями:

```rust
pub struct WasmSandbox {
    max_memory: 64 * 1024 * 1024,    // 64 MB
    max_fuel: 1_000_000,             // Ограничение выполнения
    allowed_syscalls: vec![],        // Нет прямых syscall
}
```

### Ограничения

| Ресурс | Лимит | Описание |
|--------|-------|----------|
| **Память** | 64 MB | Максимальный объем памяти |
| **Fuel** | 1,000,000 | Ограничение времени выполнения |
| **Сеть** | Запрещено | Нет доступа к сети по умолчанию |
| **Файлы** | plugins_dir | Только чтение директории плагинов |
| **Env** | Запрещено | Нет доступа к переменным окружения |

### Конфигурация безопасности

```rust
WasmLoaderConfig {
    plugins_dir: "./plugins".into(),
    max_memory_pages: 1024,          // 64 MB
    max_execution_time_secs: 30,
    allow_network: false,
    allow_filesystem: true,
    allow_env: false,
}
```

### Валидация плагинов

1. **Хэш-проверка** - Проверка целостности файла
2. **WASM валидация** - Проверка формата WASM
3. **Импорты** - Проверка разрешённых импортов
4. **Экспорты** - Проверка наличия обязательных экспортов

---

## ⚙️ Конфигурация

### Переменные окружения

```bash
# Директория с плагинами
SEMAPHORE_PLUGINS_DIR="./plugins"

# Автозагрузка плагинов
SEMAPHORE_PLUGINS_AUTO_LOAD=true

# Разрешённые плагины
SEMAPHORE_PLUGINS_ENABLED="hook_logger,task_executor"

# Отключённые плагины
SEMAPHORE_PLUGINS_DISABLED="debug_plugin"
```

### Конфигурационный файл

```yaml
plugins:
  wasm:
    enabled: true
    directory: /opt/semaphore/plugins
    auto_load: true
    security:
      max_memory_mb: 64
      max_execution_secs: 30
      allow_network: false
      allow_filesystem: true
```

---

## 📚 Примеры

### Типы плагинов

#### 1. Hook Plugin

Логирование событий:

```rust
#[no_mangle]
pub extern "C" fn task_after_complete(data_ptr: i32, data_len: i32) -> i32 {
    log_info("Task completed!");
    0
}
```

#### 2. Task Executor

Выполнение кастомных задач:

```rust
#[no_mangle]
pub extern "C" fn can_execute(task_ptr: i32, task_len: i32) -> i32 {
    // Проверить тип задачи
    1 // Могу выполнить
}

#[no_mangle]
pub extern "C" fn execute(task_ptr: i32, task_len: i32) -> i32 {
    // Выполнить задачу
    0 // Успех
}
```

#### 3. Notification Plugin

Отправка уведомлений:

```rust
#[no_mangle]
pub extern "C" fn send_notification(notif_ptr: i32, notif_len: i32) -> i32 {
    // Отправить уведомление во внешний сервис
    0 // Успех
}
```

### Готовые примеры

В каталоге `rust/src/plugins/wasm_examples/`:

- `hook_logger/` - Плагин логирования хуков
- `task_executor/` - Пример исполнителя задач
- `notification/` - Пример уведомлений

---

## 🐛 Отладка

### Логирование

Включите подробное логирование:

```bash
RUST_LOG=semaphore::plugins::wasm=debug cargo run
```

### WASM отладка

Используйте `wasm-objdump` для анализа:

```bash
# Просмотр экспортов
wasm-objdump -x hook_logger.wasm | grep Export

# Просмотр импортов
wasm-objdump -x hook_logger.wasm | grep Import
```

### Тестирование

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_plugin_init() {
        assert_eq!(plugin_init(), 0);
    }
    
    #[test]
    fn test_can_handle_hook() {
        assert_eq!(can_handle_hook(0, 0), 1);
    }
}
```

### Частые проблемы

| Проблема | Решение |
|----------|---------|
| Plugin not loading | Проверьте путь к plugins_dir |
| Import error | Убедитесь что импорты совпадают |
| Memory limit | Увеличьте max_memory_pages |
| Timeout | Увеличьте max_execution_time_secs |

---

## 📊 Метрики

### Мониторинг плагинов

```rust
// Prometheus метрики
semaphore_plugins_loaded_total        // Всего загружено
semaphore_plugins_errors_total        // Ошибки плагинов
semaphore_plugin_hook_calls_total     // Вызовы хуков
semaphore_plugin_execution_duration   // Время выполнения
```

---

## 🔗 Ссылки

- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [WASI Specification](https://github.com/WebAssembly/WASI)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [WebAssembly.org](https://webassembly.org/)

---

*Последнее обновление: 10 марта 2026 г.*
