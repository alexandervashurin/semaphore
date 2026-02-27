//! SQL DB Utils - вспомогательные функции
//!
//! Аналог db/sql/SqlDb.go из Go версии (часть 5: утилиты)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use sqlx::Row;

/// Выполняет SQL запрос и возвращает количество затронутых строк
pub async fn execute_query(db: &SqlDb, query: &str) -> Result<u64> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            // Для SQLite используем динамический запрос
            // В production лучше использовать prepared statements
            let result = sqlx::query(query)
                .execute(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

            Ok(result.rows_affected())
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Выполняет SQL запрос и возвращает одну строку
pub async fn query_one<T>(db: &SqlDb, query: &str) -> Result<T>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            let row = sqlx::query_as::<_, T>(query)
                .fetch_one(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            
            Ok(row)
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Выполняет SQL запрос и возвращает все строки
pub async fn query_all<T>(db: &SqlDb, query: &str) -> Result<Vec<T>>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            let rows = sqlx::query_as::<_, T>(query)
                .fetch_all(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            
            Ok(rows)
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Проверяет существует ли таблица
pub async fn table_exists(db: &SqlDb, table_name: &str) -> Result<bool> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            let result = sqlx::query(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?"
            )
            .bind(table_name)
            .fetch_one(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
            .await
            .map_err(|e| Error::Database(e))?;
            
            let count: i64 = result.get(0);
            Ok(count > 0)
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Получает список всех таблиц
pub async fn get_all_tables(db: &SqlDb) -> Result<Vec<String>> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
                .fetch_all(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            
            let tables = result
                .iter()
                .map(|row| row.get::<String, _>(0))
                .collect();
            
            Ok(tables)
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Очищает таблицу (удаляет все данные)
pub async fn truncate_table(db: &SqlDb, table_name: &str) -> Result<()> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            sqlx::query(&format!("DELETE FROM {}", table_name))
                .execute(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            
            Ok(())
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Сбрасывает автоинкремент для таблицы
pub async fn reset_autoincrement(db: &SqlDb, table_name: &str) -> Result<()> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            sqlx::query("DELETE FROM sqlite_sequence WHERE name=?")
                .bind(table_name)
                .execute(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

            Ok(())
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Включает foreign keys
pub async fn enable_foreign_keys(db: &SqlDb) -> Result<()> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            
            Ok(())
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

/// Проверяет включены ли foreign keys
pub async fn foreign_keys_enabled(db: &SqlDb) -> Result<bool> {
    match db.get_dialect() {
        crate::db::sql::types::SqlDialect::SQLite => {
            let result = sqlx::query("PRAGMA foreign_keys")
                .fetch_one(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            
            let enabled: i64 = result.get(0);
            Ok(enabled == 1)
        }
        _ => Err(Error::Other("Only SQLite supported for now".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_utils.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём тестовую таблицу
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_table (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_table_exists() {
        let db = create_test_db().await;
        
        let exists = table_exists(&db, "test_table").await.unwrap();
        assert!(exists);
        
        let exists = table_exists(&db, "nonexistent_table").await.unwrap();
        assert!(!exists);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_all_tables() {
        let db = create_test_db().await;
        
        let tables = get_all_tables(&db).await.unwrap();
        assert!(tables.contains(&"test_table".to_string()));
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_truncate_table() {
        let db = create_test_db().await;
        
        // Вставляем данные
        sqlx::query("INSERT INTO test_table (name) VALUES (?)")
            .bind("test")
            .execute(db.get_sqlite_pool().unwrap())
            .await
            .unwrap();
        
        // Очищаем таблицу
        truncate_table(&db, "test_table").await.unwrap();
        
        // Проверяем что таблица пуста
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_table")
            .fetch_one(db.get_sqlite_pool().unwrap())
            .await
            .unwrap();
        
        assert_eq!(count, 0);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_enable_foreign_keys() {
        let db = create_test_db().await;
        
        enable_foreign_keys(&db).await.unwrap();
        
        let enabled = foreign_keys_enabled(&db).await.unwrap();
        assert!(enabled);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[test]
    fn test_utils_functions() {
        // Тест для проверки что функции существуют
        let _ = table_exists;
        let _ = get_all_tables;
        let _ = truncate_table;
        let _ = reset_autoincrement;
        let _ = enable_foreign_keys;
        let _ = foreign_keys_enabled;
    }
}
