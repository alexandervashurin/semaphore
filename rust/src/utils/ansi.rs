//! ANSI Codes Utility
//!
//! Утилиты для работы с ANSI escape sequences

use regex::Regex;
use once_cell::sync::Lazy;

/// Regex для удаления ANSI escape sequences
static ANSI_CODE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap()
});

/// Удаляет ANSI escape sequences из строки
///
/// # Пример
///
/// ```
/// use crate::utils::ansi::clear_from_ansi_codes;
///
/// let input = "\x1b[31mRed Text\x1b[0m";
/// let output = clear_from_ansi_codes(input);
/// assert_eq!(output, "Red Text");
/// ```
pub fn clear_from_ansi_codes(s: &str) -> String {
    ANSI_CODE_RE.replace_all(s, "").to_string()
}

/// Проверяет, содержит ли строка ANSI escape sequences
pub fn contains_ansi_codes(s: &str) -> bool {
    ANSI_CODE_RE.is_match(s)
}

/// Удаляет ANSI escape sequences и обрезает пробелы
pub fn strip_ansi_and_trim(s: &str) -> String {
    clear_from_ansi_codes(s).trim().to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_from_ansi_codes_red() {
        let input = "\x1b[31mRed Text\x1b[0m";
        let output = clear_from_ansi_codes(input);
        assert_eq!(output, "Red Text");
    }

    #[test]
    fn test_clear_from_ansi_codes_multiple() {
        let input = "\x1b[1m\x1b[32mGreen Bold\x1b[0m";
        let output = clear_from_ansi_codes(input);
        assert_eq!(output, "Green Bold");
    }

    #[test]
    fn test_clear_from_ansi_codes_plain() {
        let input = "Plain Text";
        let output = clear_from_ansi_codes(input);
        assert_eq!(output, "Plain Text");
    }

    #[test]
    fn test_contains_ansi_codes() {
        assert!(contains_ansi_codes("\x1b[31mRed\x1b[0m"));
        assert!(!contains_ansi_codes("Plain Text"));
    }

    #[test]
    fn test_strip_ansi_and_trim() {
        let input = "  \x1b[31mRed Text\x1b[0m  ";
        let output = strip_ansi_and_trim(input);
        assert_eq!(output, "Red Text");
    }
}
