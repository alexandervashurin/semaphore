//! Модуль для работы с shell
//!
//! Предоставляет функции для экранирования и очистки строк для shell

/// Экранирует строку для безопасного использования в shell
///
/// Возвращает shell-escaped версию строки. Результат может безопасно
/// использоваться как один токен в shell командной строке.
///
/// # Пример
///
/// ```
/// use semaphore_ffi::utils::shell::shell_quote;
///
/// assert_eq!(shell_quote("hello world"), "'hello world'");
/// assert_eq!(shell_quote("hello"), "hello");
/// assert_eq!(shell_quote(""), "''");
/// ```
pub fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }

    // Проверяем, содержит ли строка специальные символы
    if needs_quoting(s) {
        // Экранируем одинарные кавычки
        format!("'{}'", s.replace('\'', "'\"'\"'"))
    } else {
        s.to_string()
    }
}

/// Проверяет, нуждается ли строка в экранировании
fn needs_quoting(s: &str) -> bool {
    s.chars().any(|c| {
        // Специальные символы shell
        matches!(
            c,
            ' ' | '\t' | '\n'
                | '\\'
                | '\''
                | '"'
                | '`'
                | '$'
                | '!'
                | '{'
                | '}'
                | '('
                | ')'
                | '['
                | ']'
                | '<'
                | '>'
                | '|'
                | '&'
                | ';'
                | '*'
                | '?'
                | '~'
                | '#'
        )
    })
}

/// Удаляет небезопасные (непечататаемые) символы из строки
///
/// Удаляет управляющие символы, оставляя только печатаемые.
/// Результат может безопасно отображаться в терминале.
///
/// # Пример
///
/// ```
/// use semaphore_ffi::utils::shell::shell_strip_unsafe;
///
/// assert_eq!(shell_strip_unsafe("hello\x00world"), "helloworld");
/// assert_eq!(shell_strip_unsafe("normal text"), "normal text");
/// ```
pub fn shell_strip_unsafe(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect()
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_quote_empty() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn test_shell_quote_simple() {
        assert_eq!(shell_quote("hello"), "hello");
        assert_eq!(shell_quote("test123"), "test123");
        assert_eq!(shell_quote("hello-world"), "hello-world");
        assert_eq!(shell_quote("path/to/file"), "path/to/file");
    }

    #[test]
    fn test_shell_quote_with_spaces() {
        assert_eq!(shell_quote("hello world"), "'hello world'");
        assert_eq!(shell_quote("test with spaces"), "'test with spaces'");
    }

    #[test]
    fn test_shell_quote_with_special_chars() {
        assert_eq!(shell_quote("hello'world"), "'hello'\"'\"'world'");
        assert_eq!(shell_quote("test$var"), "'test$var'");
        assert_eq!(shell_quote("file*.txt"), "'file*.txt'");
        assert_eq!(shell_quote("cmd && other"), "'cmd && other'");
    }

    #[test]
    fn test_shell_quote_with_newlines() {
        assert_eq!(shell_quote("line1\nline2"), "'line1\nline2'");
        assert_eq!(shell_quote("line1\tline2"), "'line1\tline2'");
    }

    #[test]
    fn test_shell_strip_unsafe_basic() {
        assert_eq!(shell_strip_unsafe("hello world"), "hello world");
        assert_eq!(shell_strip_unsafe("normal text"), "normal text");
    }

    #[test]
    fn test_shell_strip_unsafe_control_chars() {
        assert_eq!(shell_strip_unsafe("hello\x00world"), "helloworld");
        assert_eq!(shell_strip_unsafe("test\x01\x02"), "test");
        // \n и \t сохраняются как whitespace
        assert_eq!(shell_strip_unsafe("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_shell_strip_unsafe_preserves_printable() {
        let input = "Hello! @#$%^&*() World 123";
        let output = shell_strip_unsafe(input);
        assert_eq!(output, "Hello! @#$%^&*() World 123");
    }

    #[test]
    fn test_needs_quoting() {
        assert!(!needs_quoting("hello"));
        assert!(!needs_quoting("test123"));
        assert!(!needs_quoting("path/to/file"));
        
        assert!(needs_quoting("hello world"));
        assert!(needs_quoting("test$var"));
        assert!(needs_quoting("file*.txt"));
        assert!(needs_quoting("cmd && other"));
    }
}
