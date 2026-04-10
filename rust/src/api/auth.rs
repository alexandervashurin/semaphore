//! API - Auth re-exports
//!
//! Реэкспорт утилит аутентификации. Основные обработчики — в api/handlers/auth.rs.

pub use crate::api::extractors::extract_token_from_header;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_header_is_exported() {
        // Verify the re-export is accessible
        let _fn_ptr: fn(Option<&str>) -> Option<&str> = extract_token_from_header;
    }

    #[test]
    fn test_extract_token_from_bearer_header() {
        let result = extract_token_from_header(Some("Bearer test-token-123"));
        assert_eq!(result, Some("test-token-123"));
    }

    #[test]
    fn test_extract_token_from_missing_header() {
        let result = extract_token_from_header(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_token_from_invalid_format() {
        let result = extract_token_from_header(Some("InvalidFormat"));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_token_basic_prefix() {
        let result = extract_token_from_header(Some("Basic dXNlcjpwYXNz"));
        // Should return None because it's not Bearer format
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_token_empty_bearer() {
        let result = extract_token_from_header(Some("Bearer "));
        assert_eq!(result, Some(""));
    }

    #[test]
    fn test_extract_token_complex_jwt() {
        let token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature";
        let result = extract_token_from_header(Some(token));
        assert_eq!(result, Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test.signature"));
    }
}
