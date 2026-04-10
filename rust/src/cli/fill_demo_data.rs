//! Утилита для наполнения БД демо-данными
//! Использование: cargo run --release -- fill-demo-data

use std::fs;
use std::path::Path;

/// Проверяет, существует ли файл базы данных
fn check_db_exists(db_path: &str) -> bool {
    Path::new(db_path).exists()
}

/// Формирует приветственное сообщение
fn get_welcome_message() -> String {
    format!(
        "Наполнение SQLite демо-данными для Velum...\n\
        ============================================\n\
        База данных: {{}}\n\
        Подключение к базе данных...",
        "placeholder"
    )
}

/// Формирует сообщение с учётными данными
fn get_credentials_message() -> String {
    format!(
        "============================================\n\
        Учётные данные для входа:\n\
           admin / admin123\n\
           john.doe / admin123\n\
           jane.smith / admin123\n\
           devops / admin123\n\
        ============================================"
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Наполнение SQLite демо-данными для Velum...");
    println!("============================================");

    let db_path = "./data/semaphore.db";

    if !Path::new(db_path).exists() {
        eprintln!("❌ База данных не найдена: {}", db_path);
        eprintln!("   Сначала выполните: ./semaphore.sh init native");
        std::process::exit(1);
    }

    println!("📁 База данных: {}", db_path);
    println!();

    // Читаем SQL файл
    let sql_content = fs::read_to_string("fill-sqlite-demo-data.sql")?;

    // Используем sqlx для выполнения SQL
    let database_url = format!("sqlite:{}", db_path);

    println!("Подключение к базе данных...");

    // Выполняем SQL через внешний процесс sqlite3 или через Rust
    let output = std::process::Command::new("sqlite3")
        .arg(db_path)
        .arg("fill-sqlite-demo-data.sql")
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                println!("✅ Демо-данные успешно добавлены!");
                println!();
                println!("============================================");
                println!("🔐 Учётные данные для входа:");
                println!("   admin / admin123");
                println!("   john.doe / admin123");
                println!("   jane.smith / admin123");
                println!("   devops / admin123");
                println!("============================================");
            } else {
                eprintln!("❌ Ошибка выполнения SQL: {}", String::from_utf8_lossy(&out.stderr));
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Ошибка запуска sqlite3: {}", e);
            eprintln!("   Установите sqlite3 или используйте альтернативный метод");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_check_db_exists_with_temp_file() {
        let temp_path = "/tmp/test_semaphore_db_exists.tmp";
        fs::write(temp_path, "").unwrap();
        assert!(check_db_exists(temp_path));
        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_check_db_not_exists() {
        assert!(!check_db_exists("/tmp/nonexistent_file_that_does_not_exist.db"));
    }

    #[test]
    fn test_check_db_empty_path() {
        assert!(!check_db_exists(""));
    }

    #[test]
    fn test_welcome_message_contains_app_name() {
        let msg = get_welcome_message();
        assert!(msg.contains("Velum"));
    }

    #[test]
    fn test_welcome_message_contains_separator() {
        let msg = get_welcome_message();
        assert!(msg.contains("======"));
    }

    #[test]
    fn test_credentials_message_contains_admin() {
        let msg = get_credentials_message();
        assert!(msg.contains("admin / admin123"));
    }

    #[test]
    fn test_credentials_message_contains_all_users() {
        let msg = get_credentials_message();
        assert!(msg.contains("john.doe"));
        assert!(msg.contains("jane.smith"));
        assert!(msg.contains("devops"));
    }

    #[test]
    fn test_credentials_message_format() {
        let msg = get_credentials_message();
        assert!(msg.starts_with("======"));
        assert!(msg.ends_with("======"));
    }

    #[test]
    fn test_credentials_message_line_count() {
        let msg = get_credentials_message();
        let lines: Vec<&str> = msg.lines().collect();
        // 6 lines: separator header, credentials header, 4 users, separator footer
        assert_eq!(lines.len(), 7);
    }

    #[test]
    fn test_check_db_with_relative_path_nonexistent() {
        assert!(!check_db_exists("data/semaphore.db"));
    }
}
