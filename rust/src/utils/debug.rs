//! Debug Utilities
//!
//! Утилиты для отладки

use std::fmt::Debug;
use tracing::{debug, info};

/// Получает ID текущего goroutine (потока)
/// В Rust это ID потока
pub fn thread_id() -> u64 {
    use std::thread;
    // В Rust нет прямого аналога goroutine ID,
    // но можно использовать ID потока
    format!("{:?}", thread::current().id())
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
        .parse()
        .unwrap_or(0)
}

/// Логирует сообщение с ID потока
pub fn log_thread_id<T: Debug>(msg: &str, data: &T) {
    let id = thread_id();
    info!("{} (thread_id={}): {:?}", msg, id, data);
}

/// Логирует debug сообщение с ID потока
pub fn debug_thread_id<T: Debug>(msg: &str, data: &T) {
    let id = thread_id();
    debug!("{} (thread_id={}): {:?}", msg, id, data);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_id() {
        let id = thread_id();
        assert!(id > 0);
    }

    #[test]
    fn test_log_thread_id() {
        let data = "test data";
        log_thread_id("Test message", &data);
        // Визуальная проверка в логах
    }
}
