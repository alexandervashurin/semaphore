# Пример WASM плагина для Semaphore

Этот каталог содержит примеры WASM плагинов для Velum.

## Пример 1: Hook Plugin (Rust)

Простой плагин который логирует события задач.

### Структура проекта

```
wasm_examples/
└── hook_logger/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

### Cargo.toml

```toml
[package]
name = "hook_logger"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasi = "0.11"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### lib.rs

```rust
use serde::{Deserialize, Serialize};

// Экспортируемые функции для Semaphore

/// Инициализация плагина
#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log_info("Hook Logger plugin initialized");
    0 // Success
}

/// Обработка хука
#[no_mangle]
pub extern "C" fn handle_hook(event_ptr: i32, event_len: i32) -> i32 {
    // Читаем событие из памяти (в реальной реализации)
    log_info("Hook event received");
    0 // Success
}

/// Проверка может ли плагин обработать хук
#[no_mangle]
pub extern "C" fn can_handle_hook(hook_name_ptr: i32, hook_name_len: i32) -> i32 {
    1 // Yes, can handle
}

// Хост-функции (импортируются из Semaphore)

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

### Сборка

```bash
# Установить WASM target
rustup target add wasm32-wasi

# Собрать плагин
cargo build --target wasm32-wasi --release

# Плагин будет в target/wasm32-wasi/release/hook_logger.wasm
```

## Пример 2: Task Executor Plugin

Плагин для выполнения кастомных задач.

### lib.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Task {
    id: i64,
    name: String,
    template_id: i64,
}

#[derive(Debug, Serialize)]
struct TaskResult {
    success: bool,
    output: String,
    exit_code: i32,
    duration_secs: f64,
}

/// Проверка может ли плагин выполнить задачу
#[no_mangle]
pub extern "C" fn can_execute(task_ptr: i32, task_len: i32) -> i32 {
    // В реальной реализации нужно распарсить task и проверить
    1 // Yes
}

/// Выполнение задачи
#[no_mangle]
pub extern "C" fn execute(task_ptr: i32, task_len: i32) -> i32 {
    log_info("Executing task...");
    
    // Симуляция выполнения
    let result = TaskResult {
        success: true,
        output: "Task completed successfully".to_string(),
        exit_code: 0,
        duration_secs: 1.5,
    };
    
    // Сериализуем результат в память
    // (в реальной реализации)
    
    0 // Success
}

/// Остановка задачи
#[no_mangle]
pub extern "C" fn stop(task_id: i64) -> i32 {
    log_info(&format!("Stopping task {}", task_id));
    0
}

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

## Пример 3: Notification Plugin

Плагин для отправки уведомлений в кастомный канал.

### lib.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Notification {
    title: String,
    message: String,
    level: String,
}

/// Отправка уведомления
#[no_mangle]
pub extern "C" fn send_notification(notif_ptr: i32, notif_len: i32) -> i32 {
    log_info("Sending notification...");
    
    // В реальной реализации:
    // 1. Распарсить notification
    // 2. Отправить в кастомный сервис (HTTP запрос и т.д.)
    // 3. Вернуть результат
    
    0 // Success
}

/// Получение доступных каналов
#[no_mangle]
pub extern "C" fn get_channels() -> i32 {
    // Вернуть список каналов
    0
}

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

## Интерфейс WASM API

### Экспортируемые функции

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `plugin_init` | `fn() -> i32` | Инициализация плагина |
| `handle_hook` | `fn(event_ptr: i32, event_len: i32) -> i32` | Обработка хука |
| `can_handle_hook` | `fn(hook_name_ptr: i32, hook_name_len: i32) -> i32` | Проверка хука |
| `can_execute` | `fn(task_ptr: i32, task_len: i32) -> i32` | Проверка задачи |
| `execute` | `fn(task_ptr: i32, task_len: i32) -> i32` | Выполнение задачи |
| `stop` | `fn(task_id: i64) -> i32` | Остановка задачи |
| `send_notification` | `fn(notif_ptr: i32, notif_len: i32) -> i32` | Отправка уведомления |

### Импортируемые функции (хост)

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `semaphore:log` | `fn(level: i32, msg_ptr: i32, msg_len: i32)` | Логирование |
| `semaphore:get_config` | `fn(key_ptr: i32, key_len: i32) -> (ptr: i32, len: i32)` | Получение конфига |
| `semaphore:set_config` | `fn(key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32) -> i32` | Установка конфига |
| `semaphore:call_hook` | `fn(hook_ptr: i32, hook_len: i32, data_ptr: i32, data_len: i32) -> (ptr: i32, len: i32)` | Вызов хука |

### Уровни логирования

- `0` - Error
- `1` - Warn
- `2` - Info
- `3` - Debug
- `4` - Trace

## Сборка всех примеров

```bash
#!/bin/bash
# build_all.sh

set -e

EXAMPLES=("hook_logger" "task_executor" "notification")

for example in "${EXAMPLES[@]}"; do
    echo "Building $example..."
    cd "$example"
    cargo build --target wasm32-wasi --release
    cp target/wasm32-wasi/release/"$example".wasm ../../plugins/
    cd ..
done

echo "All plugins built successfully!"
```
