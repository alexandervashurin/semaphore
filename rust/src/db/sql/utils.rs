//! SQL DB Utils - вспомогательные функции (PostgreSQL)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use sqlx::Row;

fn pg_pool(db: &SqlDb) -> Result<&sqlx::PgPool> {
    db.get_postgres_pool()
        .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
}

/// Выполняет SQL запрос и возвращает количество затронутых строк
pub async fn execute_query(db: &SqlDb, query: &str) -> Result<u64> {
    let result = sqlx::query(query)
        .execute(pg_pool(db)?)
        .await
        .map_err(Error::Database)?;
    Ok(result.rows_affected())
}

/// Проверяет существует ли таблица
pub async fn table_exists(db: &SqlDb, table_name: &str) -> Result<bool> {
    let result = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_name = $1"
    )
    .bind(table_name)
    .fetch_optional(pg_pool(db)?)
    .await
    .map_err(Error::Database)?;
    Ok(result.is_some())
}

/// Получает список всех таблиц
pub async fn get_all_tables(db: &SqlDb) -> Result<Vec<String>> {
    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'"
    )
    .fetch_all(pg_pool(db)?)
    .await
    .map_err(Error::Database)?;

    Ok(rows.into_iter().map(|row| row.get::<String, _>("table_name")).collect())
}

/// Очищает таблицу (удаляет все данные)
pub async fn truncate_table(db: &SqlDb, table_name: &str) -> Result<()> {
    sqlx::query(&format!("TRUNCATE TABLE {}", table_name))
        .execute(pg_pool(db)?)
        .await
        .map_err(Error::Database)?;
    Ok(())
}

/// Сбрасывает автоинкремент для таблицы (PostgreSQL sequences)
pub async fn reset_autoincrement(db: &SqlDb, table_name: &str) -> Result<()> {
    sqlx::query(&format!("ALTER SEQUENCE {}_id_seq RESTART WITH 1", table_name))
        .execute(pg_pool(db)?)
        .await
        .map_err(Error::Database)?;
    Ok(())
}

/// Foreign keys всегда включены в PostgreSQL
pub async fn enable_foreign_keys(_db: &SqlDb) -> Result<()> {
    Ok(())
}

/// Foreign keys всегда включены в PostgreSQL
pub async fn foreign_keys_enabled(_db: &SqlDb) -> Result<bool> {
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utils_functions_exist() {
        let _ = table_exists;
        let _ = get_all_tables;
        let _ = truncate_table;
        let _ = reset_autoincrement;
        let _ = enable_foreign_keys;
        let _ = foreign_keys_enabled;
    }
}
