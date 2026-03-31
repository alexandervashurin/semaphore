//! API - Auth re-exports
//!
//! Реэкспорт утилит аутентификации. Основные обработчики — в api/handlers/auth.rs.

pub use crate::api::extractors::extract_token_from_header;
