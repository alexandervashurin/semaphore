//! Hook Logger - WASM плагин для логирования событий
//!
//! Этот плагин демонстрирует базовую функциональность WASM плагинов
//! для Velum UI.

use serde::{Deserialize, Serialize};

// ============================================================================
// Структуры данных
// ============================================================================

/// Событие хука
#[derive(Debug, Deserialize, Serialize)]
struct HookEvent {
    name: String,
    timestamp: String,
    data: serde_json::Value,
    context: PluginContext,
}

/// Контекст плагина
#[derive(Debug, Deserialize, Serialize)]
struct PluginContext {
    plugin_id: String,
    project_id: Option<i64>,
    user_id: Option<i64>,
    task_id: Option<i64>,
}

/// Результат выполнения хука
#[derive(Debug, Serialize)]
struct HookResult {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

// ============================================================================
// Экспортируемые функции
// ============================================================================

/// Инициализация плагина
/// Возвращает 0 при успехе
#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log_info("Hook Logger plugin initialized");
    0
}

/// Обработка хука
/// Принимает указатель и длину JSON события
/// Возвращает 0 при успехе
#[no_mangle]
pub extern "C" fn handle_hook(event_ptr: i32, event_len: i32) -> i32 {
    // В реальной реализации здесь было бы чтение из памяти
    // и парсинг JSON события
    
    log_info("Hook event received");
    
    // Создаём результат
    let result = HookResult {
        success: true,
        data: Some(serde_json::json!({
            "logged": true,
            "plugin": "hook_logger"
        })),
        error: None,
    };
    
    // В реальной реализации нужно записать результат в память
    // и вернуть указатель/длину
    
    0 // Success
}

/// Проверка может ли плагин обработать указанный хук
/// Возвращает 1 если может, 0 если нет
#[no_mangle]
pub extern "C" fn can_handle_hook(hook_name_ptr: i32, hook_name_len: i32) -> i32 {
    // Этот плагин обрабатывает все хуки
    1
}

/// Получение информации о плагине
/// Возвращает указатель на JSON с информацией
#[no_mangle]
pub extern "C" fn get_plugin_info() -> i32 {
    let info = serde_json::json!({
        "id": "hook_logger",
        "name": "Hook Logger",
        "version": "0.1.0",
        "description": "WASM плагин для логирования событий хуков",
        "author": "Velum Team",
        "type": "hook",
        "enabled": true
    });
    
    // В реальной реализации нужно записать в память и вернуть указатель
    0
}

/// Обработка хука перед созданием задачи
#[no_mangle]
pub extern "C" fn task_before_create(data_ptr: i32, data_len: i32) -> i32 {
    log_info("Task before create hook triggered");
    0
}

/// Обработка хука после создания задачи
#[no_mangle]
pub extern "C" fn task_after_create(data_ptr: i32, data_len: i32) -> i32 {
    log_info("Task after create hook triggered");
    0
}

/// Обработка хука перед запуском задачи
#[no_mangle]
pub extern "C" fn task_before_start(data_ptr: i32, data_len: i32) -> i32 {
    log_info("Task before start hook triggered");
    0
}

/// Обработка хука после завершения задачи
#[no_mangle]
pub extern "C" fn task_after_complete(data_ptr: i32, data_len: i32) -> i32 {
    log_info("Task after complete hook triggered");
    0
}

/// Обработка хука после провала задачи
#[no_mangle]
pub extern "C" fn task_after_fail(data_ptr: i32, data_len: i32) -> i32 {
    log_warn("Task failed hook triggered");
    0
}

// ============================================================================
// Хост-функции (импортируются из Velum)
// ============================================================================

#[link(wasm_import_module = "semaphore")]
extern "C" {
    /// Логирование сообщений
    /// level: 0=error, 1=warn, 2=info, 3=debug, 4=trace
    fn log(level: i32, message_ptr: i32, message_len: i32);
}

// ============================================================================
// Helper функции
// ============================================================================

/// Логирование на уровне Info
fn log_info(msg: &str) {
    unsafe {
        log(2, msg.as_ptr() as i32, msg.len() as i32);
    }
}

/// Логирование на уровне Warn
fn log_warn(msg: &str) {
    unsafe {
        log(1, msg.as_ptr() as i32, msg.len() as i32);
    }
}

/// Логирование на уровне Error
fn log_error(msg: &str) {
    unsafe {
        log(0, msg.as_ptr() as i32, msg.len() as i32);
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_init() {
        let result = plugin_init();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_can_handle_hook() {
        let result = can_handle_hook(0, 0);
        assert_eq!(result, 1);
    }
}
