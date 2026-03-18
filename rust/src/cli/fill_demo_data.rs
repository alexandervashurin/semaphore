//! Утилита для наполнения БД демо-данными
//! Использование: cargo run --release -- fill-demo-data

use std::fs;
use std::path::Path;

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
