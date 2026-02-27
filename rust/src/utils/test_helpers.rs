//! Test Helpers
//!
//! Вспомогательные функции для тестирования

use rand::{distributions::Alphanumeric, Rng};

/// Генерирует случайную строку заданной длины
///
/// # Пример
///
/// ```
/// use crate::utils::test_helpers::rand_string;
///
/// let s = rand_string(10);
/// assert_eq!(s.len(), 10);
/// ```
pub fn rand_string(n: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

/// Генерирует случайное число в диапазоне
pub fn rand_range(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min..=max)
}

/// Генерирует случайный boolean
pub fn rand_bool() -> bool {
    rand::thread_rng().gen_bool(0.5)
}

/// Генерирует случайный email для тестов
pub fn rand_email() -> String {
    format!("test{}@example.com", rand_string(8))
}

/// Генерирует случайный username для тестов
pub fn rand_username() -> String {
    format!("user_{}", rand_string(6))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_string_length() {
        let s = rand_string(10);
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_rand_string_different_lengths() {
        for len in [5, 10, 20, 50, 100] {
            let s = rand_string(len);
            assert_eq!(s.len(), len);
        }
    }

    #[test]
    fn test_rand_string_alphanumeric() {
        let s = rand_string(100);
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_rand_range() {
        for _ in 0..100 {
            let n = rand_range(1, 10);
            assert!(n >= 1);
            assert!(n <= 10);
        }
    }

    #[test]
    fn test_rand_bool() {
        // Просто проверяем, что работает
        let _ = rand_bool();
    }

    #[test]
    fn test_rand_email_format() {
        let email = rand_email();
        assert!(email.contains("@example.com"));
        assert!(email.starts_with("test"));
    }

    #[test]
    fn test_rand_username_format() {
        let username = rand_username();
        assert!(username.starts_with("user_"));
    }

    #[test]
    fn test_rand_string_uniqueness() {
        let s1 = rand_string(16);
        let s2 = rand_string(16);
        // Маловероятно, но возможно совпадение
        // Просто проверяем, что генерация работает
        assert_eq!(s1.len(), 16);
        assert_eq!(s2.len(), 16);
    }
}
