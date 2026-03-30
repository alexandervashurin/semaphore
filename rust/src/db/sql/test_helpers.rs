//! Вспомогательные функции для интеграционных тестов с PostgreSQL.
//!
//! Задайте `VELUM_TEST_DATABASE_URL` (например `postgres://user:pass@localhost:5432/velum_test`).

#[cfg(test)]
pub fn test_database_url() -> Option<String> {
    std::env::var("VELUM_TEST_DATABASE_URL").ok()
}
